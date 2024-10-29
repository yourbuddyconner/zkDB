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
