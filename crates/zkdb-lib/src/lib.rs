use sha2::{Digest, Sha256};
use sp1_sdk::{
    HashableKey, ProverClient, SP1ProofWithPublicValues, SP1ProvingKey, SP1PublicValues, SP1Stdin,
    SP1VerifyingKey,
};
use std::env;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use thiserror::Error;
use tracing::{debug, error, instrument};
use zkdb_store::{Store, StoreError};

// reexport zkdb_core
pub use zkdb_core::{Command, QueryResult};

#[derive(Debug, Clone)]
pub enum DatabaseType {
    Merkle,
}

pub struct Database {
    #[allow(dead_code)]
    engine: DatabaseType,
    store: Arc<dyn Store>,
    state: Vec<u8>,
    executor: SP1Executor,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProvenQueryResult {
    pub data: serde_json::Value,
    pub new_state: Vec<u8>,
    pub sp1_proof: Option<ProvenOutput>,
}

pub fn get_elf() -> &'static [u8] {
    debug!("Loading ELF binary from {}", env!("ZKDB_ELF_PATH"));
    include_bytes!(env!("ZKDB_ELF_PATH"))
}

impl Database {
    #[instrument(skip(store))]
    pub async fn new(
        engine: DatabaseType,
        store: Arc<dyn Store>,
        // bincoded state is optional, defaults to empty
        state: Option<Vec<u8>>,
    ) -> Result<Self, DatabaseError> {
        debug!("Creating new Database instance");
        let elf = get_elf();
        debug!("Loaded ELF binary, size: {} bytes", elf.len());

        Ok(Database {
            engine,
            store,
            state: state.unwrap_or_default(),
            executor: SP1Executor::new(elf),
        })
    }

    #[instrument(skip(self, value))]
    pub async fn put(
        &mut self,
        key: &str,
        value: &[u8],
        generate_proof: bool,
    ) -> Result<(), DatabaseError> {
        // 1. Store the actual value
        self.store.put(key, value).await?;

        // 2. Calculate hash for Merkle tree
        let mut hasher = Sha256::new();
        hasher.update(value);
        let value_hash = hex::encode(hasher.finalize());
        debug!("PUT: Original value: {:?}", String::from_utf8_lossy(value));
        debug!("PUT: Calculated hash: {}", value_hash);

        // 3. Store hash in Merkle tree via SP1
        let command = Command::Insert {
            key: key.to_string(),
            value: value_hash,
        };

        let result = self
            .executor
            .execute_query(&self.state, &command, generate_proof)?;

        debug!("PUT: Result from executor: {:?}", result.data);

        // update state
        self.set_state(result.new_state);

        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get(&self, key: &str, generate_proof: bool) -> Result<Vec<u8>, DatabaseError> {
        // 1. Get hash from Merkle tree for verification
        let command = Command::Query {
            key: key.to_string(),
        };
        let result = self
            .executor
            .execute_query(&self.state, &command, generate_proof)?;
        debug!("GET: Query Result: {:?}", result.data);

        if result.data.get("error").is_some() {
            return Err(DatabaseError::QueryExecutionFailed(format!(
                "Query execution failed, error: {:?}",
                result.data
            )));
        }

        let merkle_hash = result
            .data
            .get("value")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                DatabaseError::QueryExecutionFailed("Invalid result format".to_string())
            })?;

        // 2. Get actual value from store
        let value = self.store.get(key).await?;
        debug!(
            "GET: Retrieved value from store: {:?}",
            String::from_utf8_lossy(&value)
        );

        // 3. Verify hash matches
        let mut hasher = Sha256::new();
        hasher.update(&value);
        let computed_hash = hex::encode(hasher.finalize());
        debug!("GET: Computed hash of retrieved value: {}", computed_hash);

        if computed_hash != merkle_hash {
            return Err(DatabaseError::Store(StoreError::Storage(
                "Value hash mismatch - data may be corrupted".to_string(),
            )));
        }

        // Return the actual value
        Ok(value)
    }

    #[instrument(skip(self, command))]
    pub fn execute_query(
        &mut self,
        command: Command,
        generate_proof: bool,
    ) -> Result<ProvenQueryResult, DatabaseError> {
        debug!(?generate_proof, "Executing query");
        let result = self
            .executor
            .execute_query(&self.state, &command, generate_proof)?;
        debug!("Query executed successfully, updating state");
        self.state.clone_from(&result.new_state);
        Ok(result)
    }

    #[instrument(skip(self, proof))]
    pub fn verify_proof(&self, proof: &ProvenOutput) -> Result<bool, DatabaseError> {
        debug!("Verifying proof");
        self.executor.verify_proof(proof)
    }

    #[instrument(skip(self))]
    pub fn get_state(&self) -> &[u8] {
        &self.state
    }

    #[instrument(skip(self))]
    pub fn set_state(&mut self, state: Vec<u8>) {
        self.state.clone_from(&state);
    }

    #[instrument(skip(self, path))]
    pub fn save_state(&self, path: &Path) -> Result<(), DatabaseError> {
        debug!(path = ?path, "Saving database state");
        fs::write(path, &self.state).map_err(|e| {
            error!(error = ?e, "Failed to save state");
            DatabaseError::QueryExecutionFailed(format!("Failed to save state: {}", e))
        })
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProvenOutput {
    pub proof_data: SP1ProofWithPublicValues,
    pub vk: Vec<u8>,
}

#[derive(Error, Debug, serde::Serialize, serde::Deserialize)]
pub enum DatabaseError {
    #[error("Query execution failed: {0}")]
    QueryExecutionFailed(String),
    #[error("Proof generation failed: {0}")]
    ProofGenerationFailed(String),
    #[error("Proof verification failed: {0}")]
    ProofVerificationFailed(String),
    #[error("Store error: {0}")]
    Store(#[from] StoreError),
}

pub struct SP1Executor {
    client: ProverClient,
    elf: &'static [u8],
    pk: SP1ProvingKey,
    vk: SP1VerifyingKey,
}

impl SP1Executor {
    #[instrument(skip(elf))]
    pub fn new(elf: &'static [u8]) -> Self {
        debug!("Creating new SP1Executor");
        let client = ProverClient::new();
        debug!("Generated ProverClient");
        let (pk, vk) = client.setup(elf);
        debug!("Generated proving and verifying keys");
        SP1Executor {
            client,
            elf,
            pk,
            vk,
        }
    }

    #[instrument(skip(self, state, command))]
    pub fn execute_query(
        &self,
        state: &[u8],
        command: &Command,
        generate_proof: bool,
    ) -> Result<ProvenQueryResult, DatabaseError> {
        debug!(?generate_proof, "Preparing query execution");
        debug!(?command, "Command to execute");

        let mut stdin = SP1Stdin::new();
        stdin.write(&state.to_vec());
        stdin.write(command);
        debug!(?stdin, "Stdin prepared");

        if generate_proof {
            debug!("Generating proof");
            let proof = self
                .client
                .prove(&self.pk, stdin.clone())
                .run()
                .map_err(|e| {
                    error!(error = ?e, "Proof generation failed");
                    DatabaseError::ProofGenerationFailed(e.to_string())
                })?;
            debug!("Proof generated successfully");

            let (output, _) = self
                .client
                .execute(self.elf, stdin.clone())
                .run()
                .map_err(|e| {
                    error!(error = ?e, "Query execution failed");
                    DatabaseError::QueryExecutionFailed(format!(
                        "Failed to execute query with proof: {}",
                        e
                    ))
                })?;
            debug!("Query executed with proof");

            self.parse_output(
                output,
                Some(ProvenOutput {
                    proof_data: proof,
                    vk: self.vk.bytes32().as_bytes().to_vec(),
                }),
            )
        } else {
            debug!("Executing query without proof");
            let (output, _) = self.client.execute(self.elf, stdin).run().map_err(|e| {
                error!(error = ?e, "Query execution failed");
                DatabaseError::QueryExecutionFailed(format!(
                    "Failed to execute query without proof: {}",
                    e
                ))
            })?;
            debug!("Query executed successfully");
            self.parse_output(output, None)
        }
    }

    #[instrument(skip(self, output, proof))]
    fn parse_output(
        &self,
        output: SP1PublicValues,
        proof: Option<ProvenOutput>,
    ) -> Result<ProvenQueryResult, DatabaseError> {
        debug!("Parsing query output");
        let output_str = String::from_utf8(output.to_vec()).map_err(|e| {
            error!(error = ?e, "Failed to parse output as UTF-8");
            DatabaseError::QueryExecutionFailed(format!("Failed to parse output as UTF-8: {}", e))
        })?;

        let output_json: serde_json::Value = serde_json::from_str(&output_str).map_err(|e| {
            error!(error = ?e, "Failed to parse output as JSON");
            DatabaseError::QueryExecutionFailed(format!("Failed to parse output as JSON: {}", e))
        })?;

        debug!(?output_json, "Parsed output JSON");

        let data = output_json["data"].clone();
        let new_state = output_json["new_state"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_u64().unwrap() as u8)
            .collect();

        if let Some(proof) = proof.clone() {
            debug!("Verifying generated proof");
            self.verify_proof(&proof)?;
            debug!("Proof verified successfully");
        }

        Ok(ProvenQueryResult {
            data,
            new_state,
            sp1_proof: proof,
        })
    }

    #[instrument(skip(self, proof))]
    pub fn verify_proof(&self, proof: &ProvenOutput) -> Result<bool, DatabaseError> {
        debug!("Verifying proof");
        self.client
            .verify(&proof.proof_data, &self.vk)
            .map(|_| {
                debug!("Proof verified successfully");
                true
            })
            .map_err(|e| {
                error!(error = ?e, "Proof verification failed");
                DatabaseError::ProofVerificationFailed(e.to_string())
            })
    }
}
