#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

pub trait DatabaseEngine {
    fn execute_query(
        &mut self,
        state: &[u8],
        command: &Command,
    ) -> Result<QueryResult, DatabaseError>;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Command {
    Query { key: String },
    Prove { key: String },
    Insert { key: String, value: String },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct QueryResult {
    pub data: serde_json::Value,
    pub new_state: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DatabaseError {
    QueryExecutionFailed(String),
}
