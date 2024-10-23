use clap::Parser;
use prettytable::{row, Table};
use sp1_sdk::{ProverClient, SP1Stdin};
use std::time::Instant;
use tracing::log::{error, info};

/// The ELF file for the zkdb-merkle program.
pub const ZKDB_MERKLE_ELF: &[u8] = include_bytes!("../../../../elf/riscv32im-succinct-zkvm-elf");

#[derive(Parser, Debug)]
struct Args {
    #[clap(long, default_value = "100")]
    iterations: usize,
}

struct BenchmarkResult {
    operation: String,
    cycles: u64,
    total_time: std::time::Duration,
    avg_time: std::time::Duration,
}

fn main() {
    sp1_sdk::utils::setup_logger();
    let args = Args::parse();

    let client = ProverClient::new();

    let insert_result = benchmark_operation(&client, "insert", args.iterations);
    let query_result = benchmark_operation(&client, "query", args.iterations);
    let prove_result = benchmark_operation(&client, "prove", args.iterations);

    print_results(&[insert_result, query_result, prove_result]);
}

fn benchmark_operation(
    client: &ProverClient,
    operation: &str,
    iterations: usize,
) -> BenchmarkResult {
    let mut total_time = std::time::Duration::new(0, 0);
    let cycles;

    // Execute once to get the cycle count
    let command = create_command(operation, 0);
    let mut stdin = SP1Stdin::new();
    stdin.write(&command);

    match client.execute(ZKDB_MERKLE_ELF, stdin).run() {
        Ok((_, report)) => {
            cycles = report.total_instruction_count();
        }
        Err(e) => {
            error!("Execution failed: {:?}", e);
            cycles = 0;
        }
    }

    // Run multiple iterations for timing
    for i in 0..iterations {
        let command = create_command(operation, i);
        let mut stdin = SP1Stdin::new();
        stdin.write(&command);

        let start = Instant::now();
        match client.execute(ZKDB_MERKLE_ELF, stdin).run() {
            Ok(_) => {
                total_time += start.elapsed();
            }
            Err(e) => error!("Execution failed: {:?}", e),
        }
    }

    BenchmarkResult {
        operation: operation.to_string(),
        cycles,
        total_time,
        avg_time: total_time / iterations as u32,
    }
}

fn create_command(operation: &str, i: usize) -> String {
    match operation {
        "insert" => format!(
            "{{
                \"command\": \"insert\",
                \"params\": {{
                    \"key\": \"key{}\",
                    \"value\": \"value{}\"
                }},
                \"state\": null
            }}",
            i, i
        ),
        "query" | "prove" => format!(
            "{{
                \"command\": \"{}\",
                \"params\": {{
                    \"key\": \"key{}\"
                }},
                \"state\": null
            }}",
            operation, i
        ),
        _ => panic!("Unknown operation: {}", operation),
    }
}

fn print_results(results: &[BenchmarkResult]) {
    let mut table = Table::new();
    table.add_row(row!["Operation", "Cycles", "Total Time", "Avg Time"]);

    for result in results {
        table.add_row(row![
            result.operation,
            result.cycles,
            format!("{:?}", result.total_time),
            format!("{:?}", result.avg_time)
        ]);
    }

    table.printstd();
}
