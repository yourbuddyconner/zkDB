//! A SP1 program for time series analysis and forecasting.
//!
//! This program demonstrates how to perform time series calculations within a zero-knowledge proof system.
//! It takes a series of timestamps and corresponding forecast values as input, performs statistical
//! calculations, and outputs the results in a format compatible with Solidity smart contracts.

#![no_main]
sp1_zkvm::entrypoint!(main);

use sled;
use sp1_zkvm::io;

/// The main entry point for the SP1 program.
///
/// This function performs the following steps:
/// 1. Reads input data from the prover.
/// 2. Performs database operations using sled.
/// 3. Outputs the result as public output of the ZK proof.
pub fn main() {
    // Read input data (for simplicity, we'll just use a single integer)
    let input_value: i32 = io::read();

    // Perform database operations
    let result = perform_db_operations(input_value);

    // Write the result as public output
    sp1_zkvm::io::commit_slice(&[result as u8]);
}

fn perform_db_operations(input_value: i32) -> i32 {
    // Open a sled database in memory
    let config = sled::Config::new().temporary(true);
    let db = config.open().unwrap();

    // Insert the input value with a key
    db.insert(b"id-1", &input_value.to_be_bytes()).unwrap();

    // Retrieve the value
    let retrieved = db.get(b"id-1").unwrap();

    let result = if let Some(value) = retrieved {
        let bytes = value.as_ref();
        let int_bytes: [u8; 4] = bytes.try_into().expect("Slice with incorrect length");
        i32::from_be_bytes(int_bytes)
    } else {
        -1 // Return -1 if no value was found
    };

    // Close the database
    db.flush().unwrap();

    result
}
