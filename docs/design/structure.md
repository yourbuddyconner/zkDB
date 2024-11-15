# zkDB Crate Structure

The zkDB project consists of several crates, each serving a specific purpose:

## zkdb-core

This crate provides the foundation for building SP1 Programs that implement database functionality:

- Core traits like `DatabaseEngine` for implementing SP1 programs
- Common data structures for commands and results used within SP1
- No-std compatible for SP1 runtime environment
- Serialization interfaces for SP1 program I/O
- Error types specific to SP1 program execution

## zkdb-merkle

An example implementation of a database engine as an SP1 program:

- Built using zkdb-core traits and interfaces
- Implements Merkle tree operations inside SP1
- Demonstrates how to build a database engine using zkdb-core
- Compiles to an ELF that is used by zkdb-lib

## zkdb-lib

The primary interface crate that applications will use to interact with zkDB:

- High-level API for all database operations
- Directly includes compiled SP1 program ELF binaries
- Handles state management and serialization
- Provides proof generation and verification
- Example usage:
  ```rust
  let db = Database::new(DatabaseType::Merkle);
  db.execute_query(Command::Insert { key, value })?;
  ```

## zkdb-cli

Command-line interface built on top of zkdb-lib:

- User-friendly commands for database operations
- Built entirely using zkdb-lib's public API
- Proof management utilities
- Configuration handling

## Additional Crates (future)

### zkdb-types
- Common data types for both SP1 programs and client code
- Serialization formats
- Type conversion utilities

### zkdb-utils
- Shared utility functions
- Helper methods
- Common tools

### zkdb-store

A new crate for managing value storage:

- Trait-based storage interface
- Multiple backend implementations
- Configurable storage policies
- Example usage:
  ```rust
  let store = FileStore::new(config);
  let location = store.put(key, value)?;
  let value = store.get(key)?;
  ```

### Storage Backends

1. FileStore
   - Local filesystem storage
   - Directory-based organization
   - Optional compression
## Architecture Flow

1. SP1 Program Development:
   - Use zkdb-core to implement database logic
   - Compile to ELF binary
   - Include directly in zkdb-lib

2. Client Usage:
   - Applications use zkdb-lib
   - zkdb-lib uses embedded ELF binaries
   - zkdb-lib handles all SP1 execution details

## Dependencies

The project uses different dependencies for SP1 programs vs client code:

### SP1 Program Dependencies (zkdb-core, database implementations)
- `sp1-zkvm`: Core SP1 functionality
- `serde` with no-std features
- Domain-specific libraries (e.g., rs_merkle)

### Client Dependencies (zkdb-lib, zkdb-cli)
- `sp1-sdk`: SP1 prover/verifier functionality
- Full `serde` stack
- `clap` and other user-facing utilities

## Build Process

1. SP1 Programs:
   - Implement using zkdb-core
   - Compile to ELF
   - ELF binary is included directly in zkdb-lib

2. Client Libraries:
   - Use zkdb-lib which contains the ELF binaries
   - No separate registry or loading mechanism needed

## State Management

- SP1 programs define their state format using zkdb-core
- zkdb-lib handles state serialization and management
- State is passed between operations in a format specific to each engine

## Proof System

- SP1 programs focus on computation logic
- zkdb-lib handles all proof generation and verification
- Proof artifacts are managed consistently across engines

## Storage Architecture

### Two-Layer Storage Design

1. Merkle State Tree (MST)
   - Stores key-value pairs where:
     - Key: Original key from user
     - Value: Hash of the actual data
   - Handled by zkdb-merkle SP1 program
   - Provides cryptographic proofs and consistency guarantees

2. Value Store
   - Stores actual data values
   - Multiple backend options:
     - Local filesystem
   - Not part of the zero-knowledge proofs
   - Configurable based on application needs

### Data Flow

Actors:
- Database (Main client interface in zkdb-lib)
- FileStore (Value storage backend in zkdb-store)
- ProverClient (SP1 prover interface)
- MerkleEngine (SP1 program implementation)
- MerkleState (Merkle tree state in SP1 program)
- Command (Cross-boundary message type)
- QueryResult (Cross-boundary result type)
- ProvenQueryResult (Proof-carrying result type)

1. Write Operation:
   ```mermaid
   sequenceDiagram
      participant Client
      participant DB as Database
      participant Store as FileStore
      participant SP1 as ProverClient
      participant Program as MerkleEngine
      participant MST as MerkleState

      %% Write Operation
      Client->>DB: put(key, value)
      
      %% Store actual value first
      DB->>Store: put(key, value)
      Store-->>DB: ok
      
      %% Calculate hash and update Merkle tree
      DB->>DB: hash = Sha256::digest(value)
      DB->>SP1: execute(zkdb-merkle ELF, Command::Insert{key, hash})
      
      SP1->>Program: main(state, Command::Insert)
      Program->>Program: deserialize_state()
      Program->>MST: insert(key, hash)
      MST-->>Program: ok
      Program->>Program: serialize_state()
      Program-->>SP1: QueryResult{data: {"inserted": true}, new_state}
      
      SP1-->>DB: QueryResult
      DB-->>Client: ok
   ```

2. Get Operation:
   ```mermaid
   sequenceDiagram
      participant Client
      participant DB as Database
      participant Store as FileStore
      participant SP1 as ProverClient
      participant Program as MerkleEngine
      participant MST as MerkleState

      Client->>DB: get(key)
      DB->>SP1: execute(zkdb-merkle ELF, Command::Query)
      SP1->>Program: main(state, Command::Query)
      Program->>MST: query(key)
      MST-->>Program: value_hash
      Program-->>SP1: QueryResult{data, new_state}
      SP1-->>DB: value_hash
      DB->>Store: get(key)
      Store-->>DB: value
      DB->>DB: verify_hash(value, value_hash)
      DB-->>Client: value_hash
   ```

3. Prove Operation:
   ```mermaid
   sequenceDiagram
      participant Client
      participant DB as Database
      participant SP1 as ProverClient
      participant Program as MerkleEngine
      participant MST as MerkleState

      Client->>DB: prove(key)
      DB->>SP1: execute(zkdb-merkle ELF, Command::Prove)
      SP1->>Program: main(state, Command::Prove)
      Program->>MST: generate_proof(key)
      MST-->>Program: proof
      Program-->>SP1: QueryResult{proof, root}
      SP1-->>DB: ProvenQueryResult
      DB-->>Client: proof
   ```
