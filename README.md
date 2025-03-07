# Namada Monitoring

Namada Monitoring is a tool designed to track and report various metrics related to the Namada blockchain. It provides insights into the blockchain's performance, validator activity, transaction statistics, and more.

## Features

- **Block Height Counter**: Tracks the latest block height of the Namada blockchain.
- **Block Time**: Tracks the time spent processing a block.
- **Bonding Activity Metrics**: Measures the total amount of tokens bonded and unbonded per epoch.
- **Epoch Counter**: Tracks the latest epoch recorded on the blockchain.
- **Fees tracker**:  Total fees paid per block and per token.
- **Peer Count**: Tracks the number of active peers known to the node.
- **Validator Signatures counter**:  Tracks the number of validators signatures in each block
- **Validator Slashes counter**:  Tracks the number of slashes in each block
- **Total Supply of Native Token**: Monitors the total supply of Namada's native token.
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
    docker build -t namada/monitoring .
    ```
2. **Run docker image**
    ```sh
    docker run -it -p 9184:9184 --rm namada/monitoring --rpc https://rpc.namada-archive.citizenweb3.com
    ```

## Run with Composed Prometheus and Grafana

1. **Build the namada/monitor image**
    ```sh
    just build docker
    ```
2. **Start all services**
    ```sh
    just compose up
    ```
3. **Explore the dashboards/alerts**

    Grafana dashboards(admin/admin):`open http://127.0.0.1:3000`

    Prometheus database and alert rules:`open http://127.0.0.1:9090/alerts`

    Namada monitor metrics: `open http://127.0.0.1:9184/metrics`

See `composer/.env` to monitor different chains. Currently this setting only support one chain at a time:

```sh
CHAIN_ID=namada.5f5de2dd1b88cba30586420
RPC=https://rpc.namada-archive.citizenweb3.com,http://104.251.123.123:26657
```