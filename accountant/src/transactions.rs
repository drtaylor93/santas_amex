use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use serde::{Deserialize, Serialize};
use csv::{ReaderBuilder};
use crate::client::{Client};
use dashmap::DashMap;
use std::sync::LazyLock;


#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
    // Type is not a valid field for a struct, changed field name to transaction_type
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub client: u16,
    pub tx: u32,
    pub amount: f32,
}

// In my experience dashmap is better suited for concurrency. Will likely make the client hashmap
// a dashmap as well after further testing
static TRANSACTIONS_MAP: LazyLock<DashMap<u32, Transaction>> = LazyLock::new(|| DashMap::new());


pub fn process_transactions(input_file: &str) -> Result<(), Box<dyn Error>> {
    transaction_to_map(input_file)?;
    let mut client_map: HashMap<u16, Client> = HashMap::new();

    for transaction in TRANSACTIONS_MAP.iter() {
        // Try to find the client in the map
        let client_entry = if let Some(client) = client_map.get_mut(&transaction.client) {
            client
        } else {
            // If the client id is not in the map add the client id as the key
            // and the client as the value. Return the reference so we can
            // perform a transaction on it
            println!("Creating new client with ID: {}", transaction.client);
            client_map.insert(transaction.client, Client::new(transaction.client));
            client_map.get_mut(&transaction.client).unwrap()
        };


        match transaction.transaction_type.as_str() {
            "deposit" => {
                client_entry.deposit(transaction.amount);
                println!(
                    "Deposit of ${} successful for client {}",
                    transaction.amount, transaction.client
                );
            }
            "withdrawal" => {
                match client_entry.withdraw(transaction.amount) {
                    Ok(_) => println!(
                        "Withdrawal of ${} successful for client {}",
                        transaction.amount, transaction.client
                    ),
                    Err(error) => println!(
                        "Error withdrawing for client {}: {}",
                        transaction.client, error
                    ),
                }
            }
            // Any non recognized transactions will go here. I'll make transactions case insensitive
            // later
            _ => println!(
                "This type of transaction isn't available for Santa's Amex: {} for client {}",
                transaction.transaction_type, transaction.client
            ),
        }
    }

    //println!("Client List: {:?}", client_map);
    Ok(())
}


pub fn transaction_to_map(input_file: &str) -> Result<(), Box<dyn Error>> {
    let transaction_file = File::open(input_file)?;
    // Assumption: CSV file has a header (type, client, tx, amount)
    let mut transaction_extractor = ReaderBuilder::new()
        .trim(csv::Trim::All)
        .has_headers(true)
        .from_reader(transaction_file);

    for result in transaction_extractor.deserialize() {
        match result {
            Ok(record) => {
                let transaction: Transaction = record;
                TRANSACTIONS_MAP.insert(transaction.tx, transaction);
            }
            Err(err) => {
                eprintln!("Uh oh, there was an issue accessing your transactions. Please try again later: {}", err);
            }
        }
    }
    Ok(())
}