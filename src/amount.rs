use std::fmt;
use std::fmt::Formatter;
use std::num::ParseIntError;
use std::str::FromStr;
use std::ops::{Add, Mul, Sub};

use crate::amount::Error::{Malformed, PrecisionTooHigh};

/// fixed point precision with 4 fraction digits, to act as monetary type
/// NB: only the operators +-* are implemented!
/// NB: there is no overflow check, but limit is +-2^49, well within practical limits of monetary types
#[derive(Debug, Clone, Copy, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct Amount {
    amount_fx4: i64,
}

impl Amount {
    fn new(whole: i64, fraction: u32) -> Amount {
        Amount { amount_fx4: whole * 10000 + (fraction as i64) }
    }
}

impl Add for Amount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Amount { amount_fx4: self.amount_fx4 + rhs.amount_fx4 }
    }
}

impl Mul for Amount {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Amount { amount_fx4: (self.amount_fx4 * rhs.amount_fx4) / 10000 }
    }
}

impl Sub for Amount {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Amount { amount_fx4: self.amount_fx4 - rhs.amount_fx4 }
    }
}

#[derive(Debug)]
pub enum Error {
    NoInput,
    Malformed,
    PrecisionTooHigh,
}

impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Self {
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

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let whole = self.amount_fx4 / 10000;
        let mut fraction = self.amount_fx4 % 10000;

        if fraction == 0 {
            write!(f, "{}", whole)
        } else {
            let mut width = 4;
            // get rid of 'ending zeroes'; this is a fraction after all
            while fraction % 10 == 0 {
                fraction = fraction / 10;
                width = width - 1;
            }
            write!(f, "{}.{:0width$}", whole, fraction)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::amount::Amount;

    #[test]
    fn parse_test() {
        let balance: Amount = "12.5".parse().unwrap();
        assert_eq!(balance.amount_fx4, 125000);
        assert_eq!(balance.to_string(), "12.5");
    }

    #[test]
    fn into_test() {
        let balance: Amount = Amount::from_str("9.05").unwrap();
        assert_eq!(balance.amount_fx4, 90500);
        assert_eq!(balance.to_string(), "9.05");
    }

    #[test]
    fn new_test() {
        let balance = Amount::new(24, 4321);
        assert_eq!(balance.amount_fx4, 244321);
        assert_eq!(balance.to_string(), "24.4321");
    }

    #[test]
    fn add_test() {
        let balance = Amount::from_str("11.99").unwrap();
        let transaction = Amount::from_str("9.99").unwrap();
        let new_balance = balance + transaction;
        assert_eq!(new_balance.to_string(), "21.98");
    }

    #[test]
    fn sub_test() {
        let balance = Amount::from_str("11.99").unwrap();
        let transaction = Amount::from_str("9.99").unwrap();
        let new_balance = balance - transaction;
        assert_eq!(new_balance.to_string(), "2");
    }

    #[test]
    fn mul_test() {
        let balance = Amount::from_str("11.99").unwrap();
        let transaction = Amount::from_str("3").unwrap();
        let new_balance = balance * transaction;
        assert_eq!(new_balance.to_string(), "35.97");
    }
}