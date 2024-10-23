# SP1 Time Series Analysis and Forecasting Project

This project demonstrates how to use [SP1](https://github.com/succinctlabs/sp1) to perform time series analysis and forecasting within a zero-knowledge proof system. It showcases the ability to process time series data, calculate statistical measures, and generate forecasts, all while maintaining privacy and verifiability.

## What Does This Project Do?

This project implements a time series analysis and forecasting system using SP1, which allows for:

1. Processing time series data (timestamps and corresponding values)
2. Calculating statistical measures such as mean, median, and standard deviation
3. Computing moving averages and exponential moving averages
4. Performing simple exponential smoothing for forecasting
5. Generating zero-knowledge proofs of these computations

## Why is it Useful?

1. **Privacy-Preserving Analytics**: Perform time series analysis without revealing the underlying data.
2. **Verifiable Forecasts**: Generate forecasts that can be verified without exposing the model or historical data.
3. **On-Chain Analytics**: Enable complex time series computations that can be verified on-chain, opening up possibilities for DeFi applications, prediction markets, and more.
4. **Data Integrity**: Ensure the integrity of time series data and computations through zero-knowledge proofs.

## Requirements

- [Rust](https://rustup.rs/)
- [SP1](https://docs.succinct.xyz/getting-started/install.html)

## Running the Project

### Build the Program

To build the program, run:
To build the program, run the following command:

```sh
cd program
cargo prove build
```

### Execute the Program

To run the program without generating a proof:

```sh
cd script
cargo run --release -- --execute
```

This will execute the program and display the output.

### Generate a Core Proof

To generate a core proof for your program:

```sh
cd script
cargo run --release -- --prove
```

### Generate an EVM-Compatible Proof

> [!WARNING]
> You will need at least 128GB RAM to generate a Groth16 or PLONK proof.

To generate a proof that is small enough to be verified on-chain and verifiable by the EVM:

```sh
cd script
cargo run --release --bin evm -- --system groth16
```

this will generate a Groth16 proof. If you want to generate a PLONK proof, run the following command:

```sh
cargo run --release --bin evm -- --system plonk
```

These commands will also generate fixtures that can be used to test the verification of SP1 zkVM proofs
inside Solidity.

### Retrieve the Verification Key

To retrieve your `programVKey` for your on-chain contract, run the following command:

```sh
cargo prove vkey --elf elf/riscv32im-succinct-zkvm-elf
```

## Using the Prover Network

We highly recommend using the Succinct prover network for any non-trivial programs or benchmarking purposes. For more 
information, see the [setup guide](https://docs.succinct.xyz/generating-proofs/prover-network.html).

To get started, copy the example environment file:

```sh
cp .env.example .env
```

Then, set the `SP1_PROVER` environment variable to `network` and set the `SP1_PRIVATE_KEY`
environment variable to your whitelisted private key.

For example, to generate an EVM-compatible proof using the prover network, run the following
command:

```sh
SP1_PROVER=network SP1_PRIVATE_KEY=... cargo run --release --bin evm
```