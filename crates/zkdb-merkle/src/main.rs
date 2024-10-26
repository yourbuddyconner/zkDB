//! A SP1 program for Merkle tree-based database operations.
//!
//! Supports `insert`, `query`, and `prove` commands.
//! State is managed by passing the Merkle tree in and out as serialized data.

#![no_main]
sp1_zkvm::entrypoint!(main);

extern crate alloc;

use crate::alloc::string::ToString;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use rs_merkle::{algorithms::Sha256, Hasher, MerkleTree};
use serde::ser::{SerializeSeq, Serializer};
use serde::{Deserialize, Serialize};
use sp1_zkvm::io;
use zkdb_core::{Command, DatabaseEngine, DatabaseError, QueryResult};

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

/// Custom wrapper for MerkleProof serialization
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
    let state: Vec<u8> = io::read();
    let command: Command = io::read();

    let result = main_internal(&state, &command).unwrap_or_else(|e| QueryResult {
        data: serde_json::json!({"error": e}),
        new_state: state,
    });

    let output = serde_json::to_vec(&result).expect("Failed to serialize output");
    sp1_zkvm::io::commit_slice(&output);
}

fn main_internal(state: &[u8], command: &Command) -> Result<QueryResult, DatabaseError> {
    let mut merkle_state: MerkleState = bincode::deserialize(state)
        .map_err(|e| DatabaseError::QueryExecutionFailed(e.to_string()))?;

    let (result, new_state) = match command {
        Command::Insert { key, value } => {
            let insert_result = insert(&mut merkle_state, key.clone(), value.clone())?;
            (insert_result, merkle_state)
        }
        Command::Query { key } => {
            let query_result = query(&merkle_state, key)?;
            (query_result, merkle_state)
        }
        Command::Prove { key } => {
            let prove_result = prove(&merkle_state, key)?;
            (prove_result, merkle_state)
        }
    };

    let new_state_bytes = bincode::serialize(&new_state)
        .map_err(|e| DatabaseError::QueryExecutionFailed(e.to_string()))?;

    Ok(QueryResult {
        data: result,
        new_state: new_state_bytes,
    })
}

/// Inserts a new key-value pair into the Merkle tree.
fn insert(
    state: &mut MerkleState,
    key: String,
    value: String,
) -> Result<serde_json::Value, DatabaseError> {
    // Hash the value.
    let leaf = Sha256::hash(value.as_bytes());
    // Insert into the tree.
    state.leaves.push(leaf);
    let index = state.leaves.len() - 1;
    state.key_indices.insert(key, index);

    Ok(serde_json::json!({"status": "inserted"}))
}

/// Queries the value associated with a key.
fn query(state: &MerkleState, key: &str) -> Result<serde_json::Value, DatabaseError> {
    if let Some(&index) = state.key_indices.get(key) {
        let value_hash = &state.leaves[index];
        Ok(serde_json::json!({"value_hash": hex::encode(value_hash)}))
    } else {
        Err(DatabaseError::QueryExecutionFailed(
            "Key not found".to_string(),
        ))
    }
}

/// Generates a Merkle Inclusion Proof for a given key.
fn prove(state: &MerkleState, key: &str) -> Result<serde_json::Value, DatabaseError> {
    if let Some(&index) = state.key_indices.get(key) {
        let merkle_tree = MerkleTree::<Sha256>::from_leaves(&state.leaves);
        let proof = merkle_tree.proof(&[index]);
        let root = merkle_tree
            .root()
            .ok_or_else(|| DatabaseError::QueryExecutionFailed("Tree is empty".to_string()))?;

        let proof_serialized = serde_json::to_value(ProofWrapper(proof))
            .map_err(|e| DatabaseError::QueryExecutionFailed(e.to_string()))?;
        let proof_encoded = base64::encode(serde_json::to_string(&proof_serialized).unwrap());

        Ok(serde_json::json!({
            "root": hex::encode(root),
            "proof": proof_encoded,
            "index": index,
            "leaf": hex::encode(state.leaves[index]),
        }))
    } else {
        Err(DatabaseError::QueryExecutionFailed(
            "Key not found".to_string(),
        ))
    }
}
