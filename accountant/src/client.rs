use serde::{Serialize};

#[derive(Debug, Serialize)]
pub struct Client {
    pub id: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
}

impl Client {
    pub fn new(client_id: u16) -> Self {
        Self {
            id: client_id,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }

    pub fn deposit(&mut self, amount: Option<f32>) {
        match amount {
            Some(value) if value > 0.0 => {
                self.available += value;
                self.total += value;
                self.held += value;
                println!(
                    "Deposited ${:.2} to Client {}. New available balance: ${:.2}",
                    value, self.id, self.available
                );
            }
            Some(value) => {
                println!("Cannot deposit a negative amount of money: ${:.2}", value);
            }
            None => {
                println!("Deposit failed: Amount is None");
            }
        }
    }

    pub fn withdraw(&mut self, amount: Option<f32>) -> Result<(), &str> {
        match amount {
            Some(value) if value > 0.0 => {
                if self.available >= value {
                    self.available -= value;
                    println!(
                        "Withdrawal of ${:.2} successful. Your new balance is: ${:.2}",
                        value, self.available
                    );
                    Ok(())
                } else {
                    Err("Insufficient funds") // Return error if funds are insufficient
                }
            }
            Some(value) => {
                println!("Cannot withdraw a negative or zero amount: ${:.2}", value);
                Err("Invalid withdrawal amount")
            }
            None => {
                println!("Withdrawal failed: None value found");
                Err("No amount provided for withdrawal")
            }
        }
    }


    // Dispute logic still has a couple of holes in it, but it functions as expected
    // Return to this if time permits
    pub fn dispute(&mut self, amount: Option<f32>) -> Result<(), &str> {
        match amount {
            Some(value) if value > 0.0 => {
                if self.available >= value {
                    self.available -= value;
                    self.held += value;
                    println!(
                        "Dispute initiated for amount ${:.4}. Held: ${:.4}, Available: ${:.4}",
                        value, self.held, self.available
                    );
                    Ok(())
                } else {
                    Err("Insufficient available funds for dispute")
                }
            }
            Some(value) => {
                // Handle negative or zero values
                println!("Cannot dispute a negative or zero amount: ${:.2}", value);
                Err("Invalid dispute amount")
            }
            None => {
                println!("Santa, you need to specify an amount to dispute");
                Err("The dispute does not have an amount")
            }
        }
    }
}

