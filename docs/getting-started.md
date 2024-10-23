# Getting Started with zkDB

## Prerequisites

- Rust and Cargo (latest stable version)
- SP1 zkVM toolchain

## Setting up SP1

Before building the zkDB project, you need to set up the SP1 zkVM toolchain. Follow these steps:

1. Install the required dependencies:
   - Git
   - Rust (Nightly)
   - Docker

2. Install SP1 using the prebuilt binaries (recommended):

   ```
   curl -L https://sp1.succinct.xyz | bash
   ```

   Follow the on-screen instructions to make the `sp1up` command available in your CLI.

3. Run `sp1up` to install the toolchain and the `cargo prove` CLI:

   ```
   sp1up
   ```

   This will install:
   - The `succinct` Rust toolchain with support for the `riscv32im-succinct-zkvm-elf` compilation target.
   - The `cargo prove` CLI tool for compiling SP1 programs and other helper functionality.

4. Verify the installation:

   ```
   cargo prove --version
   RUSTUP_TOOLCHAIN=succinct cargo --version
   ```

For more detailed instructions or troubleshooting, refer to the [official SP1 installation guide](https://docs.succinct.xyz/getting-started/install.html).

## Building the Project

1. Clone the repository and navigate to the project directory:

   ```
   git clone <repository-url>
   cd zkdb
   ```

2. Build the project in release mode:

   ```
   cargo build --release
   ```

   *Note: It's crucial to use the `--release` flag when building, as sp1-sdk must be built in release mode.*

## Project Structure

- `src/main.rs`: Contains the main zkVM program logic for Merkle tree operations.
- `src/bin/merkle.rs`: Implements the command-line interface for interacting with the zkVM program.
- `tests/integration_tests.rs`: Contains integration tests for the zkVM program.
- `script/merkle_state.txt`: Stores the current state of the Merkle tree.
- `.env.example`: Example environment variables configuration.

## Running Tests

To run the integration tests:

```
cargo test
```

This will execute the tests defined in `tests/integration_tests.rs`.
