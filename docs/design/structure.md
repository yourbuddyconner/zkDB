# zkDB Crate Structure

The zkDB project will consist of several crates, each serving a specific purpose. The main crates are:

## zkdb-core

This crate will serve as the base for various database engines implemented as SP1 programs. It will provide common utilities, data structures, and interfaces that can be shared across different database engines.

### zkdb-merkle

This crate will implement a Merkle tree-based database engine as an SP1 program. It will provide functionality for inserting, querying, and generating proofs for key-value pairs stored in a Merkle tree.

### zkdb-duckdb

This crate will implement a database engine based on DuckDB, a lightweight and embeddable SQL database management system. It will provide functionality for executing SQL queries and generating proofs for the query results.

### Other Database Engines

Additional crates can be added to implement other database engines as SP1 programs, such as a time-series database, a graph database, or any other specialized database system.

## zkdb-lib

This crate will serve as the primary interface for interacting with the various database engines. It will provide a high-level API for creating, querying, and managing databases, as well as generating and verifying proofs.

The `zkdb-lib` crate will act as a wrapper around the different `zkdb-core` crates, abstracting away the underlying implementation details and providing a consistent interface for working with different database engines.

## zkdb-cli

This crate will provide a command-line interface (CLI) for interacting with the zkDB system. It will allow users to create, manage, and query databases, as well as generate and verify proofs from the command line.

The `zkdb-cli` crate will depend on the `zkdb-lib` crate to interact with the underlying database engines.

## Additional Crates

Depending on the project's requirements, additional crates may be added to provide supplementary functionality, such as:

- `zkdb-types`: A crate for defining common data types and structures used across the zkDB project.
- `zkdb-utils`: A crate containing utility functions and helpers used throughout the project.
- `zkdb-tests`: A crate for housing integration and end-to-end tests for the zkDB system.

## Dependencies

The zkDB crates will depend on various external crates, including:

- `sp1-sdk`: The SP1 Software Development Kit, which provides utilities for developing and proving SP1 programs.
- `serde`, `serde_json`: For serialization and deserialization of data structures.
- `clap`: For parsing command-line arguments in the `zkdb-cli` crate.
- `tracing`, `log`: For logging and debugging purposes.
- `duckdb`: The DuckDB crate, used in the `zkdb-duckdb` crate.
- Any other crates required by the specific database engines or utilities.

This crate structure separates concerns and allows for modular development and testing of different database engines. The `zkdb-lib` crate acts as a unified interface, making it easier to work with and switch between different database engines without modifying the application code.

