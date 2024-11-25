use std::env;
use std::error::Error;
use std::fs::File;
use std::process;
use csv::{ReaderBuilder, Writer};
use serde::{Deserialize, Serialize};
use clap::{Arg, Command};


// At this stage I am just trying to make sure I can read from a csv and write to
// another CSV

//TODO: Refactor read and writing into separate fns
fn read_transaction(input_file: &str) -> Result<(), Box<dyn Error>> {
    let transaction_file = File::open(input_file)?;
    // Assumption: CSV file has a header(type, client, tx, amount)
    // If this is untrue, change .has_headers to false
    let mut csv_parser = ReaderBuilder::new()
        .trim(csv::Trim::All)
        .has_headers(true)
        .from_reader(transaction_file);



    for result in csv_parser.deserialize() {
        let transaction: Transaction = result?;
        println!("{:?}", transaction);
    }
    Ok(())
}


#[derive(Debug, Deserialize, Serialize)]
struct Transaction {
    // Type is not a valid field for a struct, changed fieldname to transaction_type
    #[serde(rename = "type")]
    transaction_type: String,
    client: u16,
    tx: u32,
    amount: f32,
}


fn main() {
    let matches = Command::new("Santas_amex")
        .version("1.0")
        .about("Processes Santa's toy transactions from a CSV file")
        .arg(
            Arg::new("input")
                .help("Path to the input CSV file")
                .required(true)
                .index(1), // First positional argument
        )
        .get_matches();

    let input_file = matches
        .get_one::<String>("input")
        .expect("CSV file is needed to check Santa's transactions");

    if let Err(err) = read_transaction(input_file) {
        eprintln!("Not a merry Christmas: {}", err);
        process::exit(1);
    }
}
