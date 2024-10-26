use clap::{App, Arg, SubCommand};
use std::fs;
use zkdb_core::{DatabaseError, QueryResult};
use zkdb_lib::{Command, Database};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("zkDB CLI")
        .version("1.0")
        .author("Your Name")
        .about("Interact with zkDB")
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize a new database")
                .arg(
                    Arg::with_name("elf_path")
                        .required(true)
                        .help("Path to the ELF file"),
                ),
        )
        .subcommand(
            SubCommand::with_name("query")
                .about("Execute a query")
                .arg(
                    Arg::with_name("command")
                        .required(true)
                        .help("Query command to execute"),
                )
                .arg(
                    Arg::with_name("generate_proof")
                        .long("generate-proof")
                        .help("Generate a proof for the query"),
                ),
        )
        .subcommand(
            SubCommand::with_name("verify").about("Verify a proof").arg(
                Arg::with_name("proof_path")
                    .required(true)
                    .help("Path to the proof file"),
            ),
        )
        .get_matches();

    match matches.subcommand() {
        ("init", Some(init_matches)) => {
            let elf_path = init_matches.value_of("elf_path").unwrap();
            let elf_data = fs::read(elf_path)?;
            let initial_state = vec![]; // Empty initial state
            let db = Database::new(Box::leak(elf_data.into_boxed_slice()), initial_state);
            println!("Database initialized with ELF from {}", elf_path);
            // Here you might want to save the database state to a file
        }
        ("query", Some(query_matches)) => {
            let command_str = query_matches.value_of("command").unwrap();
            let generate_proof = query_matches.is_present("generate_proof");

            // For this example, we'll assume the database is already initialized
            // In a real application, you'd load the database state from a file
            let elf_data = include_bytes!("../../../../elf/riscv32im-succinct-zkvm-elf");
            let mut db = Database::new(elf_data, vec![]);

            let command = parse_command(command_str)?;
            match db.execute_query(command, generate_proof) {
                Ok(result) => print_query_result(result),
                Err(e) => eprintln!("Query execution failed: {}", e),
            }
        }
        ("verify", Some(verify_matches)) => {
            let proof_path = verify_matches.value_of("proof_path").unwrap();
            let proof_data = fs::read(proof_path)?;
            let proven_output: zkdb_lib::ProvenOutput = bincode::deserialize(&proof_data)?;

            // Again, assuming the database is already initialized
            let elf_data = include_bytes!("../../../../elf/riscv32im-succinct-zkvm-elf");
            let db = Database::new(elf_data, vec![]);

            match db.verify_proof(&proven_output) {
                Ok(true) => println!("Proof verified successfully"),
                Ok(false) => println!("Proof verification failed"),
                Err(e) => eprintln!("Proof verification error: {}", e),
            }
        }
        _ => {
            println!("Invalid command. Use --help for usage information.");
        }
    }

    Ok(())
}

fn parse_command(command_str: &str) -> Result<Command, Box<dyn std::error::Error>> {
    // This is a placeholder. You'll need to implement proper parsing based on your Command enum
    Ok(Command::Query(command_str.to_string()))
}

fn print_query_result(result: QueryResult) {
    println!("Query Result:");
    println!("New State: {:?}", result.new_state);
    println!("Output: {:?}", result.data);
    // if let Some(proof) = result.proof {
    //     println!("Proof generated: {:?}", proof);
    // }
}
