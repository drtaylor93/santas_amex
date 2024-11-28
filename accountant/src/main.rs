mod transactions;
mod utils;
mod client;
use transactions::process_transactions;
use utils::parse_cli_arguments;
use std::process;

fn main() {
    let input_file = parse_cli_arguments();
    match process_transactions(&input_file) {
        Ok(_) => println!("Successfully processed transactions."),
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    }
}



