# Using zkDB

## Command-Line Interface

The zkDB script provides a command-line interface to interact with the Merkle tree database.

### Basic Usage

Run the script using Cargo in release mode:

```
cargo run --release --bin merkle -- <command> [arguments]
```

Replace `<command>` with one of the following: `insert`, `query`, or `prove`.

*Remember to always use the `--release` flag when running the script.*

### Commands

#### Insert

To insert a key-value pair:

```
cargo run --release --bin merkle -- insert <key> <value>
```

**Example:**

```
cargo run --release --bin merkle -- insert mykey myvalue
```

#### Query

To query a value by key:

```
cargo run --release --bin merkle -- query <key>
```

**Example:**

```
cargo run --release --bin merkle -- query mykey
```

#### Prove

To generate a proof for a key:

```
cargo run --release --bin merkle -- prove <key>
```

**Example:**

```
cargo run --release --bin merkle -- prove mykey
```

### Generating SP1 Proofs

To generate and verify an SP1 proof along with any command, add the `--prove` flag:

```
cargo run --release --bin merkle -- <command> [arguments] --prove
```

**Examples:**

```
cargo run --release --bin merkle -- insert mykey myvalue --prove
cargo run --release --bin merkle -- query mykey --prove
cargo run --release --bin merkle -- prove mykey --prove
```

These commands will execute the respective operations and also generate and verify an SP1 proof.

### State Management

The script automatically manages the state of the Merkle tree. The state is passed between operations as a base64-encoded JSON string. This allows for stateless execution of the zkVM program while maintaining continuity between commands.

## Implementation Details

- The project uses the `rs_merkle` crate for Merkle tree operations.
- The `sp1-zkvm` crate is used for zkVM-specific functionality.
- State is serialized and deserialized using `serde_json` and `base64` encoding.

## Benchmark Results

Here are the benchmark results for the main operations:

```
+-----------+--------+--------------+-------------+
| Operation | Cycles | Total Time   | Avg Time    |
+-----------+--------+--------------+-------------+
| insert    | 44453  | 1.076435122s | 10.764351ms |
+-----------+--------+--------------+-------------+
| query     | 19995  | 1.038018872s | 10.380188ms |
+-----------+--------+--------------+-------------+
| prove     | 20024  | 1.059557666s | 10.595576ms |
+-----------+--------+--------------+-------------+
```

These results show the number of cycles, total time, and average time for each operation over 100 iterations.

## Note

This project is a demonstration of using SP1 zkVM for Merkle tree operations. It's not intended for production use without further security audits and optimizations.
