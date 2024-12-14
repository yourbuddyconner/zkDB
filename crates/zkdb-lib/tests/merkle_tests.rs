use hex;
use serial_test::serial;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tempfile;
use zkdb_lib::{Command, Database, DatabaseType};
use zkdb_store::file::FileStore;

fn init() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .with_test_writer()
        .try_init();
}

async fn setup_database() -> (Database, Arc<FileStore>) {
    let temp_dir = tempfile::tempdir().unwrap();
    let store = Arc::new(FileStore::new(temp_dir.path()).await.unwrap());
    let db = Database::new(DatabaseType::Merkle, store.clone(), None)
        .await
        .unwrap();
    (db, store)
}

#[tokio::test]
#[serial]
async fn test_insert_and_get() {
    init();
    let (mut db, _store) = setup_database().await;

    // Insert a key-value pair
    let key = "test_key";
    let value = b"test_value";

    // First hash the value like the Database::put method does
    let mut hasher = Sha256::new();
    hasher.update(value);
    let value_hash = hex::encode(hasher.finalize());

    // Now send the hash to the Merkle tree
    let insert_command = Command::Insert {
        key: key.to_string(),
        value: value_hash.clone(), // Send the hex-encoded hash
    };

    tracing::debug!("Executing insert command");
    let insert_result = db.execute_query(insert_command, false).unwrap();
    tracing::debug!("Insert result: {:?}", insert_result.data);
    assert!(insert_result.data["inserted"].as_bool().unwrap());

    // Query the inserted value
    let get_command = Command::Query {
        key: key.to_string(),
    };

    tracing::debug!("Executing query command");
    let get_result = db.execute_query(get_command, false).unwrap();
    tracing::debug!("Query result: {:?}", get_result.data);
    assert!(get_result.data["found"].as_bool().unwrap());

    // Verify the returned hash matches
    assert_eq!(get_result.data["value"].as_str().unwrap(), value_hash);
}

#[tokio::test]
#[serial]
async fn test_proof_generation_and_verification() {
    init();
    let (mut db, _store) = setup_database().await;

    let key = "proof_key";
    let value = b"proof_value";

    // Hash the value before sending to Merkle tree
    let mut hasher = Sha256::new();
    hasher.update(value);
    let value_hash = hex::encode(hasher.finalize());

    // Insert with proof generation
    tracing::debug!("Inserting value with proof generation");
    let insert_command = Command::Insert {
        key: key.to_string(),
        value: value_hash, // Send the hex-encoded hash
    };
    let insert_result = db.execute_query(insert_command, true).unwrap();
    tracing::debug!("Insert with proof result: {:?}", insert_result.data);
    assert!(insert_result.sp1_proof.is_some());

    // Generate proof
    tracing::debug!("Generating proof for key");
    let prove_command = Command::Prove {
        key: key.to_string(),
    };
    let prove_result = db.execute_query(prove_command, true).unwrap();
    tracing::debug!("Proof generation result: {:?}", prove_result.data);

    // Verify proof exists
    assert!(prove_result.data["proof"].is_string());
    assert!(prove_result.data["root"].is_string());
}

#[tokio::test]
#[serial]
async fn test_multiple_operations() {
    init();
    let (mut db, _store) = setup_database().await;

    // Insert multiple key-value pairs
    for i in 0..5 {
        let key = format!("key_{}", i);
        let value = format!("value_{}", i);
        let value_bytes = value.as_bytes();

        // Hash the value before sending to Merkle tree
        let mut hasher = Sha256::new();
        hasher.update(value_bytes);
        let value_hash = hex::encode(hasher.finalize());

        let insert_command = Command::Insert {
            key: key.clone(),
            value: value_hash, // Send the hex-encoded hash
        };

        tracing::debug!("Inserting key-value pair {}", i);
        let result = db.execute_query(insert_command, false).unwrap();
        tracing::debug!("Insert result for pair {}: {:?}", i, result.data);
        assert!(result.data["inserted"].as_bool().unwrap());
    }

    // Verify all values
    for i in 0..5 {
        let key = format!("key_{}", i);
        let get_command = Command::Query { key: key.clone() };

        tracing::debug!("Querying key {}", key);
        let result = db.execute_query(get_command, false).unwrap();
        tracing::debug!("Query result for key {}: {:?}", key, result.data);
        assert!(result.data["found"].as_bool().unwrap());
    }
}

#[tokio::test]
#[serial]
async fn test_merkle_tree_properties() {
    init();
    let (mut db, _store) = setup_database().await;

    // Insert some values and collect their hashes
    let mut value_hashes = Vec::new();
    for i in 0..4 {
        let key = format!("key_{}", i);
        let value = format!("value_{}", i);

        // Hash the value before sending to Merkle tree
        let mut hasher = Sha256::new();
        hasher.update(value.as_bytes());
        let value_hash = hex::encode(hasher.finalize());

        tracing::debug!("Inserting test value {}", i);
        let insert_command = Command::Insert {
            key: key.clone(),
            value: value_hash.clone(), // Send the hex-encoded hash
        };
        let result = db.execute_query(insert_command, false).unwrap();
        value_hashes.push(result.data["leaf"].as_str().unwrap().to_string());
    }

    // Generate and verify proofs for each value
    for (i, hash) in value_hashes.iter().enumerate() {
        let key = format!("key_{}", i);
        let prove_command = Command::Prove { key: key.clone() };

        tracing::debug!("Generating proof for key {}", key);
        let result = db.execute_query(prove_command, false).unwrap();
        tracing::debug!("Proof result for key {}: {:?}", key, result.data);

        // Verify proof contains necessary components
        assert!(result.data["proof"].is_string());
        assert!(result.data["root"].is_string());
        assert_eq!(result.data["leaf"].as_str().unwrap(), hash);
    }
}

#[tokio::test]
#[serial]
async fn test_state_consistency() {
    init();
    let (mut db, _store) = setup_database().await;

    // Insert initial value
    let key = "state_test_key";
    let value = b"state_test_value";

    // Hash the value before sending to Merkle tree
    let mut hasher = Sha256::new();
    hasher.update(value);
    let value_hash = hex::encode(hasher.finalize());

    tracing::debug!("Inserting initial value");
    let insert_command = Command::Insert {
        key: key.to_string(),
        value: value_hash, // Send the hex-encoded hash
    };
    db.execute_query(insert_command, false).unwrap();

    // Get current state
    let state = db.get_state().to_vec();
    tracing::debug!("Current state size: {} bytes", state.len());

    // Create new database with saved state
    let (mut new_db, _) = setup_database().await;
    new_db.set_state(state);

    // Verify value exists in new database
    let get_command = Command::Query {
        key: key.to_string(),
    };

    tracing::debug!("Querying value from new database instance");
    let result = new_db.execute_query(get_command, false).unwrap();
    tracing::debug!("Query result from new instance: {:?}", result.data);
    assert!(result.data["found"].as_bool().unwrap());
}
