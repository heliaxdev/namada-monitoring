build:
    cargo build

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