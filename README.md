# zkDB: Zero-Knowledge Database 🧠

zkDB is a powerful, privacy-preserving database system that implements the Verifiable Database (VDB) Framework. It combines Merkle trees with zero-knowledge proofs using the SP1 zkVM. Designed for developers building trustless applications, zkDB offers verifiable data operations without compromising privacy.

## Key Features

- **Zero-Knowledge Proofs**: Prove data operations without revealing content
- **Complex Queries**: Go beyond simple inclusion proofs
- **Full Verifiability**: Every action (insert, query, prove) generates a proof
- **Stateless Design**: Entire database state can be serialized
- **zkVM Integration**: Allows for advanced computations while maintaining verifiability

## Why zkDB?

zkDB addresses the growing need for off-chain services in blockchain development, offering a solution that extends beyond the limitations of traditional blockchain VMs. It's positioned at the forefront of the trend towards hybrid systems that leverage both on-chain and off-chain components.

[Learn more about why zkDB matters](docs/why.md)

## Understanding zkDB

- [What is the VDB Framework?](docs/what-is-vdb-framework.md)
- [How zkDB implements the VDB Framework](docs/why.md#how-zkdb-implements-the-vdb-framework)
- [zkDB Data Flow](docs/why.md#zkdb-data-flow)

## Quick Example

Here's a simple example of how to use zkDB:

```bash
# Insert a key-value pair
cargo run --release --bin merkle -- insert user123 "John Doe"

# Query a value
cargo run --release --bin merkle -- query user123

# Generate a proof
cargo run --release --bin merkle -- prove user123
```

## Documentation

- [Getting Started Guide](docs/getting-started.md)
- [Usage Guide](docs/usage.md)

## Benchmarks

Check out our [usage guide](docs/usage.md#benchmark-results) for the latest performance benchmarks.

## Project Structure

- `src/main.rs`: Contains the main zkVM program logic for Merkle tree operations.
- `src/bin/merkle.rs`: Implements the command-line interface for interacting with the zkVM program.
- `tests/integration_tests.rs`: Contains integration tests for the zkVM program.
- `script/merkle_state.txt`: Stores the current state of the Merkle tree.

For more details, see our [Getting Started Guide](docs/getting-started.md#project-structure).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT
