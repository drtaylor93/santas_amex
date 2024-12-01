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
    transaction_type: String,
    client: u16,
    tx: u32,
    amount: Option<f32>,
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
                        "deposit" => process_deposit(&mut client_entry, &transaction),
                        "withdrawal" => process_withdrawal(&mut client_entry, &transaction),
                        "dispute" => process_dispute(&mut client_entry, &transaction),
                        "resolve" => process_resolve(&mut client_entry, &transaction),

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
            client_entry.key(), client.available(), client.held(), client.total()
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


fn process_deposit(client_entry: &mut Client, transaction: &Transaction) {
    // Check if the transaction has a valid amount
    if let Some(amount) = transaction.amount {
        // Ensure the amount is positive
        if amount > 0.0 {
            client_entry.deposit(Some(amount));
            println!(
                "Deposit of ${:.2} successful for client {}. New available balance: ${:.2}",
                amount, client_entry.id(), client_entry.available()
            );
        } else {
            println!("Cannot deposit negative amount of money, use a withdrawal: ${:.2}", amount);
        }
    } else {
        println!("Invalid deposit amount for client {}", client_entry.id());
    }
}

fn process_withdrawal(client_entry: &mut Client, transaction: &Transaction) {
    if let Some(amount) = transaction.amount {
        if amount > 0.0 {
            let client_id = client_entry.id();
            match client_entry.withdraw(Some(amount)) {
                Ok(_) => println!(
                    "Withdrawal of ${:.2} successful for client {}. New available balance: ${:.2}",
                    amount, client_id, client_entry.available()
                ),
                Err(error) => println!(
                    "Sorry Santa, you've exceeded your limit for client {}: {}",
                    client_id, error
                ),
            }
        } else {
            println!("Cannot withdraw a non-positive amount: ${:.2}", amount);
        }
    } else {
        println!("Invalid withdrawal amount for client {}", client_entry.id());
    }
}

fn process_dispute(client_entry: &mut Client, transaction: &Transaction) {
    if let Some(disputed_transaction) = TRANSACTIONS_MAP.get(&transaction.tx) {
        if let Some(disputed_amount) = disputed_transaction.value().amount {
            match client_entry.dispute(transaction.tx, Some(disputed_amount)) {
                Ok(_) => {
                    println!(
                        "Dispute successful for client {}: ${:.2} moved to Held, Available balance is now ${:.2}.",
                        transaction.client, disputed_amount, client_entry.available()
                    );
                }
                Err(err) => {
                    println!(
                        "Error processing dispute for client {}: {}",
                        transaction.client, err
                    );
                }
            }
        } else {
            println!(
                "Error: Transaction ID {} has no amount to dispute.",
                transaction.tx
            );
        }
    } else {
        println!(
            "Error: Transaction ID {} not found for dispute.",
            transaction.tx
        );
    }
}

fn process_resolve(client_entry: &mut Client, transaction: &Transaction) {
    // Check if the transaction exists in the disputed transactions map
    if client_entry.disputed_transactions().contains_key(&transaction.tx) {
        // Try to resolve the dispute
        match client_entry.resolve(transaction.tx) {
            Ok(_) => {
                println!(
                    "Transaction {} was resolved {}. Held funds are now available.",
                    transaction.tx, transaction.client
                );
            }
            Err(err) => {
                println!(
                    "Failed to resolve transaction {} for client {}: {}",
                    transaction.tx, transaction.client, err
                );
            }
        }
    } else {
        println!(
            "Transaction {} not found in disputed transactions for client {}. Unable to resolve.",
            transaction.tx, transaction.client
        );
    }
}



