# Namada Monitoring

Namada Monitoring is a tool designed to track and report various metrics related to the Namada blockchain. It provides insights into the blockchain's performance, validator activity, transaction statistics, and more.

## Features

- **Block Height Counter**: Tracks the latest block height of the Namada blockchain.
- **Block Time**: Tracks the time spent processing a block.
- **Bonding Activity Metrics**: Measures the total amount of tokens bonded and unbonded per epoch.
- **Epoch Counter**: Tracks the latest epoch recorded on the blockchain.
- **Fees tracker**:  Total fees paid per block and per token.
- **Peer Count**: Tracks the number of active peers known to the node.
- **Total Supply of Native Token**: Monitors the total supply of Namada's native token.
- **Transaction Metrics**: Tracks transaction activity, including batch sizes and transaction types per epoch.
- **Transfers amounts**: Tracks the total transfer amount per token and epoch.
- **Voting Power Metrics**: Tracks the number of validators required to reach 1/3 and 2/3 of the total voting power.


## Prerequisites

- Rust (stable and nightly toolchains)
- Docker (optional, for containerized deployment)

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

3. **Build the project**:
    ```sh
    just build
    ```

## Running the Project

To run the project, use the following command:

```sh
cargo run -- --rpc <vector of rpc urls>
```

## Run with Docker

1. **Build docker image**
    ```sh
    docker build -t namada-monitoring .
    ```
2. **Run docker image**
    ```sh
    docker run -it -p 9184:9184 --rm eca6f5e0be68 --rpc https://rpc.namada-archive.citizenweb3.com
    ```
