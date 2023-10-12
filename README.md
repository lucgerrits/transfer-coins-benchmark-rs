# Ethereum Benchmarking Script

This script benchmarks Ethereum transaction throughput by sending transactions at a specified rate for a given duration.

## Features

- **Keypair Management**: Generate or load Ethereum key pairs.
- **Customizable Transaction Rate**: Define transactions per second (TPS) and duration for the benchmark.
- **Concurrent Transactions**: Send multiple transactions simultaneously for efficient benchmarking.
- **Detailed Results**: Get starting and ending block numbers, expected and actual TPS, and benchmark duration.

## Prerequisites

Ensure you have the following dependencies installed:

- Rust (latest stable version)
- Cargo (comes with Rust)

## Setup

1. Clone the repository:
```bash
git clone <REPO_URL>
cd <REPO_DIRECTORY>
```

2. Install required Rust dependencies:
```bash
cargo build
```

## Usage

To run the benchmark:
```bash
cargo run <TXS_PER_SECOND> <BENCHMARK_DURATION>
```

For example, to send 50 transactions per second for 60 seconds:
```bash
cargo run 50 60
```

## Notes

* The script connects to a pre-defined Ethereum node. Make sure to update the node URL if needed.
* A hardcoded private key is currently used for signing transactions. Replace if needed.
* Be cautious when using this script on a live network. Always run benchmarks on testnets first.


## Help

For help, please contact [luc].