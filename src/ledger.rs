use std::collections::HashMap;
use crate::amount::Amount;

#[derive(Debug)]
pub enum TransactionKind {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug)]
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

#[derive(Debug, Default)]
pub struct Client {
    id: u16,
    available: Amount,
    held: Amount,
    locked: bool,
}

impl Client {
    pub fn new(id: u16) -> Client {
        Client {
            id,
            ..Default::default()
        }
    }

    pub fn deposit(mut self, amount: Amount) -> Self {
        Client {
            available: self.available + amount,
            ..self
        }
    }

    pub fn withdrawal(mut self, amount: Amount) -> Self {
        if amount > self.available { return self; }
        Client {
            available: self.available - amount,
            ..self
        }
    }

    pub fn dispute(mut self, amount: Amount) -> Self {
        // FIXME: `available` can go negative, should add sanity check
        Client {
            available: self.available - amount,
            held: self.held + amount,
            ..self
        }
    }

    pub fn resolve(mut self, amount: Amount) -> Self {
        // FIXME: `held` can go negative, should add sanity check
        Client {
            available: self.available + amount,
            held: self.held - amount,
            ..self
        }
    }

    pub fn chargeback(mut self, amount: Amount) -> Self {
        // FIXME: `held` can go negative, should add sanity check
        Client {
            held: self.held - amount,
            ..self
        }
    }
}

pub struct Ledger {
    clients: HashMap<u16, Client>,
    transactions: HashMap<u64, Transaction>,
}

impl Ledger {
    pub fn add(mut self, transaction: Transaction) -> Self {
        // let client = self.clients.get_mut(&transaction.client_id).unwrap_or_else(|| &mut Client::new(transaction.client_id));
        self
    }
}