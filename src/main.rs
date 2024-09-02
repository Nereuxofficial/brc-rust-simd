#![feature(portable_simd)]
#![feature(iter_array_chunks)]
use std::collections::HashMap;
use std::io::BufRead;
use std::ops::Div;
use std::simd::num::{SimdFloat, SimdInt};
use std::simd::{f32x8, i32x8};
use std::{error::Error, io::BufReader};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args();
    let measurements = std::fs::File::open(args.nth(1).unwrap())?;
    let reader = BufReader::new(measurements);
    let mut measurements: HashMap<String, Vec<i32>> = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        let mut parts = line.split(';');
        let name = parts.next().unwrap().to_string();
        let value: i32 = parts
            .next()
            .unwrap()
            .chars()
            .filter(|c| *c != '.')
            .collect::<String>()
            .parse()
            .unwrap();
        match measurements.get_mut(&name) {
            Some(m) => m.push(value),
            None => {
                measurements.insert(name, vec![value]);
            }
        }
    }
    let mut output = measurements
        .into_iter()
        .map(|(name, val)| {
            format!(
                "{name};{};{};{}\n",
                fmt_num(*val.iter().min().unwrap()),
                fmt_num(get_average_simd(&mut val.iter())),
                fmt_num(*val.iter().max().unwrap())
            )
        })
        .collect::<Vec<_>>();
    output.sort();
    for line in output {
        println!("{line}");
    }

    Ok(())
}

fn fmt_num(num: i32) -> String {
    let num_str = num.to_string();
    let num_chars = num_str.chars().rev().enumerate();
    let mut s = String::with_capacity(4);
    for (i, d) in num_chars {
        if i == 2 {
            s.push('.');
        }
        s.push(d);
    }
    s.chars().rev().collect()
}

fn get_average_simd<'a, I: Iterator<Item = &'a i32>>(iter: &mut I) -> i32 {
    let mut count = 0;
    let sum_matrix = iter
        .copied()
        .array_chunks::<8>()
        .map(i32x8::from_array)
        .fold(i32x8::splat(0), |acc, x| {
            count += 4;
            acc + x
        });
    let mut remain = 0;
    for _ in 0..8 {
        if let Some(v) = iter.next() {
            remain += v;
        } else {
            break;
        }
    }
    (sum_matrix.reduce_sum() + remain)
        .checked_div(count)
        .unwrap_or(0)
}

fn get_average<'a, I: Iterator<Item = &'a i32>>(iter: I) -> i32 {
    let mut count = 0;
    iter.fold(0, |acc, x| {
        count += 1;
        acc + x
    })
    .div(count)
}
#[cfg(debug_assertions)]
mod tests {
    use super::*;

    #[test]
    fn test_fmt_num() {
        let num = -789;
        let num2 = 678;
        println!("num: {}, num2: {}", fmt_num(num), fmt_num(num2));
        assert_eq!("-9.87", fmt_num(-987));
    }
}
