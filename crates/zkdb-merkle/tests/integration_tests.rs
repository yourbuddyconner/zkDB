use serde_json::Value;
use sp1_sdk::{ProverClient, SP1Stdin};

#[test]
fn test_insert_query_prove() {
    let client = ProverClient::new();
    let mut state = None;

    // Insert
    let insert_command = serde_json::json!({
        "command": "insert",
        "params": {
            "key": "testkey",
            "value": "testvalue"
        },
        "state": state,
    });
    let output = run_program(&client, insert_command);
    assert_eq!(output["result"]["status"], "inserted");
    state = output
        .get("state")
        .and_then(|s| s.as_str())
        .map(String::from);

    // Query
    let query_command = serde_json::json!({
        "command": "query",
        "params": {
            "key": "testkey",
        },
        "state": state.clone(),
    });
    let output = run_program(&client, query_command);
    assert!(output["result"]["value_hash"].is_string());

    // Prove
    let prove_command = serde_json::json!({
        "command": "prove",
        "params": {
            "key": "testkey",
        },
        "state": state.clone(),
    });
    let output = run_program(&client, prove_command);
    assert!(output["result"]["root"].is_string());
    assert!(output["result"]["proof"].is_string());
}

fn run_program(client: &ProverClient, input_json: serde_json::Value) -> Value {
    let command_str = serde_json::to_string(&input_json).unwrap();
    let mut stdin = SP1Stdin::new();
    stdin.write(&command_str);

    let zkdb_merkle_elf = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

    let (output, _) = client
        .execute(zkdb_merkle_elf, stdin)
        .run()
        .expect("Failed to execute program");

    let output_str = String::from_utf8(output.as_slice().to_vec()).expect("Invalid UTF-8");
    serde_json::from_str(&output_str).expect("Invalid JSON output")
}
