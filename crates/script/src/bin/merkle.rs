//! An end-to-end example of using the SP1 SDK to generate a proof of a Merkle tree program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --bin merkle --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --bin merkle --release -- --prove
//! ```

use clap::{Parser, Subcommand};
use serde_json::Value;
use sp1_sdk::{ProverClient, SP1Stdin};
use std::fs;
use tracing::log::{error, info};

/// The ELF file for the zkdb-merkle program.
pub const ZKDB_MERKLE_ELF: &[u8] = include_bytes!("../../../../elf/riscv32im-succinct-zkvm-elf");

/// Commands for the Merkle client.
#[derive(Subcommand, Debug)]
enum Command {
    Insert { key: String, value: String },
    Query { key: String },
    Prove { key: String },
}

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Command,

    #[clap(long, global = true, help = "Generate and verify an SP1 proof")]
    prove: bool,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = Args::parse();

    // Load state from file or initialize.
    let state_file = "merkle_state.txt";
    let state = if let Ok(encoded_state) = fs::read_to_string(state_file) {
        Some(encoded_state)
    } else {
        None
    };

    // Prepare the command input.
    let command_json = match &args.command {
        Command::Insert { key, value } => serde_json::json!({
            "command": "insert",
            "params": {
                "key": key,
                "value": value
            },
            "state": state,
        }),
        Command::Query { key } => serde_json::json!({
            "command": "query",
            "params": {
                "key": key,
            },
            "state": state,
        }),
        Command::Prove { key } => serde_json::json!({
            "command": "prove",
            "params": {
                "key": key,
            },
            "state": state,
        }),
    };

    let command_str = serde_json::to_string(&command_json).unwrap();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    stdin.write(&command_str);

    // Setup the prover client.
    let client = ProverClient::new();

    // Execute the program.
    info!("Executing the program...");
    match client.execute(ZKDB_MERKLE_ELF, stdin.clone()).run() {
        Ok((output, report)) => {
            info!("Program executed successfully.");

            // Read the output.
            let output_str = String::from_utf8(output.as_slice().to_vec()).expect("Invalid UTF-8");
            let output_json: Value =
                serde_json::from_str(&output_str).expect("Invalid JSON output");

            // Display the result.
            println!(
                "Result: {}",
                serde_json::to_string_pretty(&output_json["result"]).unwrap()
            );

            // Save the updated state.
            if let Some(state_encoded) = output_json.get("state").and_then(|s| s.as_str()) {
                fs::write(state_file, state_encoded).expect("Failed to save state");
                println!("State updated and saved.");
            }

            // Record the number of cycles executed.
            info!("Number of cycles: {}", report.total_instruction_count());

            // Generate and verify proof if requested
            if args.prove {
                info!("Generating and verifying proof...");
                let (pk, vk) = client.setup(ZKDB_MERKLE_ELF);
                let proof = client
                    .prove(&pk, stdin)
                    .run()
                    .expect("Failed to generate proof");
                println!("Proof generated successfully.");
                client.verify(&proof, &vk).expect("Failed to verify proof");
                println!("Proof verified successfully.");
            }
        }
        Err(e) => error!("Execution failed: {:?}", e),
    }
}
