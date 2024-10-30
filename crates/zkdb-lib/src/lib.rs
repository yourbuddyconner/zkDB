use sp1_sdk::{
    HashableKey, ProverClient, SP1ProofWithPublicValues, SP1ProvingKey, SP1PublicValues, SP1Stdin,
    SP1VerifyingKey,
};
use std::env;
use std::fs;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, error, instrument};

// reexport zkdb_core
pub use zkdb_core::{Command, QueryResult};

pub struct Database {
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
    #[instrument(skip(initial_state))]
    pub fn new(initial_state: Vec<u8>) -> Self {
        debug!("Creating new Database instance");
        let elf = get_elf();
        debug!("Loaded ELF binary, size: {} bytes", elf.len());
        Database {
            state: initial_state,
            executor: SP1Executor::new(elf),
        }
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

    #[instrument(skip(self, path))]
    pub fn save_state(&self, path: &Path) -> Result<(), DatabaseError> {
        debug!(path = ?path, "Saving database state");
        fs::write(path, bincode::serialize(&self.state).unwrap()).map_err(|e| {
            error!(error = ?e, "Failed to save state");
            DatabaseError::QueryExecutionFailed(e.to_string())
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
                    DatabaseError::QueryExecutionFailed(e.to_string())
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
                DatabaseError::QueryExecutionFailed(e.to_string())
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
            DatabaseError::QueryExecutionFailed(e.to_string())
        })?;

        let output_json: serde_json::Value = serde_json::from_str(&output_str).map_err(|e| {
            error!(error = ?e, "Failed to parse output as JSON");
            DatabaseError::QueryExecutionFailed(e.to_string())
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
