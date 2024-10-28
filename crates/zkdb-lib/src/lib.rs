use sp1_sdk::{
    HashableKey, ProverClient, SP1ProofWithPublicValues, SP1ProvingKey, SP1PublicValues, SP1Stdin,
    SP1VerifyingKey,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use zkdb_merkle::get_elf;

// reexport zkdb_core
pub use zkdb_core::{Command, QueryResult};

pub struct Database {
    elf: &'static [u8],
    state: Vec<u8>,
    executor: SP1Executor,
}

impl Database {
    pub fn new(initial_state: Vec<u8>) -> Self {
        let elf = get_elf();
        Database {
            elf,
            state: initial_state,
            executor: SP1Executor::new(&elf),
        }
    }

    pub fn get_elf(&self) -> &'static [u8] {
        self.elf
    }

    pub fn execute_query(
        &mut self,
        command: Command,
        generate_proof: bool,
    ) -> Result<QueryResult, DatabaseError> {
        let result = self
            .executor
            .execute_query(&self.state, &command, generate_proof)?;
        self.state.clone_from(&result.new_state);
        Ok(result)
    }

    pub fn verify_proof(&self, proof: &ProvenOutput) -> Result<bool, DatabaseError> {
        self.executor.verify_proof(proof)
    }

    pub fn get_state(&self) -> &[u8] {
        &self.state
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
    pub fn new(elf: &'static [u8]) -> Self {
        let client = ProverClient::new();
        let (pk, vk) = client.setup(elf);
        SP1Executor {
            client,
            elf,
            pk,
            vk,
        }
    }

    pub fn execute_query(
        &self,
        state: &[u8],
        command: &Command,
        generate_proof: bool,
    ) -> Result<QueryResult, DatabaseError> {
        let mut stdin = SP1Stdin::new();
        stdin.write(&state.to_vec());
        stdin.write(
            &serde_json::to_vec(command)
                .map_err(|e| DatabaseError::QueryExecutionFailed(e.to_string()))?,
        );
        stdin.write(&[generate_proof as u8]);

        if generate_proof {
            let proof = self
                .client
                .prove(&self.pk, stdin.clone())
                .run()
                .map_err(|e| DatabaseError::ProofGenerationFailed(e.to_string()))?;

            let (output, _) = self
                .client
                .execute(self.elf, stdin.clone())
                .run()
                .map_err(|e| DatabaseError::QueryExecutionFailed(e.to_string()))?;

            self.parse_output(
                output,
                Some(ProvenOutput {
                    proof_data: proof,
                    vk: self.vk.bytes32().as_bytes().to_vec(),
                }),
            )
        } else {
            let (output, _) = self
                .client
                .execute(self.elf, stdin)
                .run()
                .map_err(|e| DatabaseError::QueryExecutionFailed(e.to_string()))?;
            // No proof is generated, so we don't need to pass it to the parser.
            self.parse_output(output, None)
        }
    }

    fn parse_output(
        &self,
        output: SP1PublicValues,
        proof: Option<ProvenOutput>,
    ) -> Result<QueryResult, DatabaseError> {
        let output_str = String::from_utf8(output.to_vec())
            .map_err(|e| DatabaseError::QueryExecutionFailed(e.to_string()))?;
        let output_json: serde_json::Value = serde_json::from_str(&output_str)
            .map_err(|e| DatabaseError::QueryExecutionFailed(e.to_string()))?;

        let data = output_json["result"].clone();

        let new_state = output_json["state"]
            .as_str()
            .ok_or_else(|| {
                DatabaseError::QueryExecutionFailed("Missing state in output".to_string())
            })?
            .as_bytes()
            .to_vec();

        // If a proof is provided, verify it.
        if let Some(proof) = proof {
            self.verify_proof(&proof)?;
        }

        Ok(QueryResult { data, new_state })
    }

    pub fn verify_proof(&self, proof: &ProvenOutput) -> Result<bool, DatabaseError> {
        self.client
            .verify(&proof.proof_data, &self.vk)
            .map(|_| true)
            .map_err(|e| DatabaseError::ProofVerificationFailed(e.to_string()))
    }
}
