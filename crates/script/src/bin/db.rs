//! An end-to-end example of using the SP1 SDK to generate a proof of a program that can be executed
//! or have a core proof generated.
//!
//! You can run this script using the following command:
//! ```shell
//! RUST_LOG=info cargo run --release -- --execute
//! ```
//! or
//! ```shell
//! RUST_LOG=info cargo run --release -- --prove
//! ```

use clap::Parser;
use sp1_sdk::{ProverClient, SP1Stdin};
use tracing::log::{error, info};

/// The ELF file for the Succinct RISC-V zkDB program.
pub const ZKDB_ELF: &[u8] = include_bytes!("../../../../elf/riscv32im-succinct-zkvm-elf");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,

    #[clap(long)]
    prove: bool,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = Args::parse();

    if args.execute == args.prove {
        eprintln!("Error: You must specify either --execute or --prove");
        std::process::exit(1);
    }

    // Setup the prover client.
    let client = ProverClient::new();

    // Setup the inputs.
    let mut stdin = SP1Stdin::new();
    // Use a sample input value
    let input_value: i32 = 42;
    stdin.write(&input_value);

    info!("Input value: {}", input_value);

    if args.execute {
        // Execute the program
        info!("Executing the program...");
        match client.execute(ZKDB_ELF, stdin).run() {
            Ok((output, report)) => {
                info!("Program executed successfully.");

                // Read the output.
                let result: i32 = output.as_slice()[0] as i32;
                info!("Result: {}", result);

                // Record the number of cycles executed.
                info!("Number of cycles: {}", report.total_instruction_count());
            }
            Err(e) => error!("Execution failed: {:?}", e),
        }
    } else {
        // Setup the program for proving.
        let (pk, vk) = client.setup(ZKDB_ELF);

        // Generate the proof
        let proof = client
            .prove(&pk, stdin)
            .run()
            .expect("failed to generate proof");

        println!("Successfully generated proof!");

        // Verify the proof.
        client.verify(&proof, &vk).expect("failed to verify proof");
        println!("Successfully verified proof!");
    }
}
