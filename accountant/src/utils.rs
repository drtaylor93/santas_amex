use clap::{Arg, Command};

pub fn parse_cli_arguments() -> String {
    let matches = Command::new("Santas_amex")
        .version("1.0")
        .about("Processes Santa's toy transactions from a CSV file")
        .arg(
            Arg::new("input")
                .help("Path to the input CSV file")
                .required(true)
                .index(1),
        )
        .get_matches();

    matches
        .get_one::<String>("input")
        .expect("CSV file is needed to check Santa's transactions")
        .clone()
}