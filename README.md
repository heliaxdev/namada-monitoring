# Namada Monitoring

Namada Monitoring is a tool designed to track and report various metrics related to the Namada blockchain. It provides insights into the blockchain's performance, validator activity, transaction statistics, and more.

## Features

- **Block Height Counter**: Tracks the latest block height of the Namada blockchain.
- **Block Time**: Tracks the time spent processing a block.
- **PoS Activity Metrics**: Measures the total amount of tokens bonded and unbonded per epoch.
- **Epoch Counter**: Tracks the latest epoch recorded on the blockchain.
- **Fees tracker**:  Total fees paid per block and per token.
- **Validator Signatures counter**:  Tracks the number of validators signatures in each block
- **Validator Slashes counter**:  Tracks the number of slashes in each block
- **Total Supply of Token**: Monitors the total supply of tokens.
- **Transaction Metrics**: Tracks transaction activity, including batch sizes and transaction types per epoch.
- **Transfers amounts**: Tracks the total transfer amount per token and epoch.
- **Voting Power Metrics**: Tracks the number of validators required to reach 1/3 and 2/3 of the total voting power.

## Prerequisites

- Rust (stable and nightly toolchains)
- Docker (optional, for containerized deployment)
- Just (optional, for easier building)

## Installation

1. **Clone the repository**:
    ```sh
    git clone https://github.com/your-repo/namada-monitoring.git
    cd namada-monitoring
    ```

2. **Install Rust toolchains**:
    ```sh
    just devs
    ```

3. **Copy and modify config.toml.example**

4. **Run the project**:
    ```sh
    just run $CHAIN_ID $RPC 
    ```