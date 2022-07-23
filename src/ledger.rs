use std::collections::HashMap;
use std::ops::Deref;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::amount::{Amount, ZERO};
use crate::ledger::TransactionError::{ClientLocked, NegativeBalance, NegativeTransaction, ReferencedTransactionNonexistent};
use crate::ledger::TransactionKindConversionError::NonExistentValue;
use crate::TransactionKind::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionKind {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug)]
pub enum TransactionKindConversionError {
    NonExistentValue
}

impl FromStr for TransactionKind {
    type Err = TransactionKindConversionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "deposit" => { Ok(Deposit) }
            "withdrawal" => { Ok(Withdrawal) }
            "dispute" => { Ok(Dispute) }
            "resolve" => { Ok(Resolve) }
            "chargeback" => { Ok(Chargeback) }
            _ => { Err(NonExistentValue) }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Transaction {
    id: u64,
    client_id: u16,
    kind: TransactionKind,
    amount: Amount,
}

impl Transaction {
    pub fn new(id: u64, client_id: u16, kind: TransactionKind, amount: Amount) -> Transaction {
        Transaction { id, client_id, kind, amount }
    }
}

#[derive(Debug)]
pub enum TransactionError {
    NegativeBalance,
    NegativeTransaction,
    ClientLocked,
    ReferencedTransactionNonexistent,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct Client {
    id: u16,
    available: Amount,
    held: Amount,
    locked: bool,
}

impl Client {
    fn new(id: u16) -> Client {
        Client {
            id,
            ..Default::default()
        }
    }

    pub fn id(&self) -> u16 { self.id }
    pub fn available(&self) -> Amount { self.available }
    pub fn held(&self) -> Amount { self.held }
    pub fn locked(&self) -> bool { self.locked }
    pub fn total(&self) -> Amount { self.available + self.held }

    pub fn deposit(self, amount: Amount) -> Result<Self, TransactionError> {
        Ok(Client {
            available: self.available + amount,
            ..self
        })
    }

    pub fn withdrawal(self, amount: Amount) -> Result<Self, TransactionError> {
        if amount > self.available { return Err(NegativeBalance); }
        Ok(Client {
            available: self.available - amount,
            ..self
        })
    }

    pub fn dispute(self, amount: Amount) -> Result<Self, TransactionError> {
        // FIXME: `available` can go negative, should add sanity check
        Ok(Client {
            available: self.available - amount,
            held: self.held + amount,
            ..self
        })
    }

    pub fn resolve(self, amount: Amount) -> Result<Self, TransactionError> {
        // FIXME: `held` can go negative, should add sanity check
        Ok(Client {
            available: self.available + amount,
            held: self.held - amount,
            ..self
        })
    }

    pub fn chargeback(self, amount: Amount) -> Result<Self, TransactionError> {
        // FIXME: `held` can go negative, should add sanity check
        Ok(Client {
            held: self.held - amount,
            locked: true,
            ..self
        })
    }
}

#[derive(Debug, Default)]
pub struct Ledger {
    clients: HashMap<u16, Client>,
    transactions: HashMap<u64, Transaction>,
}

impl Deref for Ledger {
    type Target = HashMap<u16, Client>;

    fn deref(&self) -> &Self::Target {
        &self.clients
    }
}

impl Ledger {
    pub fn new() -> Self {
        Ledger { ..Default::default() }
    }

    pub fn mutate(&mut self, transaction: Transaction) -> Result<Client, TransactionError> {
        // sanity check: transaction amount is not negative
        if transaction.amount < *ZERO { return Err(NegativeTransaction); }

        let old_client = match self.clients.get(&transaction.client_id) {
            None => { Client::new(transaction.client_id) }
            Some(x) => { x.clone() }
        };

        // sanity check: locked clients can't do anything
        if old_client.locked { return Err(ClientLocked); }

        let new_client = match transaction.kind {
            Deposit => {
                old_client.deposit(transaction.amount).map(|c| {
                    self.transactions.insert(transaction.id, transaction);
                    c
                })
            }
            Withdrawal => {
                old_client.withdrawal(transaction.amount).map(|c| {
                    self.transactions.insert(transaction.id, transaction);
                    c
                })
            }
            // FIXME: implement a state machine to check if operation can be carried out; e.g. no `dispute` on `withdrawal`, or no `resolve` if there was not even a `dispute`
            Dispute => {
                match self.transactions.get(&transaction.id) {
                    Some(p) => { old_client.dispute(p.amount) }
                    _ => { Err(ReferencedTransactionNonexistent) }
                }
            }
            Resolve => {
                match self.transactions.get(&transaction.id) {
                    Some(p) => { old_client.resolve(p.amount) }
                    _ => { Err(ReferencedTransactionNonexistent) }
                }
            }
            Chargeback => {
                match self.transactions.get(&transaction.id) {
                    Some(p) => { old_client.chargeback(p.amount) }
                    _ => { Err(ReferencedTransactionNonexistent) }
                }
            }
        }?;

        self.clients.insert(transaction.client_id, new_client);
        Ok(new_client)
    }
}

// only basic test coverage here; it's a lot easier to test complex functionality end-to-end, from `main.rs`
#[cfg(test)]
mod tests {
    use crate::{Client, Ledger, Transaction, TransactionKind};

    #[test]
    fn single_deposit() {
        let mut ledger = Ledger::new();
        ledger.mutate(Transaction { id: 0, client_id: 0, kind: TransactionKind::Deposit, amount: "12.5".parse().unwrap() }).expect("");
        let mut iter = ledger.iter();
        assert_eq!(iter.next().unwrap(), (&0u16, &Client { id: 0, available: "12.5".parse().unwrap(), held: "0".parse().unwrap(), locked: false }))
    }

    #[test]
    fn multi_deposit() {
        let mut ledger = Ledger::new();
        ledger.mutate(Transaction { id: 0, client_id: 0, kind: TransactionKind::Deposit, amount: "12.5".parse().unwrap() }).expect("");
        ledger.mutate(Transaction { id: 1, client_id: 0, kind: TransactionKind::Deposit, amount: "7.5".parse().unwrap() }).expect("");
        let mut iter = ledger.iter();
        assert_eq!(iter.next().unwrap(), (&0u16, &Client { id: 0, available: "20".parse().unwrap(), held: "0".parse().unwrap(), locked: false }))
    }

    #[test]
    fn single_withdraw() {
        let mut ledger = Ledger::new();
        ledger.mutate(Transaction { id: 0, client_id: 0, kind: TransactionKind::Withdrawal, amount: "12.5".parse().unwrap() }).expect_err("");
        let mut iter = ledger.iter();
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn multi_deposit_withdraw() {
        let mut ledger = Ledger::new();
        ledger.mutate(Transaction { id: 0, client_id: 0, kind: TransactionKind::Deposit, amount: "12.5".parse().unwrap() }).expect("");
        ledger.mutate(Transaction { id: 1, client_id: 0, kind: TransactionKind::Withdrawal, amount: "7.5".parse().unwrap() }).expect("");
        ledger.mutate(Transaction { id: 2, client_id: 0, kind: TransactionKind::Deposit, amount: "5".parse().unwrap() }).expect("");
        ledger.mutate(Transaction { id: 3, client_id: 0, kind: TransactionKind::Deposit, amount: "-5".parse().unwrap() }).expect_err("");
        let mut iter = ledger.iter();
        assert_eq!(iter.next().unwrap(), (&0u16, &Client { id: 0, available: "10".parse().unwrap(), held: "0".parse().unwrap(), locked: false }))
    }

    #[test]
    fn multi_client() {
        let mut ledger = Ledger::new();
        ledger.mutate(Transaction { id: 0, client_id: 5, kind: TransactionKind::Deposit, amount: "5".parse().unwrap() }).expect("");
        ledger.mutate(Transaction { id: 1, client_id: 10, kind: TransactionKind::Withdrawal, amount: "10".parse().unwrap() }).expect_err("");
        ledger.mutate(Transaction { id: 2, client_id: 5, kind: TransactionKind::Withdrawal, amount: "2".parse().unwrap() }).expect("");
        ledger.mutate(Transaction { id: 3, client_id: 3, kind: TransactionKind::Deposit, amount: "3".parse().unwrap() }).expect("");

        let mut result: Vec<&Client> = ledger.iter().map(|e| e.1).collect();
        result.sort_by(|a, b| a.id.cmp(&b.id));

        assert_eq!(*result[0], Client { id: 3, available: "3".parse().unwrap(), held: "0".parse().unwrap(), locked: false });
        assert_eq!(*result[1], Client { id: 5, available: "3".parse().unwrap(), held: "0".parse().unwrap(), locked: false });
    }
}