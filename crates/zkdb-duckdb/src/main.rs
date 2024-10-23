//! A SP1 program for time series analysis and forecasting.
//!
//! This program demonstrates how to perform time series calculations within a zero-knowledge proof system.
//! It takes a series of timestamps and corresponding forecast values as input, performs statistical
//! calculations, and outputs the results in a format compatible with Solidity smart contracts.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use duckdb::{params, Connection};
use sp1_zkvm::io;

/// The main entry point for the SP1 program.
///
/// This function performs the following steps:
/// 1. Reads input data (timestamps and forecast values) from the prover.
/// 2. Creates a TimeSeries instance and calculates statistical measures.
/// 3. Converts the results to Solidity-compatible formats.
/// 4. Encodes the public values for verification in a smart contract.
/// 5. Commits the encoded data as public output of the ZK proof.
pub fn main() {
    // Read input data (for simplicity, we'll just use a single integer)
    let input_value: i32 = io::read();

    // Perform database operations
    let result = perform_db_operations(input_value);

    // Write the result as public output
    sp1_zkvm::io::commit_slice(&[result as u8]);
}

fn perform_db_operations(input_value: i32) -> i32 {
    // Create an in-memory DuckDB connection
    let conn = Connection::open_in_memory().unwrap();

    // Create a table
    conn.execute(
        "CREATE TABLE test (id INTEGER PRIMARY KEY, value INTEGER)",
        params![],
    )
    .unwrap();

    // Insert the input value
    conn.execute("INSERT INTO test (value) VALUES (?)", params![input_value])
        .unwrap();

    // Query the inserted value
    let mut stmt = conn.prepare("SELECT value FROM test WHERE id = 1").unwrap();
    let mut rows = stmt.query(params![]).unwrap();

    let result = if let Some(row) = rows.next().unwrap() {
        row.get(0).unwrap()
    } else {
        -1 // Return -1 if no row was found
    };

    // Close the connection
    conn.close().unwrap();

    result
}
