RUST_STABLE := "1.85.1"
RUST_NIGTHLY := "nightly"

devs:
    rustup toolchain install {{ RUST_STABLE }} --no-self-update --component clippy,rustfmt
    rustup toolchain install {{ RUST_NIGTHLY }} --no-self-update --component clippy,rustfmt

build:
    cargo build

run CHAIN_ID RPC CONFIG_PATH="config.toml":
    cargo run --release -- --chain-id {{CHAIN_ID}} --rpc {{RPC}} --config-path {{CONFIG_PATH}}

check:
    cargo check

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check
    
clippy:
    cargo clippy

clippy-fix:
    cargo clippy --all --fix --allow-dirty --allow-staged

clean:
    cargo clean

compose recipe="up":
    cd composer && just {{recipe}}
    #if [[ "w{{recipe}}" == "wup" ]]; then just build docker; fi

