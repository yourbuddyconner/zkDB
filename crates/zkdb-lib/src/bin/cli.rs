use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;
use zkdb_lib::{Database, DatabaseType};
use zkdb_store::file::FileStore;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the database storage directory
    #[arg(short, long, default_value = ".zkdb")]
    data_dir: PathBuf,

    /// Path to the state file
    #[arg(short, long, default_value = ".zkdb/state.bin")]
    state_file: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Insert a key-value pair
    Put {
        /// Key to insert
        key: String,
        /// Value to insert
        value: String,
        /// Generate proof
        #[arg(short, long)]
        proof: bool,
    },
    /// Query a value by key
    Get {
        /// Key to query
        key: String,
        /// Generate proof
        #[arg(short, long)]
        proof: bool,
    },
    /// Initialize a new database
    Init,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    // Create data directory if it doesn't exist
    tokio::fs::create_dir_all(&cli.data_dir).await?;

    // Initialize store
    let store = FileStore::new(&cli.data_dir).await?;

    // Load existing state if available
    let state_bytes = if cli.state_file.exists() {
        Some(tokio::fs::read(&cli.state_file).await?)
    } else {
        None
    };

    // Initialize database
    let mut db = Database::new(DatabaseType::Merkle, Arc::new(store), state_bytes).await?;

    match cli.command {
        Commands::Put { key, value, proof } => {
            info!("Inserting key: {}", key);
            db.put(&key, value.as_bytes(), proof).await?;
            // Save state after modification
            db.save_state(&cli.state_file)?;
            println!("Successfully inserted key: {}", key);
        }
        Commands::Get { key, proof } => {
            info!("Querying key: {}", key);
            match db.get(&key, proof).await {
                Ok(value) => {
                    println!("Value: {:?}", String::from_utf8_lossy(&value));
                }
                Err(e) => {
                    println!("Error retrieving key {}: {}", key, e);
                }
            }
        }
        Commands::Init => {
            info!("Initializing new database");
            // Save initial empty state
            db.save_state(&cli.state_file)?;
            println!("Database initialized at {:?}", cli.data_dir);
            println!("State file created at {:?}", cli.state_file);
        }
    }

    Ok(())
}
