RUST_STABLE := "1.82.0"
RUST_NIGTHLY := "nightly-2025-02-05"

devs:
    rustup toolchain install {{ RUST_STABLE }} --no-self-update --component clippy,rustfmt
    rustup toolchain install {{ RUST_NIGTHLY }} --no-self-update --component clippy,rustfmt

build target="":
    cargo build
    if [[ "w{{target}}" == "wdocker" ]]; then docker build -t namada/monitoring . ; fi


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

