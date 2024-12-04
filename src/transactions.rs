use std::error::Error;
use std::fs::File;
use serde::{Deserialize};
use csv::{ReaderBuilder};
use crate::client::{Client};
use dashmap::DashMap;
use std::sync::LazyLock;
use log::{error, info, warn};

/*
    TODO: Create a constructor, getters, and setters for Transactions to block off direct access to
          Transaction fields. For general usage it was not necessary, but the unit tests need
          to be access without the presence of a csv file
 */
#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub(crate) transaction_type: String,
    pub(crate) client: u16,
    pub(crate) tx: u32,
    pub(crate) amount: Option<f32>,
}

/*
    Using a Dashmap(Rust Hashmap with built-in handling of concurrency) to store all transactions
    from csv file. If specifically using hashmap was required I would mutex lock each transaction
    and client as they were being modified to prevent race conditions when multi-threading.
 */

static TRANSACTIONS_MAP: LazyLock<DashMap<u32, Transaction>> = LazyLock::new(|| DashMap::new());

pub fn process_transactions(input_file: &str) -> Result<DashMap<u16, Client>, Box<dyn Error>> {
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
                            info!("Creating new client: {}", transaction.client);
                            Client::new(transaction.client)
                        });

                    // Convert the transaction type to lowercase for case-insensitive matching
                    match transaction.transaction_type.to_lowercase().as_str() {
                        "deposit" => process_deposit(&mut client_entry, &transaction),
                        "withdrawal" => process_withdrawal(&mut client_entry, &transaction),
                        "dispute" => process_dispute(&mut client_entry, &transaction),
                        "resolve" => process_resolve(&mut client_entry, &transaction),
                        "chargeback" => process_chargeback(&mut client_entry, &transaction),

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
    Ok(client_map)
}


// Disputes, Chargebacks, Resolves may not have an amount provided.
// Using .flexible() for the csv crate does not seem to allow for varying rows
// so extracting each value bit by bit from the row seems to be the best way to handle this at the
// moment. Will look to improve in future iteration.
fn parse_transaction(record: &csv::StringRecord) -> Option<Transaction> {
    let transaction_type = record.get(0)?.to_string();
    let client = record.get(1)?.parse::<u16>().ok()?;
    let tx = record.get(2)?.parse::<u32>().ok()?;

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

/*
Description: Begins the deposit process, first checking if the account is locked. Transactions
             performed on a locked account will be ignored. Only positive value will be accepted.
Parameters:
    client_entry: &mut Mutable reference to the instance of the client
    transaction: &Transaction  Reference to Transaction struct
*/
pub(crate) fn process_deposit(client_entry: &mut Client, transaction: &Transaction) {
    // If account is locked skip the transaction
    if client_entry.is_account_locked(transaction.tx) {
        return;
    }
    // Check if the transaction has a valid amount
    if let Some(amount) = transaction.amount {
        if amount > 0.0 {
            client_entry.deposit(Some(amount));
            info!(
                "Deposit of ${:.2} successful for client {}. New available balance: ${:.2}",
                amount, client_entry.id(), client_entry.available()
            );
        } else {
            warn!("Cannot deposit a zero or negative amount of money");
        }
    } else {
        warn!("Invalid deposit amount for client {}", client_entry.id());
    }
}

/*
Description: Begins the withdrawal process, checks for locked account. Only positive values will
             be accepted.
Parameters:
    client_entry: &mut Mutable reference to the instance of the client
    transaction: &Transaction  Reference to Transaction struct
*/
pub(crate) fn process_withdrawal(client_entry: &mut Client, transaction: &Transaction) {
    if client_entry.is_account_locked(transaction.tx) {
        return;
    }
    if let Some(amount) = transaction.amount {
        if amount > 0.0 {
            let client_id = client_entry.id();
            match client_entry.withdraw(Some(amount)) {
                Ok(_) => info!(
                    "Withdrawal of ${:.4} successful for client {}. New available balance: ${:.4}",
                    amount, client_id, client_entry.available()
                ),
                Err(error) => error!(
                    "Sorry Santa, you've exceeded your limit for this client {}: {}",
                    client_id, error
                ),
            }
        } else {
            warn!("Cannot withdraw a non-positive amount");
        }
    } else {
        warn!("Invalid withdrawal amount for client {}", client_entry.id());
    }
}

/*
Description: Begins the dispute process, checks for locked account. Proceeds to check if transaction
             in the dispute exists in the transaction_map.
Parameters:
    client_entry: &mut Mutable reference to the instance of the client
    transaction: &Transaction  Reference to Transaction struct
*/
pub(crate) fn process_dispute(client_entry: &mut Client, transaction: &Transaction) {
    if client_entry.is_account_locked(transaction.tx) {
        return;
    }
    if let Some(disputed_transaction) = TRANSACTIONS_MAP.get(&transaction.tx) {
        if let Some(disputed_amount) = disputed_transaction.value().amount {
            match client_entry.dispute(transaction.tx, Some(disputed_amount)) {
                Ok(_) => {
                    info!(
                        "Dispute successful for client {}: ${:.4} moved to Held, Available balance is now ${:.4}.",
                        transaction.client, disputed_amount, client_entry.available()
                    );
                }
                Err(err) => {
                    error!(
                        "Error processing dispute for client {}: {}",
                        transaction.client, err
                    );
                }
            }
        } else {
            warn!(
                "Error: Transaction ID {} has no amount to dispute.",
                transaction.tx
            );
        }
    } else {
        warn!(
            "Error: Transaction ID {} not found for dispute.",
            transaction.tx
        );
    }
}

/*
Description: Used on transactions already under dispute, checks for locked account. Proceeds to
             check if transaction id is in the clients disputed_transaction hashmap.
Parameters:
    client_entry: &mut Mutable reference to the instance of the client
    transaction: &Transaction  Reference to Transaction struct
*/
pub(crate) fn process_resolve(client_entry: &mut Client, transaction: &Transaction) {
    if client_entry.locked() {
        log::warn!(
            "Skipping {} for transaction {}: Account {} is locked.",
            transaction.transaction_type,
            transaction.tx,
            client_entry.id()
        );
        return;
    }
    if client_entry.disputed_transactions().contains_key(&transaction.tx) {
        match client_entry.resolve(transaction.tx) {
            Ok(_) => {
                info!(
                    "Transaction {} was resolved {}. Held funds are now available.",
                    transaction.tx, transaction.client
                );
            }
            Err(err) => {
                error!(
                    "Failed to resolve transaction {} for client {}: {}",
                    transaction.tx, transaction.client, err
                );
            }
        }
    } else {
        warn!(
            "Transaction {} not found in disputed transactions for client {}. Unable to resolve.",
            transaction.tx, transaction.client
        );
    }
}

/*
Description: Used on transactions already under dispute, checks for locked account. Proceeds to
             check if transaction id is in the clients disputed_transaction hashmap.
Parameters:
    client_entry: &mut Mutable reference to the instance of the client
    transaction: &Transaction  Reference to Transaction struct
*/
pub(crate) fn process_chargeback(client_entry: &mut Client, transaction: &Transaction) {
    if client_entry.is_account_locked(transaction.tx) {
        return;
    }
    if client_entry.disputed_transactions().contains_key(&transaction.tx) {
        match client_entry.chargeback(transaction.tx) {
            Ok(_) => {
                info!(
                    "Chargeback processed successfully for client {}. Transaction {}: ${:?} removed from Held and Total funds.",
                    transaction.client, transaction.tx, transaction.amount
                );
            }
            Err(err) => {
                error!(
                    "Failed to process chargeback for client {}. Transaction {}: {}",
                    transaction.client, transaction.tx, err
                );
            }
        }
    } else {
        warn!(
            "Transaction {} not found in disputed transactions for client {}. Unable to resolve.",
            transaction.tx, transaction.client
        );
    }
}