//! A SP1 program for Merkle tree-based database operations.
//!
//! Supports `insert`, `query`, and `prove` commands.
//! State is managed by passing the Merkle tree in and out as serialized data.

sp1_zkvm::entrypoint!(main);

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;
use rs_merkle::proof_serializers;
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use serde::{Deserialize, Serialize};
use sp1_zkvm::io;
use zkdb_core::{Command, DatabaseEngine, DatabaseError, QueryResult};

/// Key-value pair type.
type Key = String;
// type Value = String;

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

pub struct MerkleEngine;

impl DatabaseEngine for MerkleEngine {
    fn execute_query(
        &mut self,
        state: &[u8],
        command: &Command,
    ) -> Result<QueryResult, DatabaseError> {
        main_internal(state, command)
    }
}

pub fn main() {
    let state: Vec<u8> = io::read::<Vec<u8>>();
    let command: Command = io::read::<Command>();

    let result = main_internal(&state, &command).unwrap_or_else(|e| QueryResult {
        data: serde_json::json!({
            "error": {
                "type": "QueryExecutionFailed",
                "state_len": state.len(),
                "details": format!("{:?}", e),
            }
        }),
        new_state: state,
    });

    let output = serde_json::to_vec(&result).expect("Failed to serialize output");
    sp1_zkvm::io::commit_slice(&output);
}

fn main_internal(state: &[u8], command: &Command) -> Result<QueryResult, DatabaseError> {
    // if the state is empty, initialize it
    let mut merkle_state: MerkleState = if state.is_empty() {
        MerkleState::new()
    } else {
        bincode::deserialize(state)
            .map_err(|e| DatabaseError::QueryExecutionFailed(e.to_string()))?
    };

    let result = match command {
        Command::Insert { key, value } => insert(&mut merkle_state, key.clone(), value.clone())?,
        Command::Query { key } => query(&merkle_state, key)?,
        Command::Prove { key } => prove(&merkle_state, key)?,
    };
    Ok(result)
}

/// Inserts a new key-value pair into the Merkle tree.
fn insert(
    state: &mut MerkleState,
    key: String,
    value: String,
) -> Result<QueryResult, DatabaseError> {
    // Hash the value.
    let leaf = Sha256::hash(value.as_bytes());
    // Insert into the tree.
    state.leaves.push(leaf);
    let index = state.leaves.len() - 1;
    state.key_indices.insert(key.clone(), index);

    Ok(QueryResult {
        data: serde_json::json!({
            "key": key.clone(),
            "value": value.clone(),
            "index": index,
            "leaf": hex::encode(leaf),
            "inserted": true,
        }),
        new_state: bincode::serialize(&state).unwrap(),
    })
}

/// Queries the value associated with a key.
fn query(state: &MerkleState, key: &str) -> Result<QueryResult, DatabaseError> {
    if let Some(&index) = state.key_indices.get(key) {
        let value_hash = &state.leaves[index];
        Ok(QueryResult {
            data: serde_json::json!({"value_hash": hex::encode(value_hash)}),
            new_state: bincode::serialize(&state).unwrap(),
        })
    } else {
        Err(DatabaseError::QueryExecutionFailed(
            "Key not found".to_string(),
        ))
    }
}

/// Generates a Merkle Inclusion Proof for a given key.
fn prove(state: &MerkleState, key: &str) -> Result<QueryResult, DatabaseError> {
    if let Some(&index) = state.key_indices.get(key) {
        let merkle_tree = MerkleTree::<Sha256>::from_leaves(&state.leaves);
        let proof = merkle_tree.proof(&[index]);
        let root = merkle_tree
            .root()
            .ok_or_else(|| DatabaseError::QueryExecutionFailed("Tree is empty".to_string()))?;

        let proof_serialized: Vec<u8> = proof.serialize::<proof_serializers::ReverseHashesOrder>();
        let proof_encoded = base64::encode(proof_serialized);

        Ok(QueryResult {
            data: serde_json::json!({
                "root": hex::encode(root),
                "proof": proof_encoded,
                "index": index,
                "leaf": hex::encode(state.leaves[index]),
            }),
            new_state: bincode::serialize(&state).unwrap(),
        })
    } else {
        Err(DatabaseError::QueryExecutionFailed(
            "Key not found".to_string(),
        ))
    }
}
