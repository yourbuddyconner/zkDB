use clap::{Arg, Command};
use std::fs;
use tracing_subscriber::{self, EnvFilter};
use zkdb_core::QueryResult;
use zkdb_lib::{Command as DbCommand, Database, ProvenQueryResult};

pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();

    let matches = Command::new("zkDB CLI")
        .version("1.0")
        .author("Your Name")
        .about("Interact with zkDB")
        .subcommand(
            Command::new("init")
                .about("Initialize a new database")
                .arg(
                    Arg::new("state_path")
                        .long("state")
                        .default_value("db_state.bin")
                        .help("Path to save the initial database state"),
                )
                .arg(
                    Arg::new("no_proof")
                        .long("no-proof")
                        .help("Disable SP1 proof generation")
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("query")
                .about("Execute a query")
                .arg(
                    Arg::new("command")
                        .required(true)
                        .help("Query command to execute"),
                )
                .arg(
                    Arg::new("no_proof")
                        .long("no-proof")
                        .help("Disable SP1 proof generation")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("state_path")
                        .long("state")
                        .default_value("db_state.bin")
                        .help("Path to database state file"),
                ),
        )
        .subcommand(
            Command::new("verify").about("Verify a proof").arg(
                Arg::new("proof_path")
                    .required(true)
                    .help("Path to the proof file"),
            ),
        )
        .subcommand(
            Command::new("insert")
                .about("Insert a key-value pair into the database")
                .arg(Arg::new("key").required(true).help("Key to insert"))
                .arg(Arg::new("value").required(true).help("Value to insert"))
                .arg(
                    Arg::new("no_proof")
                        .long("no-proof")
                        .help("Disable SP1 proof generation")
                        .action(clap::ArgAction::SetTrue),
                )
                .arg(
                    Arg::new("state_path")
                        .long("state")
                        .default_value("db_state.bin")
                        .help("Path to database state file"),
                ),
        )
        .subcommand(
            Command::new("prove")
                .about("Generate a Merkle proof for a key")
                .arg(
                    Arg::new("key")
                        .required(true)
                        .help("Key to generate proof for"),
                )
                .arg(
                    Arg::new("state_path")
                        .long("state")
                        .default_value("db_state.bin")
                        .help("Path to database state file"),
                )
                .arg(
                    Arg::new("output_path")
                        .long("output")
                        .help("Path to save the proof (defaults to proof_<key>_<timestamp>.bin)"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("init", init_matches)) => {
            let state_path = init_matches.get_one::<String>("state_path").unwrap();
            let _generate_proof = !init_matches.get_flag("no_proof");
            let initial_state = vec![]; // Empty initial state
            let db = Database::new(initial_state);
            println!("Initializing new database");
            let db_state = db.get_state();
            fs::write(state_path, bincode::serialize(&db_state)?)?;
            println!("Database state saved to: {}", state_path);
        }
        Some(("query", query_matches)) => {
            let command_str = query_matches.get_one::<String>("command").unwrap();
            let generate_proof = !query_matches.get_flag("no_proof");
            let state_path = query_matches.get_one::<String>("state_path").unwrap();

            let db_state: Vec<u8> = bincode::deserialize(&fs::read(state_path)?)?;
            let mut db = Database::new(db_state);

            let command = parse_command(command_str)?;
            match db.execute_query(command, generate_proof) {
                Ok(result) => {
                    fs::write(state_path, bincode::serialize(&db.get_state())?)?;

                    print_query_result(&result);

                    if generate_proof {
                        let proof_path = format!("proof_{}.bin", chrono::Utc::now().timestamp());
                        fs::write(&proof_path, bincode::serialize(&result)?)?;
                        println!("Proof saved to: {}", proof_path);
                    }
                }
                Err(e) => eprintln!("Query execution failed: {}", e),
            }
        }
        Some(("verify", verify_matches)) => {
            let proof_path = verify_matches.get_one::<String>("proof_path").unwrap();
            let proven_output: zkdb_lib::ProvenOutput =
                bincode::deserialize(&fs::read(proof_path)?)?;

            let db_state: Vec<u8> = bincode::deserialize(&fs::read("db_state.bin")?)?;
            let db = Database::new(db_state);

            match db.verify_proof(&proven_output) {
                Ok(true) => println!("Proof verified successfully"),
                Ok(false) => println!("Proof verification failed"),
                Err(e) => eprintln!("Proof verification error: {}", e),
            }
        }
        Some(("insert", insert_matches)) => {
            let key = insert_matches.get_one::<String>("key").unwrap();
            let value = insert_matches.get_one::<String>("value").unwrap();
            let generate_proof = !insert_matches.get_flag("no_proof");
            let state_path = insert_matches.get_one::<String>("state_path").unwrap();

            let db_state: Vec<u8> = bincode::deserialize(&fs::read(state_path)?)?;
            let mut db = Database::new(db_state);

            let command = DbCommand::Insert {
                key: key.clone(),
                value: value.clone(),
            };

            match db.execute_query(command, generate_proof) {
                Ok(result) => {
                    fs::write(state_path, bincode::serialize(&db.get_state())?)?;
                    println!("Successfully inserted key '{}' with value '{}'", key, value);
                    print_query_result(&result);
                }
                Err(e) => eprintln!("Insert failed: {}", e),
            }
        }
        Some(("prove", prove_matches)) => {
            let key = prove_matches.get_one::<String>("key").unwrap();
            let state_path = prove_matches.get_one::<String>("state_path").unwrap();

            let db_state: Vec<u8> = bincode::deserialize(&fs::read(state_path)?)?;
            let mut db = Database::new(db_state);

            let command = DbCommand::Prove { key: key.clone() };

            match db.execute_query(command, true) {
                Ok(result) => {
                    let output_path = prove_matches
                        .get_one::<String>("output_path")
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| {
                            format!("proof_{}_{}.bin", key, chrono::Utc::now().timestamp())
                        });

                    fs::write(&output_path, bincode::serialize(&result)?)?;

                    println!("Merkle Proof Generation:");
                    println!("------------------------");
                    print_proof_result(&result);
                    println!("\nProof saved to: {}", output_path);
                }
                Err(e) => eprintln!("Proof generation failed: {}", e),
            }
        }
        _ => {
            println!("Invalid command. Use --help for usage information.");
        }
    }

    Ok(())
}

fn parse_command(command_str: &str) -> Result<DbCommand, Box<dyn std::error::Error>> {
    if let Some(parts) = command_str.split_once(':') {
        if parts.0 == "insert" {
            let (key, value) = parts
                .1
                .split_once('=')
                .ok_or("Invalid insert format. Use 'insert:key=value'")?;
            return Ok(DbCommand::Insert {
                key: key.to_string(),
                value: value.to_string(),
            });
        }
    }

    Ok(DbCommand::Query {
        key: command_str.to_string(),
    })
}

fn print_query_result(result: &ProvenQueryResult) {
    println!("Query Result:");
    println!("New State: {:?}", result.new_state);
    println!("Output: {:?}", result.data);
}

fn print_proof_result(result: &ProvenQueryResult) {
    if let Ok(proof_data) = serde_json::from_value::<serde_json::Value>(result.data.clone()) {
        if let Some(root) = proof_data.get("root") {
            println!("Merkle Root: {}", root.as_str().unwrap_or("N/A"));
        }
        if let Some(index) = proof_data.get("index") {
            println!("Leaf Index: {}", index.as_u64().unwrap_or(0));
        }
        if let Some(leaf) = proof_data.get("leaf") {
            println!("Leaf Hash: {}", leaf.as_str().unwrap_or("N/A"));
        }
        println!("\nProof Details:");
        if let Some(proof) = proof_data.get("proof") {
            println!("Proof (base64): {}", proof.as_str().unwrap_or("N/A"));
        }
    } else {
        println!("Raw Result: {:?}", result.data);
    }
}
