use std::num::ParseIntError;
use std::str::FromStr;
use crate::amount::Error::{Malformed, PrecisionTooHigh};

/// fixed point precision with 4 fraction digits
pub struct Amount {
    amount_fx4: i64,
}

impl Amount {
    fn new(whole: i64, fraction: u32) -> Amount {
        Amount { amount_fx4: whole * 10000 + (fraction as i64) }
    }
}

pub enum Error {
    NoInput,
    Malformed,
    PrecisionTooHigh,
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Malformed
    }
}

impl FromStr for Amount {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split_input = s.split('.');
        let whole: i64 = split_input.next().ok_or(Error::NoInput)?.parse()?;
        let fraction_str_opt = split_input.next();
        if fraction_str_opt.is_none() {
            return Ok(Amount::new(whole, 0));
        }

        let fraction_str = fraction_str_opt.unwrap();
        let fraction_len = fraction_str.len();

        let parsed_fraction: u32 = fraction_str.parse()?;
        return match fraction_len {
            0 => { Err(Malformed) }
            1..=4 => { Ok(Amount::new(whole, parsed_fraction * 10_u32.pow((4 - fraction_len) as u32))) }
            _ => { Err(PrecisionTooHigh) }
        };
    }
}

impl From<Amount> for String {
    fn from(input: Amount) -> Self {
        let whole = input.amount_fx4 / 10000;
        let fraction = input.amount_fx4 % 10000;
        format!("{}.{:04}", whole, fraction)
    }
}

pub struct Transaction {
    id: u64,
    client_id: u64,
    /// fixed point with 4 decimals (=actual_value*10_000)
    amount_fx4: u64,
}

pub struct Client {
    id: u64,
    /// fixed point with 4 decimals (=actual_value*10_000)
    balance_fx4: u64,
}

impl Transaction {
    pub fn new(id: u64, client_id: u64, amount_fx4: u64) -> Transaction {
        Transaction { id, client_id, amount_fx4 }
    }
}

impl Client {
    pub fn new(id: u64, balance_fx4: u64) -> Client {
        Client { id, balance_fx4 }
    }
}

