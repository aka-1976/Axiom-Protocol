use std::collections::HashMap;
use crate::transaction::{Transaction, Address};

#[derive(Clone)]
pub struct State {
    pub balances: HashMap<Address, u64>,
    pub total_issued: u64,
}

impl State {
    pub fn new() -> Self {
        Self {
            balances: HashMap::new(),
            total_issued: 0,
        }
    }

    pub fn balance(&self, addr: &Address) -> u64 {
        *self.balances.get(addr).unwrap_or(&0)
    }

    pub fn credit(&mut self, addr: Address, amount: u64) {
        let bal = self.balance(&addr);
        self.balances.insert(addr, bal + amount);
    }

    pub fn apply_tx(&mut self, tx: &Transaction) -> bool {
        let sender_bal = self.balance(&tx.from);
        let cost = tx.amount + tx.fee;

        if sender_bal < cost {
            return false;
        }

        self.balances.insert(tx.from, sender_bal - cost);
        self.credit(tx.to, tx.amount);
        true
    }
}
