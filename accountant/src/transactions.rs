use std::error::Error;
use std::fs::File;
use serde::{Deserialize};
use csv::{ReaderBuilder};
use crate::client::{Client};
use dashmap::DashMap;
use std::sync::LazyLock;

#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: String,
    pub client: u16,
    pub tx: u32,
    pub amount: Option<f32>,
}

static TRANSACTIONS_MAP: LazyLock<DashMap<u32, Transaction>> = LazyLock::new(|| DashMap::new());

// TODO: Add a logger to separate debug statements from final output

pub fn process_transactions(input_file: &str) -> Result<(), Box<dyn Error>> {
    // Client Map keeps a copy of all client data in a map for future reference
    let client_map: DashMap<u16, Client> = DashMap::new();
    let transaction_file = File::open(input_file)?;

    let mut transaction_reader = ReaderBuilder::new()
        .flexible(true)
        .trim(csv::Trim::All)
        .has_headers(true)
        .from_reader(transaction_file);

    for row in transaction_reader.records() {
        match row {
            // If the CSV row is valid add it to the client map
            // If the client in the transaction does not exist create a new client in the map
            Ok(record) => {
                if let Some(transaction) = parse_transaction(&record) {
                    let mut client_entry = client_map
                        .entry(transaction.client)
                        .or_insert_with(|| {
                            println!("Creating new client: {}", transaction.client);
                            Client::new(transaction.client)
                        });

                    // Convert the transaction type to lowercase for case-insensitive matching
                    match transaction.transaction_type.to_lowercase().as_str() {
                        "deposit" => {
                            client_entry.deposit(transaction.amount);
                            println!(
                                "Deposit of ${:?} successful {}",
                                transaction.amount, transaction.client
                            );
                        }
                        "withdrawal" => {
                            match client_entry.withdraw(transaction.amount) {
                                Ok(_) => println!(
                                    "Withdrawal of ${:?} successful {}",
                                    transaction.amount, transaction.client
                                ),
                                Err(error) => println!(
                                    "Sorry Santa, you've exceeded your limit {}: {}",
                                    transaction.client, error
                                ),
                            }
                        }
                        "dispute" => {
                            if let Some(disputed_transaction) = TRANSACTIONS_MAP.get(&transaction.tx) {
                                if let Some(disputed_amount) = disputed_transaction.value().amount {
                                    _ = client_entry.dispute(Some(disputed_amount));
                                    println!(
                                        "Dispute processed for client {}: Held += {:?}, Available -= {:?}",
                                        transaction.client, disputed_amount, disputed_amount
                                    );
                                } else {
                                    println!(
                                        "Disputed transaction has no amount for Transaction ID {}.",
                                        transaction.tx
                                    );
                                }
                            } else {
                                println!(
                                    "Transaction with ID {} not found for dispute.",
                                    transaction.tx
                                );
                            }
                        }
                        _ => println!(
                            "Unsupported transaction type: {:?} for client {:?}",
                            transaction.transaction_type, transaction.client
                        ),
                    }

                    TRANSACTIONS_MAP.insert(transaction.tx, transaction);
                } else {
                    eprintln!("Skipping invalid transaction: {:?}", record);
                }
            }
            Err(err) => {
                eprintln!(
                    "Error reading transactions from the CSV file: {}",
                    err
                );
            }
        }
    }

    println!("\nClient Details:");
    for client_entry in &client_map {
        let client = client_entry.value();
        println!(
            "Client ID: {}, Available: {:.2}, Held: {:.2}, Total: {:.2}",
            client_entry.key(), client.available, client.held, client.total
        );
    }

    Ok(())
}


// Disputes, Chargebacks, Resolves may not have an amount provided.
// Using .flexible() for the csv crate does not seem to allow for varying rows
// so extracting each value bit by bit from the row seems to be the best way to handle this at the
// moment.
fn parse_transaction(record: &csv::StringRecord) -> Option<Transaction> {
    // Ensure we get the first field properly split
    let transaction_type = record.get(0)?.to_string();
    let client = record.get(1)?.parse::<u16>().ok()?;
    let tx = record.get(2)?.parse::<u32>().ok()?;

    // If the transaction type is "dispute" and there's no amount, set amount to None
    let amount = if transaction_type == "dispute" && record.get(3).is_none() {
        None
    } else {
        record.get(3).and_then(|s| s.parse::<f32>().ok())
    };

    Some(Transaction {
        transaction_type,
        client,
        tx,
        amount,
    })
}