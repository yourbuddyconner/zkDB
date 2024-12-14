use std::sync::Arc;
use zkdb_lib::{Database, DatabaseType};
use zkdb_store::file::FileStore;

// Add this function to set up logging for tests
fn init() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug") // Set default level to debug
        .with_test_writer() // Use test writer to see output in test runs
        .try_init(); // Initialize the subscriber
}

#[tokio::test]
async fn test_storage_integration() {
    // Initialize logging
    init();

    // Create temporary directory for test
    let temp_dir = tempfile::tempdir().unwrap();
    let store = FileStore::new(temp_dir.path()).await.unwrap();

    let mut db = Database::new(DatabaseType::Merkle, Arc::new(store), None)
        .await
        .unwrap();

    // Test basic operations
    let key = "test_key";
    let value = b"test_value";

    // Put value
    db.put(key, value, false).await.unwrap();

    // Get value and verify it matches
    let retrieved = db.get(key, false).await.unwrap();
    assert_eq!(&retrieved, value);
}
