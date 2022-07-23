use std::collections::HashMap;
use std::ops::Deref;
use crate::amount::{Amount, ZERO};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionKind {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
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

    pub fn deposit(self, amount: Amount) -> Self {
        Client {
            available: self.available + amount,
            ..self
        }
    }

    pub fn withdrawal(self, amount: Amount) -> Self {
        if amount > self.available { return self; }
        Client {
            available: self.available - amount,
            ..self
        }
    }

    pub fn dispute(self, amount: Amount) -> Self {
        // FIXME: `available` can go negative, should add sanity check
        Client {
            available: self.available - amount,
            held: self.held + amount,
            ..self
        }
    }

    pub fn resolve(self, amount: Amount) -> Self {
        // FIXME: `held` can go negative, should add sanity check
        Client {
            available: self.available + amount,
            held: self.held - amount,
            ..self
        }
    }

    pub fn chargeback(self, amount: Amount) -> Self {
        // FIXME: `held` can go negative, should add sanity check
        Client {
            held: self.held - amount,
            ..self
        }
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

    pub fn mutate(&mut self, transaction: Transaction) {
        // sanity check: transaction amount is not negative
        if transaction.amount < *ZERO { return; }

        let old_client = match self.clients.get(&transaction.client_id) {
            None => { Client::new(transaction.client_id) }
            Some(x) => { x.clone() }
        };

        let new_client = match transaction.kind {
            TransactionKind::Deposit => {
                Some(old_client.deposit(transaction.amount))
            }
            TransactionKind::Withdrawal => {
                Some(old_client.withdrawal(transaction.amount))
            }
            TransactionKind::Dispute => {
                match self.transactions.get(&transaction.id) {
                    Some(p) => { Some(old_client.dispute(p.amount)) }
                    _ => { None }
                }
            }
            TransactionKind::Resolve => {
                match self.transactions.get(&transaction.id) {
                    Some(p) => { Some(old_client.resolve(p.amount)) }
                    _ => { None }
                }
            }
            TransactionKind::Chargeback => {
                match self.transactions.get(&transaction.id) {
                    Some(p) => { Some(old_client.chargeback(p.amount)) }
                    _ => { None }
                }
            }
        };

        if new_client.is_some() {
            self.clients.insert(transaction.client_id, new_client.unwrap());
        }
    }
}

// only basic test coverage here; it's a lot easier to test complex functionality end-to-end, from `main.rs`
#[cfg(test)]
mod tests {
    use crate::{Client, Ledger, Transaction, TransactionKind};

    #[test]
    fn single_deposit() {
        let mut ledger = Ledger::new();
        ledger.mutate(Transaction { id: 0, client_id: 0, kind: TransactionKind::Deposit, amount: "12.5".parse().unwrap() });
        let mut iter = ledger.iter();
        assert_eq!(iter.next().unwrap(), (&0u16, &Client { id: 0, available: "12.5".parse().unwrap(), held: "0".parse().unwrap(), locked: false }))
    }

    #[test]
    fn multi_deposit() {
        let mut ledger = Ledger::new();
        ledger.mutate(Transaction { id: 0, client_id: 0, kind: TransactionKind::Deposit, amount: "12.5".parse().unwrap() });
        ledger.mutate(Transaction { id: 1, client_id: 0, kind: TransactionKind::Deposit, amount: "7.5".parse().unwrap() });
        let mut iter = ledger.iter();
        assert_eq!(iter.next().unwrap(), (&0u16, &Client { id: 0, available: "20".parse().unwrap(), held: "0".parse().unwrap(), locked: false }))
    }

    #[test]
    fn single_withdraw() {
        let mut ledger = Ledger::new();
        ledger.mutate(Transaction { id: 0, client_id: 0, kind: TransactionKind::Withdrawal, amount: "12.5".parse().unwrap() });
        let mut iter = ledger.iter();
        assert_eq!(iter.next().unwrap(), (&0u16, &Client { id: 0, available: "0".parse().unwrap(), held: "0".parse().unwrap(), locked: false }))
    }

    #[test]
    fn multi_deposit_withdraw() {
        let mut ledger = Ledger::new();
        ledger.mutate(Transaction { id: 0, client_id: 0, kind: TransactionKind::Deposit, amount: "12.5".parse().unwrap() });
        ledger.mutate(Transaction { id: 1, client_id: 0, kind: TransactionKind::Withdrawal, amount: "7.5".parse().unwrap() });
        ledger.mutate(Transaction { id: 2, client_id: 0, kind: TransactionKind::Deposit, amount: "5".parse().unwrap() });
        ledger.mutate(Transaction { id: 3, client_id: 0, kind: TransactionKind::Deposit, amount: "-5".parse().unwrap() });
        let mut iter = ledger.iter();
        assert_eq!(iter.next().unwrap(), (&0u16, &Client { id: 0, available: "10".parse().unwrap(), held: "0".parse().unwrap(), locked: false }))
    }

    #[test]
    fn multi_client() {
        let mut ledger = Ledger::new();
        ledger.mutate(Transaction { id: 0, client_id: 5, kind: TransactionKind::Deposit, amount: "5".parse().unwrap() });
        ledger.mutate(Transaction { id: 1, client_id: 10, kind: TransactionKind::Withdrawal, amount: "10".parse().unwrap() });
        ledger.mutate(Transaction { id: 2, client_id: 5, kind: TransactionKind::Withdrawal, amount: "2".parse().unwrap() });
        ledger.mutate(Transaction { id: 3, client_id: 3, kind: TransactionKind::Deposit, amount: "3".parse().unwrap() });

        let mut result: Vec<&Client> = ledger.iter().map(|e| e.1).collect();
        result.sort_by(|a, b| a.id.cmp(&b.id));

        assert_eq!(*result[0], Client { id: 3, available: "3".parse().unwrap(), held: "0".parse().unwrap(), locked: false });
        assert_eq!(*result[1], Client { id: 5, available: "3".parse().unwrap(), held: "0".parse().unwrap(), locked: false });
        assert_eq!(*result[2], Client { id: 10, available: "0".parse().unwrap(), held: "0".parse().unwrap(), locked: false });
    }
}