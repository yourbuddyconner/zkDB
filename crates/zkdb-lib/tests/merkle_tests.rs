use serde_json::json;
use zkdb_lib::{Command, Database, QueryResult};

fn setup_database() -> Database {
    let initial_state = Vec::new(); // Empty initial state
    Database::new(initial_state)
}

#[test]
fn test_insert_and_get() {
    let mut db = setup_database();

    // Insert a key-value pair
    let insert_command = Command::Insert {
        key: "test_key".to_string(),
        value: "test_value".to_string(),
    };
    let insert_result = db.execute_query(insert_command, false).unwrap();
    assert!(insert_result.data.is_null()); // Insert should return null

    // Get the inserted value
    let get_command = Command::Get {
        key: "test_key".to_string(),
    };
    let get_result = db.execute_query(get_command, false).unwrap();
    assert_eq!(get_result.data, json!("test_value"));
}

#[test]
fn test_proof_generation_and_verification() {
    let mut db = setup_database();

    // Insert a key-value pair with proof generation
    let insert_command = Command::Insert {
        key: "proof_key".to_string(),
        value: "proof_value".to_string(),
    };
    let insert_result = db.execute_query(insert_command, true).unwrap();

    // Verify the proof
    assert!(db.verify_proof(&insert_result.new_state).unwrap());

    // Get the inserted value with proof generation
    let get_command = Command::Get {
        key: "proof_key".to_string(),
    };
    let get_result = db.execute_query(get_command, true).unwrap();

    // Verify the proof
    assert!(db.verify_proof(&get_result.new_state).unwrap());
    assert_eq!(get_result.data, json!("proof_value"));
}

#[test]
fn test_multiple_operations() {
    let mut db = setup_database();

    // Insert multiple key-value pairs
    for i in 0..5 {
        let insert_command = Command::Insert {
            key: format!("key_{}", i),
            value: format!("value_{}", i),
        };
        db.execute_query(insert_command, false).unwrap();
    }

    // Verify all inserted values
    for i in 0..5 {
        let get_command = Command::Get {
            key: format!("key_{}", i),
        };
        let get_result = db.execute_query(get_command, false).unwrap();
        assert_eq!(get_result.data, json!(format!("value_{}", i)));
    }

    // Update a value
    let update_command = Command::Insert {
        key: "key_2".to_string(),
        value: "updated_value".to_string(),
    };
    db.execute_query(update_command, false).unwrap();

    // Verify the updated value
    let get_command = Command::Get {
        key: "key_2".to_string(),
    };
    let get_result = db.execute_query(get_command, false).unwrap();
    assert_eq!(get_result.data, json!("updated_value"));
}
