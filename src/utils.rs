use clap::{Arg, Command};
use std::error::Error;
use csv::Writer;
use crate::client::Client;
use dashmap::DashMap;
use std::io;
use rayon::prelude::*;
use std::time::Instant;
use log::{info};

pub fn parse_cli_arguments() -> String {
    let matches = Command::new("Santas_amex")
        .version("1.0")
        .about("Processes Santa's toy purchases from a CSV file")
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

pub fn setup_logger(log_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .chain(fern::log_file(log_file)?) // Log to a file
        .apply()?;
    Ok(())
}

pub fn write_clients_to_csv(client_map: &DashMap<u16, Client>) -> Result<(), Box<dyn Error>> {
    let start_time = Instant::now();
    let mut writer = Writer::from_writer(io::stdout());

    // Write the header row
    writer.write_record(&["client", " available", " held", " total", " locked"])?;

    // Write each client's data as a row
    for client_entry in client_map {
        let client = client_entry.value();
        writer.write_record(&[
            client.id().to_string(),
            format!(" {:.4}", client.available()),
            format!(" {:.4}", client.held()),
            format!(" {:.4}", client.total()),
            format!(" {}", client.locked().to_string()),
        ])?;
    }

    writer.flush()?;
    let elapsed_time = start_time.elapsed();
        info!(
            "write_clients_to_csv completed in {:.2?} seconds.",
            elapsed_time
        );
    Ok(())
}


/* NEW: Using multi-threading to increase speed of writing the output to csv. A cost of
        this function is the Dashmap must be converted into Vec which operates at O(n) and
        could become a slog if given a large enough dataset.
 */
// pub fn write_clients_to_csv(client_map: &DashMap<u16, Client>) -> Result<(), Box<dyn Error>> {
//     let start_time = Instant::now();
//     // Convert DashMap into a Vec for parallel processing. Rayon cannot use its .par_iter()
       // function on a dashmap
//     let clients: Vec<_> = client_map.iter().collect();
//     let mut writer = Writer::from_writer(io::stdout());
//     writer.write_record(&["client", "available", "held", "total", "locked"])?;
//
//     // Using par_iter to write the rows in chunks. This should open additional threads to complete
//     // the  task and increase speed
//     let formatted_rows: Vec<Vec<String>> = clients
//         .par_iter()
//         .map(|client_entry| {
//             let client = client_entry.value();
//             vec![
//                 client.id().to_string(),
//                 format!("{:.4}", client.available()),
//                 format!("{:.4}", client.held()),
//                 format!("{:.4}", client.total()),
//                 client.locked().to_string(),
//             ]
//         })
//         .collect();
//
//     for row in formatted_rows {
//         writer.write_record(&row)?;
//     }
//
//     writer.flush()?;
//     let elapsed_time = start_time.elapsed();
//     info!(
//         "write_clients_to_csv completed in {:.2?} seconds.",
//         elapsed_time
//     );
//     Ok(())
// }