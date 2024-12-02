use serde::{Serialize};
use std::collections::HashMap;
use log::{info, warn};

#[derive(Debug, Serialize)]
pub struct Client {
    id: u16,
    available: f32,
    held: f32,
    total: f32,
    locked: bool,
    disputed_transactions: HashMap<u32, f32>,
}

impl Client {
    pub fn new(client_id: u16) -> Self {
        Self {
            id: client_id,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
            // Each client instance will keep track of its disputes. Reduces the number of times the
            // master transaction map has to be accessed
            // Key: tx_id, Value: tx amount
            disputed_transactions: HashMap::new(),
        }
    }

    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn available(&self) -> f32 {
        self.available
    }

    pub fn held(&self) -> f32 {
        self.held
    }


    pub fn total(&self) -> f32 {
        self.total
    }

    pub fn locked(&self) -> bool {
        self.locked
    }

    pub fn disputed_transactions(&self) -> &HashMap<u32, f32> {
        &self.disputed_transactions
    }

    /*
    Description: Modifies client instance by adding to available and total funds in their account
    Parameters:
        amount: Option<f32> The amount to be deposited into the account
    */
    pub fn deposit(&mut self, amount: Option<f32>) {
        match amount {
            Some(value) if value > 0.0 => {
                self.available += value;
                self.total += value;
                info!(
                    "Deposited ${:.2} to Client {}. New available balance: ${:.2}",
                    value, self.id, self.available
                );
            }
            Some(value) => {
                warn!("Cannot deposit a negative amount of money: ${:.2}", value);
            }
            None => {
                warn!("Deposit failed: Amount is None");
            }
        }
    }

    /*
    Description:
        Modifies client instance by reducing the available and total funds in their account.
        If the withdrawal amount is greater than the available funds, result in error
    Parameters:
        amount: Option<f32> The amount to be deposited into the account
    */
    pub fn withdraw(&mut self, amount: Option<f32>) -> Result<(), &str> {
        match amount {
            Some(value) if value > 0.0 => {
                if self.available >= value {
                    self.available -= value;
                    self.total -= value;
                    info!(
                        "Withdrawal of ${:.2} successful. Your new balance is: ${:.2}",
                        value, self.available
                    );
                    Ok(())
                } else {
                    Err("Insufficient funds")
                }
            }
            Some(value) => {
                println!("Cannot withdraw a negative or zero amount: ${:.2}", value);
                Err("Invalid withdrawal amount")
            }
            None => {
                warn!("Withdrawal failed: None value found");
                Err("No amount provided for withdrawal")
            }
        }
    }


    /*
    Description: Disputes a transaction, adding the transaction in question to the clients hashmap
    Parameters:
        tx_id: u32 The transaction id of the tx in question
        amount: Option<f32> The amount to be deposited into the account
    NOTE:
        Currently this function only disputes withdrawals, but there could be a case where a
        deposit should be disputed. Will adjust in the future to handle this

    */
    pub fn dispute(&mut self, tx_id: u32, amount: Option<f32>) -> Result<(), &str> {
        match amount {
            Some(value) if value > 0.0 => {
                if self.available >= value {
                    self.available -= value;
                    self.held += value;
                    self.disputed_transactions.insert(tx_id, value);
                    info!(
                        "Dispute initiated for amount ${:.4} on transaction {}. Held: ${:.4}, Available: ${:.4}",
                        value, tx_id, self.held, self.available
                    );
                    Ok(())
                } else {
                    Err("Insufficient available funds for dispute")
                }
            }
            Some(_) => Err("Invalid dispute amount"),
            None => Err("No amount provided for dispute"),
        }
    }

    /*
    Description:
        Undoes a dispute by removing it from the clients map. Held funds are transferred to the
        available field
    Parameters:
        tx_id: u32 The id of the transaction being disputed
    */
    pub fn resolve(&mut self, tx_id: u32) -> Result<(), &str> {
        if let Some(&amount) = self.disputed_transactions.get(&tx_id) {
            if self.held >= amount {
                self.held -= amount;
                self.available += amount;
                self.disputed_transactions.remove(&tx_id); // Remove the resolved transaction
                info!(
                    "Resolved dispute for transaction {}: Held -= {:.2}, Available += {:.2}",
                    tx_id, amount, amount
                );
                Ok(())
            } else {
                Err("Insufficient held funds to resolve dispute")
            }
        } else {
            Err("Transaction not found in disputed transactions")
        }
    }

    pub fn chargeback(&mut self, tx_id: u32) -> Result<(), &str> {
        if let Some(&disputed_amount) = self.disputed_transactions.get(&tx_id) {
            if self.held >= disputed_amount {
                self.held -= disputed_amount;
                self.total -= disputed_amount;
                self.disputed_transactions.remove(&tx_id);
                self.locked = true;

                info!(
                    "Chargeback processed for transaction {}: Held -= {:.4}, Total -= {:.4}. Account is locked.",
                    tx_id, disputed_amount, disputed_amount
                );
                Ok(())
            } else {
                Err("Insufficient held funds for chargeback")
            }
        } else {
            Err("Transaction not found in disputed transactions for chargeback")
        }
    }
}