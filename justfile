devs:
    rustup toolchain install 1.79.0 --no-self-update --component clippy,rustfmt
    rustup toolchain install nightly-2024-06-14 --no-self-update --component clippy,rustfmt

build:
    cargo build

check:
    cargo check

fmt:
    cargo +nightly-2024-06-14 fmt --all

fmt-check:
    cargo +nightly-2024-06-14 fmt --all --check
    
clippy:
    cargo clippy

clippy-fix:
    cargo clippy --all --fix --allow-dirty --allow-staged

clean:
    cargo clean