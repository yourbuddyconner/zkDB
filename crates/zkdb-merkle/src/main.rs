//! A SP1 program for Merkle tree-based database operations.
//!
//! Supports `insert`, `query`, and `prove` commands.
//! State is managed by passing the Merkle tree in and out as serialized data.

#![no_main]
#![no_std]

extern crate alloc;

sp1_zkvm::entrypoint!(main);

use crate::alloc::string::ToString;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use serde::ser::{SerializeSeq, Serializer};
use serde::{Deserialize, Serialize};
use sp1_zkvm::io;

/// Key-value pair type.
type Key = String;
type Value = String;

/// Serializable state of the Merkle tree.
#[derive(Serialize, Deserialize)]
struct MerkleState {
    /// The list of leaves in the Merkle tree.
    leaves: Vec<[u8; 32]>,
    /// Map from keys to leaf indices.
    key_indices: BTreeMap<Key, usize>,
}

impl MerkleState {
    fn new() -> Self {
        MerkleState {
            leaves: Vec::new(),
            key_indices: BTreeMap::new(),
        }
    }
}

/// The main entry point for the SP1 program.
pub fn main() {
    // Read input data: command and state.
    let command_input: String = io::read();

    // Parse the input (expecting JSON format).
    let input: serde_json::Value =
        serde_json::from_str(&command_input).expect("Invalid JSON input");

    // Extract command and parameters.
    let command = input
        .get("command")
        .expect("Missing command")
        .as_str()
        .expect("Invalid command");
    let params = input.get("params").unwrap_or(&serde_json::Value::Null);
    let state_data = input.get("state").and_then(|s| s.as_str());

    // Deserialize state or create a new one.
    let mut merkle_state = if let Some(state_str) = state_data {
        let decoded = base64::decode(state_str).expect("Failed to decode state");
        serde_json::from_slice(&decoded).expect("Failed to deserialize state")
    } else {
        MerkleState::new()
    };

    // Perform the requested command.
    let result = match command {
        "insert" => {
            let key = params
                .get("key")
                .expect("Missing key")
                .as_str()
                .expect("Invalid key");
            let value = params
                .get("value")
                .expect("Missing value")
                .as_str()
                .expect("Invalid value");
            insert(&mut merkle_state, key.to_string(), value.to_string())
        }
        "query" => {
            let key = params
                .get("key")
                .expect("Missing key")
                .as_str()
                .expect("Invalid key");
            query(&merkle_state, key)
        }
        "prove" => {
            let key = params
                .get("key")
                .expect("Missing key")
                .as_str()
                .expect("Invalid key");
            prove(&merkle_state, key)
        }
        _ => Err("Unknown command".to_string()),
    };

    // Serialize updated state.
    let serialized_state = serde_json::to_vec(&merkle_state).expect("Failed to serialize state");
    let encoded_state = base64::encode(serialized_state);

    // Prepare the output.
    let output = serde_json::json!({
        "result": result.unwrap_or_else(|e| serde_json::json!({"error": e})),
        "state": encoded_state,
    });

    // Write the result as public output.
    let output_str = serde_json::to_string(&output).expect("Failed to serialize output");
    sp1_zkvm::io::commit_slice(output_str.as_bytes());
}

/// Inserts a new key-value pair into the Merkle tree.
fn insert(state: &mut MerkleState, key: Key, value: Value) -> Result<serde_json::Value, String> {
    // Hash the value.
    let leaf = Sha256::hash(value.as_bytes());
    // Insert into the tree.
    state.leaves.push(leaf);
    let index = state.leaves.len() - 1;
    state.key_indices.insert(key, index);

    Ok(serde_json::json!({"status": "inserted"}))
}

/// Queries the value associated with a key.
fn query(state: &MerkleState, key: &str) -> Result<serde_json::Value, String> {
    if let Some(&index) = state.key_indices.get(key) {
        let value_hash = &state.leaves[index];
        Ok(serde_json::json!({"value_hash": hex::encode(value_hash)}))
    } else {
        Err("Key not found".to_string())
    }
}

/// Generates a proof for a given key.
fn prove(state: &MerkleState, key: &str) -> Result<serde_json::Value, String> {
    if let Some(&index) = state.key_indices.get(key) {
        // Create Merkle tree.
        let merkle_tree = MerkleTree::<Sha256>::from_leaves(&state.leaves);
        // Generate proof.
        let proof = merkle_tree.proof(&[index]);
        // Get the root.
        let root = merkle_tree.root().ok_or("Tree is empty")?;

        // Custom serialization for MerkleProof
        let proof_serialized =
            serde_json::to_value(ProofWrapper(proof)).expect("Failed to serialize proof");
        let proof_encoded = base64::encode(serde_json::to_string(&proof_serialized).unwrap());

        Ok(serde_json::json!({
            "root": hex::encode(root),
            "proof": proof_encoded,
            "indices": [index],
            "leaf": hex::encode(state.leaves[index]),
        }))
    } else {
        Err("Key not found".to_string())
    }
}

// Custom wrapper for MerkleProof serialization
struct ProofWrapper(rs_merkle::MerkleProof<Sha256>);

impl Serialize for ProofWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let proof_hashes = self.0.proof_hashes();
        let mut seq = serializer.serialize_seq(Some(proof_hashes.len()))?;
        for hash in proof_hashes {
            seq.serialize_element(&hex::encode(hash))?;
        }
        seq.end()
    }
}
