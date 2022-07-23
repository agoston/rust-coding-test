extern crate core;

mod amount;
mod ledger;

use std::{env, io};
use std::path::Path;
use std::process::exit;
use csv::{Reader, ReaderBuilder, Trim, Writer, WriterBuilder};
use crate::amount::{Amount, ZERO};
use crate::ledger::{Client, Ledger, Transaction, TransactionKind};
use serde::{Serialize, Deserialize};

#[derive(Debug)]
enum Error {
    Read(csv::Error),
    Write(io::Error),
}

impl From<csv::Error> for Error {
    fn from(error: csv::Error) -> Self {
        Error::Read(error)
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Write(error)
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

    let mut wtr = WriterBuilder::new()
        .from_writer(io::stdout());

    process_transactions(&mut reader, &mut wtr)?;

    Ok("ok".to_string())
}

fn process_transactions<R: io::Read, W: io::Write>(reader: &mut Reader<R>, wtr: &mut Writer<W>) -> Result<String, Error> {
    let mut ledger = Ledger::new();

    for result in reader.deserialize() {
        let transaction: ApiTransaction = result?;
        // println!("{:?}", transaction);
        ledger.mutate((&transaction).into());
    }

    for x in ledger.iter() {
        let client: ApiClient = x.1.into();
        wtr.serialize(client)?;
    }
    wtr.flush()?;

    Ok("ok".to_string())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        eprintln!("specify input file name");
        exit(1);
    }

    match run(&args[1]) {
        Ok(_res) => {}
        Err(err) => {
            println!("{:?}", err)
        }
    }
}

#[cfg(test)]
mod tests {
    use csv::{ReaderBuilder, Trim, WriterBuilder};
    use crate::process_transactions;

    pub fn assert_transaction(data: &str, result: &str) {
        let mut rdr = ReaderBuilder::new()
            .trim(Trim::All)
            .flexible(true)
            .has_headers(false)
            .from_reader(data.as_bytes());

        let mut wrt = WriterBuilder::new()
            .has_headers(false)
            .flexible(true)
            .from_writer(Vec::new());

        process_transactions(&mut rdr, &mut wrt).unwrap();
        let bytes = wrt.into_inner().unwrap();

        assert_eq!(String::from_utf8(bytes).unwrap(), result.to_string())
    }

    #[test]
    pub fn basic() {
        assert_transaction(
            concat!(
            "deposit, 1, 1, 1.0\n",
            "deposit, 2, 2, 2.0\n",
            "deposit, 1, 3, 2.0\n",
            "withdrawal, 1, 4, 1.5\n",
            "withdrawal, 2, 5, 3.0\n"),
            concat!(
            "1,1.5,0,1.5,false\n",
            "2,2,0,2,false\n"
            ),
        )
    }

    #[test]
    pub fn tx_reference_fail() {
        assert_transaction("deposit, 1, 1, 10\ndispute, 1, 2", "1,10,0,10,false");

        assert_transaction("deposit, 1, 1, 10\nresolve, 1, 2", "1,10,0,10,false");

        assert_transaction("deposit, 1, 1, 10\ndispute,1,1\nresolve,1,2", "1,0,10,10,false");
    }

    #[test]
    pub fn dispute() {
        assert_transaction(
            concat!(
            "deposit, 1, 1, 10\n",
            "dispute, 1, 2, 2.6666\n"),
            concat!(
            "1,7.3334,2.6666,10,false\n"
            ),
        )
    }
}