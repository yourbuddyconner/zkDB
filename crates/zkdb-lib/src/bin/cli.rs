use bincode;
use clap::{Arg, Command};
use std::fs;
use std::path::PathBuf;
use tracing_subscriber::{self, EnvFilter};
use zkdb_core::QueryResult;
use zkdb_lib::{Command as DbCommand, Database};

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
            Command::new("init").about("Initialize a new database").arg(
                Arg::new("state_path")
                    .long("state")
                    .default_value("db_state.bin")
                    .help("Path to save the initial database state"),
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
                    Arg::new("generate_proof")
                        .long("generate-proof")
                        .help("Generate a proof for the query"),
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
                    Arg::new("state_path")
                        .long("state")
                        .default_value("db_state.bin")
                        .help("Path to database state file"),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("init", init_matches)) => {
            let state_path = init_matches.get_one::<String>("state_path").unwrap();
            let initial_state = vec![]; // Empty initial state
            let db = Database::new(initial_state);
            println!("Initializing new database");
            let db_state = db.get_state();
            fs::write(state_path, bincode::serialize(&db_state)?)?;
            println!("Database state saved to: {}", state_path);
        }
        Some(("query", query_matches)) => {
            let command_str = query_matches.get_one::<String>("command").unwrap();
            let generate_proof = query_matches.contains_id("generate_proof");
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
            let state_path = insert_matches.get_one::<String>("state_path").unwrap();

            let db_state: Vec<u8> = bincode::deserialize(&fs::read(state_path)?)?;
            let mut db = Database::new(db_state);

            let command = DbCommand::Insert {
                key: key.clone(),
                value: value.clone(),
            };

            match db.execute_query(command, false) {
                Ok(result) => {
                    fs::write(state_path, bincode::serialize(&db.get_state())?)?;
                    println!("Successfully inserted key '{}' with value '{}'", key, value);
                    print_query_result(&result);
                }
                Err(e) => eprintln!("Insert failed: {}", e),
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
        match parts.0 {
            "insert" => {
                let (key, value) = parts
                    .1
                    .split_once('=')
                    .ok_or("Invalid insert format. Use 'insert:key=value'")?;
                return Ok(DbCommand::Insert {
                    key: key.to_string(),
                    value: value.to_string(),
                });
            }
            _ => {}
        }
    }

    Ok(DbCommand::Query {
        key: command_str.to_string(),
    })
}

fn print_query_result(result: &QueryResult) {
    println!("Query Result:");
    println!("New State: {:?}", result.new_state);
    println!("Output: {:?}", result.data);
}
