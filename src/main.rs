extern crate core;

mod amount;
mod ledger;

use std::env;
use std::path::Path;
use std::process::exit;
use csv::{ReaderBuilder, Trim};

#[derive(Debug)]
enum Error {
    Read(csv::Error),
}

impl From<csv::Error> for Error {
    fn from(csv_error: csv::Error) -> Self {
        Error::Read(csv_error)
    }
}

fn run<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .from_path(path)?;

    for result in reader.records() {
        let record = result?;
        println!("{:?}", record);
    }
    Ok("ok".to_string())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        eprintln!("specify input file name");
        exit(1);
    }

    match run(&args[1]) {
        Ok(res) => {
            println!("Got {}", res)
        }
        Err(err) => {
            println!("{:?}", err)
        }
    }
}
