extern crate core;

mod amount;
mod ledger;

use std::env;
use std::path::Path;
use std::process::exit;
use csv::{ReaderBuilder, Trim};
use crate::amount::Amount;
use crate::ledger::{Client, Ledger, Transaction, TransactionKind};
use serde::{Serialize, Deserialize};

#[derive(Debug)]
enum Error {
    Read(csv::Error),
}

impl From<csv::Error> for Error {
    fn from(csv_error: csv::Error) -> Self {
        Error::Read(csv_error)
    }
}

#[derive(Debug, Deserialize)]
struct ApiTransaction {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    kind: TransactionKind,
    client: u16,
    tx: u64,
    amount: Amount,
}

impl From<&ApiTransaction> for Transaction {
    fn from(transaction: &ApiTransaction) -> Self {
        Transaction::new(transaction.tx, transaction.client, transaction.kind, transaction.amount)
    }
}

#[derive(Debug, Serialize)]
struct ApiClient {
    client: u16,
    available: Amount,
    held: Amount,
    total: Amount,
    locked: bool,
}

impl From<&Client> for ApiClient {
    fn from(client: &Client) -> Self {
        ApiClient {
            client: client.id(),
            available: client.available(),
            held: client.held(),
            total: client.total(),
            locked: client.locked(),
        }
    }
}

fn run<P: AsRef<Path>>(path: P) -> Result<String, Error> {
    let mut reader = ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .from_path(path)?;

    let ledger = Ledger::new();

    for result in reader.deserialize() {
        let transaction: ApiTransaction = result?;
        println!("{:?}", transaction);
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
