use std::collections::HashMap;
use std::io::BufRead;
use std::ops::Div;
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
                fmt_num(get_average(val.iter())),
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
    println!("{:?}", num_chars);
    let mut s = String::with_capacity(4);
    for (i, d) in num_chars {
        if i == 2 {
            s.push('.');
        }
        s.push(d);
    }
    s.chars().rev().collect()
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
mod tests{
    use super::*;
    
    #[test]
    fn test_fmt_num(){
        let num = -789;
        let num2 = 678;
        println!("num: {}, num2: {}", fmt_num(num), fmt_num(num2));
        assert_eq!("-9.87", fmt_num(-987));
    }
}