mod transactions;
pub mod utils;
mod client;
mod client_tests;

use transactions::process_transactions;
use utils::{parse_cli_arguments, write_clients_to_csv, setup_logger};
use std::process;
use log::{info, error};

fn main() {
    let log_file = "transactions.log";
    if let Err(err) = setup_logger(log_file) {
        error!("Failed to initialize logger: {}", err);
        std::process::exit(1);
    }

    info!("Transactions initialized!");
    let input_file = parse_cli_arguments();
    match process_transactions(&input_file) {
        Ok(client_map) => {
            if let Err(err) = write_clients_to_csv(&client_map) {
                error!("Error writing to CSV: {}", err);
                process::exit(1);
            }
        }
        Err(err) => {
            error!("Error processing transactions: {}", err);
            process::exit(1);
        }
    }
}



