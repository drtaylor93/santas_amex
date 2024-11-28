use serde::{Serialize};

// Ya, I know I need getters and setters here too instead of making everything public
// Chill, I'll refactor it once basic transaction functionality is complete
#[derive(Debug, Serialize)]
pub struct Client {
    pub id: u16,
    pub available: f32,
    pub held: f32,
    pub total: f32,
    pub locked: bool,
}

// Client ID needs to be thread safe because otherwise two clients may end up
// with duplicate id and instead of going into each respective key in the map
// they'll actually overwrite each other

//I will fix this issue later. Most focused on ensuring basic deposit, withdraw works
// Also shouldn't be public #rulebreaker


// Constructor for client
impl Client {
    pub fn new(client_id : u16) -> Self {
        Self {
            id: client_id,
            available: 0.0,
            held: 0.0,
            total: 0.0,
            locked: false,
        }
    }

    pub fn deposit(&mut self, amount: f32) {
        if amount > 0.0 {
            self.available += amount;
            self.total += amount;
            self.held += amount;
            //println!("Deposited ${:.2} to Client {}. New available balance: ${:.2}", amount, self.id, self.available);
        } else {
            println!("Cannot deposit a negative amount of money: ${:.2}", amount);
        }
    }

    pub fn withdraw(&mut self, amount: f32) -> Result<(), &str> {
        if self.available >= amount {
            self.available -= amount;
            println!("Withdrawal of ${} successful. Your new balance is: ${}", amount, self.available);
            Ok(()) // No error, successful operation
        } else {
            Err("Insufficient funds") // Return error message if funds are insufficient
        }
    }
}