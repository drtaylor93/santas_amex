use crate::client::Client;
use crate::transactions::*;
use test_case::test_case;
use std::sync::Once;
use fern::Dispatch;
use log::{info};

/*
    Suppressing warnings in test regarding unused variables, dead_code, etc. as they are just
    noise. Rust cannot detect that these will be called by cargo test, and it distracts from
    actual issues that could be occurring.
 */
#[allow(dead_code)]
static INIT: Once = Once::new();

/*
    Configuring logger to print to a separate file than the standard output file so we
    don't confuse test output with production output.
 */
#[allow(dead_code)]
fn logger(log_file: &str) {
    INIT.call_once(|| {
        Dispatch::new()
            .format(|out, message, record| {
                out.finish(format_args!(
                    "[{}] [{}] {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    message
                ))
            })
            .level(log::LevelFilter::Info)
            .chain(fern::log_file(log_file).expect("Failed to open log file"))
            .apply()
            .expect("Failed to initialize logger");
    });
}

/* Using test-case crate for unit tests for client functions
   Each test case is enumerated per unit test
   Each line that starts with #[test_case] is calling the following function, but with
   varying parameters being passed. The last parameter is the expected log output.
   In the future more robust errors and checking will be added but for now this gets
   the job done.

   I.   Account is locked
   II.  Valid deposit amount
   III. Invalid deposit of $0
   V.   Valid deposit 4 places past decimal
 */
#[test_case(1, 100.0, true, "Skipping transaction 1: Account 1 is locked.")]
#[test_case(1, 100.0, false, "Deposit of $100.00 successful for client 1.")]
#[test_case(1, 0.0, false, "Cannot deposit a zero or negative amount of money")]
#[test_case(1, 100.1234, false, "Deposit of $100.1234 successful for client 1.")]
#[allow(dead_code, unused_variables, unused_mut)]
fn test_process_deposit(tx: u32, amount: f32, locked: bool, expected_log: &str) {
    logger("client_test.log");
    // TODO: Tried to make the log skip a line when a new test is run for better readability but
    //       it's not writing to the file the way I want :(. Adjust fern settings to fix this.
    info!("\nTest: test_process_deposit");
    let mut client = Client::new(1);
    client.set_locked(locked);
    let transaction = Transaction {
        transaction_type: "deposit".to_string(),
        client: 1,
        tx,
        amount: Some(amount),
    };

    process_deposit(&mut client, &transaction);

    if locked {
        assert_eq!(client.available(), 0.0);
    } else {
        assert_eq!(client.available(), amount);
    }
}

/*
   I.   Account is locked
   II.  Valid withdrawal amount
   III. Invalid negative withdrawal
   V.   Valid withdrawal 4 places past decimal
 */
#[test_case(1, 50.0, 100.0, true, "Skipping transaction 1: Account 1 is locked.")]
#[test_case(1, 50.0, 100.0, false, "Withdrawal of $50.00 successful for client 1.")]
#[test_case(1, 0.0, 50.0, false, "Cannot withdraw a non-positive amount")]
#[test_case(1, 50.1234, 100.0, false, "Withdrawal of $50.1234 successful for client 1.")]
#[allow(dead_code, unused_variables, unused_mut)]
fn test_process_withdrawal(tx: u32, withdrawal_amount: f32, initial_balance: f32, locked: bool, expected_log: &str) {
    logger("client_test.log");
    info!("\nTest: test_process_withdrawal");
    let mut client = Client::new(1);
    client.deposit(Some(initial_balance));
    client.set_locked(locked);

    let transaction = Transaction {
        transaction_type: "withdrawal".to_string(),
        client: 1,
        tx,
        amount: Some(withdrawal_amount),
    };

    process_withdrawal(&mut client, &transaction);

    if locked {
        assert_eq!(client.available(), initial_balance);
    } else {
        assert_eq!(client.available(), initial_balance - withdrawal_amount);
    }
}

/*
   I.   Account is locked
   II.  Transaction is able to be successfully put under a dispute
 */
#[test_case(1, 50.0, true, "Skipping transaction 1: Account 1 is locked.")]
#[test_case(1, 50.0, false, "Dispute successful for client 1.")]
#[allow(dead_code, unused_variables, unused_mut)]
fn test_process_dispute(tx: u32, amount: f32, locked: bool, expected_log: &str) {
    logger("client_test.log");
    info!("\nTest: test_process_withdrawal");
    let mut client = Client::new(1);
    let mut transaction = Transaction {
        transaction_type: "dispute".to_string(),
        client: 1,
        tx,
        amount: Some(amount),
    };

    client.deposit(Some(amount));
    if !locked {
        client.dispute(tx, Some(amount)).unwrap();
    }

    process_dispute(&mut client, &transaction);

    if locked {
        assert_eq!(client.held(), 0.0);
    } else {
        assert_eq!(client.held(), amount);
    }
}

/*
   I.   Account is locked
   II.  Transaction is able to be successfully resolved
 */
#[test_case(1, 50.0, true, "Skipping transaction 1: Account 1 is locked.")]
#[test_case(1, 50.0, false, "Transaction 1 resolved for client 1.")]
#[allow(dead_code, unused_variables, unused_mut)]
fn test_process_resolve(tx: u32, amount: f32, locked: bool, expected_log: &str) {
    logger("client_test.log");
    let mut client = Client::new(1);
    client.deposit(Some(amount));
    client.dispute(tx, Some(amount)).unwrap();
    client.set_locked(locked);

    let transaction = Transaction {
        transaction_type: "resolve".to_string(),
        client: 1,
        tx,
        amount: None,
    };

    process_resolve(&mut client, &transaction);

    if locked {
        assert_eq!(client.held(), amount);
    } else {
        assert_eq!(client.available(), amount);
    }
}

/*
   I.   Account is locked
   II.  Transaction is able to be successfully chargedback
 */
#[test_case(1, 50.0, true, "Skipping transaction 1: Account 1 is locked.")]
#[test_case(1, 50.0, false, "Transaction 1 chargedback for client 1.")]
#[allow(dead_code, unused_variables, unused_mut)]
fn test_process_chargeback(tx: u32, amount: f32, locked: bool, expected_log: &str) {
    logger("client_test.log");
    let mut client = Client::new(1);
    client.deposit(Some(amount));
    client.dispute(tx, Some(amount)).unwrap();

    let transaction = Transaction {
        transaction_type: "chargeback".to_string(),
        client: 1,
        tx,
        amount: None,
    };

    process_chargeback(&mut client, &transaction);

    if locked {
        assert_eq!(client.locked(), true);
    } else {
        assert!(client.locked());
    }
}