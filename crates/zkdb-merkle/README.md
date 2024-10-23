# zkvm-merkle

This project implements a Merkle tree-based database using SP1 zkVM. It supports insert, query, and prove operations.

## Prerequisites

- Rust and Cargo (latest stable version)
- SP1 zkVM toolchain

## Building the Project

1. Clone the repository and navigate to the project directory:

   ```
   git clone <repository-url>
   cd zkdb-merkle
   ```

2. Build the project in release mode:

   ```
   cargo build --release
   ```

   Note: It's crucial to use the `--release` flag when building, as sp1-sdk must be built in release mode.

## Using the zkvm-merkle Script

The zkvm-merkle script provides a command-line interface to interact with the Merkle tree database. Here's how to use it:

1. Ensure you're in the project root directory.

2. Run the script using cargo in release mode:

   ```
   cargo run --release --bin merkle -- <command> [arguments]
   ```

   Replace `<command>` with one of the following: `insert`, `query`, or `prove`.

   Remember to always use the `--release` flag when running the script.

### Commands

#### Insert

To insert a key-value pair:

```
cargo run --release --bin merkle -- insert <key> <value>
```

Example:
```
cargo run --release --bin merkle -- insert mykey myvalue
```

#### Query

To query a value by key:

```
cargo run --release --bin merkle -- query <key>
```

Example:
```
cargo run --release --bin merkle -- query mykey
```

#### Prove

To generate a proof for a key:

```
cargo run --release --bin merkle -- prove <key>
```

Example:
```
cargo run --release --bin merkle -- prove mykey
```

### Generating SP1 Proofs

To generate and verify an SP1 proof along with any command, add the `--prove` flag:

```
cargo run --release --bin merkle -- <command> [arguments] --prove
```

Examples:
```
cargo run --release --bin merkle -- insert mykey myvalue --prove
cargo run --release --bin merkle -- query mykey --prove
cargo run --release --bin merkle -- prove mykey --prove
```

These commands will execute the respective operations and also generate and verify an SP1 proof.

### State Management

The script automatically manages the state of the Merkle tree. The state is passed between operations as a base64-encoded JSON string. This allows for stateless execution of the zkVM program while maintaining continuity between commands.

## Project Structure

- `src/main.rs`: Contains the main zkVM program logic for Merkle tree operations.
- `src/bin/merkle.rs`: Implements the command-line interface for interacting with the zkVM program.
- `tests/integration_tests.rs`: Contains integration tests for the zkVM program.

## Running Tests

To run the integration tests:

```
cargo test
```

This will execute the tests defined in `tests/integration_tests.rs`.

## Implementation Details

- The project uses the `rs_merkle` crate for Merkle tree operations.
- The `sp1-zkvm` crate is used for zkVM-specific functionality.
- State is serialized and deserialized using `serde_json` and `base64` encoding.

## Note

This project is a demonstration of using SP1 zkVM for Merkle tree operations. It's not intended for production use without further security audits and optimizations.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

[Insert your license information here]
