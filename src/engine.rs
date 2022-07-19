use crate::amount::Amount;

pub struct Transaction {
    id: u64,
    client_id: u64,
    amount: Amount,
}

pub struct Client {
    id: u64,
    balance: Amount,
}

// impl Transaction {
//     pub fn new(id: u64, client_id: u64, amount_fx4: u64) -> Transaction {
//         Transaction { id, client_id, amount_fx4 }
//     }
// }
//
// impl Client {
//     pub fn new(id: u64, balance_fx4: u64) -> Client {
//         Client { id, balance_fx4 }
//     }
// }

