#![feature(portable_simd)]
#![feature(iter_array_chunks)]
#![feature(array_chunks)]
#![feature(slice_as_chunks)]
use std::collections::HashMap;
use std::io::{BufRead, Read};
use std::ops::Div;
use std::simd::num::{SimdFloat, SimdInt};
use std::simd::{f32x8, i16x16, i32x8};
use std::{error::Error, io::BufReader};

struct City {
    count: i16,
    sum: i16,
    max: i16,
    min: i16,
}

impl Default for City {
    fn default() -> Self {
        Self {
            count: 0,
            sum: 0,
            max: i16::min_value(),
            min: i16::max_value(),
        }
    }
}

impl City {
    fn add(&mut self, value: i16) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    let mut measurements = std::fs::File::open(args.nth(1).unwrap())?;
    let mut reader = String::new();
    measurements.read_to_string(&mut reader).unwrap();
    let mut measurements: HashMap<&str, City> = HashMap::new();
    for line in reader.lines() {
        let (name, v) = line.split_once(';').unwrap();
        let value: i16 = v
            .chars()
            .filter(|c| *c != '.')
            .collect::<String>()
            .parse()
            .unwrap();
        measurements
            .entry(name)
            .or_insert(Default::default())
            .add(value);
    }
    let mut output = measurements
        .into_iter()
        .map(|(name, city)| {
            format!(
                "{name};{};{};{}\n",
                fmt_num(city.min),
                fmt_num(city.sum.div(city.count)),
                fmt_num(city.max)
            )
        })
        .collect::<Vec<_>>();
    output.sort();
    for line in output {
        println!("{line}");
    }

    Ok(())
}

fn fmt_num(num: i16) -> String {
    let num_str = num.to_string();
    let num_chars = num_str.chars().rev().enumerate();
    let mut s = String::with_capacity(4);
    for (i, d) in num_chars {
        if i == 1 {
            s.push('.');
        }
        s.push(d);
    }
    s.chars().rev().collect()
}

fn get_average_simd(slice: &[i16]) -> i16 {
    let mut count = 0;
    let (chunks, remainder) = slice.as_chunks();
    let sum_matrix = chunks
        .iter()
        .map(|array: &[i16; 16]| i16x16::from_array(*array))
        .fold(i16x16::splat(0), |acc, x| {
            count += 4;
            acc + x
        });
    let remain: i16 = remainder.iter().sum();
    (sum_matrix.reduce_sum() + remain)
        .checked_div(count)
        .unwrap_or(0)
}

fn get_average(slice: &[i16]) -> i16 {
    let mut count = 0;
    slice
        .iter()
        .fold(0, |acc, x| {
            count += 1;
            acc + x
        })
        .div(count)
}
#[cfg(debug_assertions)]
mod tests {
    use std::time::{Duration, Instant};

    use rand::{distributions::Standard, thread_rng, Rng};

    use super::*;

    #[test]
    fn test_fmt_num() {
        let num = -789;
        let num2 = 678;
        println!("num: {}, num2: {}", fmt_num(num), fmt_num(num2));
        assert_eq!("-98.7", fmt_num(-987));
    }

    #[test]
    fn test_get_avg_simd() {
        let mut rng = thread_rng();
        // Measure get_avg_simd for increasing numbers of input values. Output the data recorded as JSON
        struct Measurement {
            input_size: usize,
            old_time: Duration,
            new_time: Duration,
        }
        const max_size: usize = 100;
        const samples: usize = 10;
        for i in 0..max_size {
            let results = [0; max_size];
            let measurements = [0; samples];
            for s in 0..samples {
                let input = rng.sample_iter(&Standard).take(i).map(|f| &f);
                let start = Instant::now();
                get_average_simd(&mut input);
                let time = Instant::now().duration_since(start);
            }
        }
    }
}
