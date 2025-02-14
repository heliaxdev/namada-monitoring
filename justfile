RUST_STABLE := trim(read("rust-stable-version"))

devs:
    rustup toolchain install {{ RUST_STABLE }} --no-self-update --component clippy,rustfmt

toolchains:
    @echo {{ RUST_STABLE }}
    @echo {{ RUST_STABLE }}

build:
    cargo +{{ RUST_STABLE }} build

check:
    cargo +{{ RUST_STABLE }} check

fmt:
    cargo +{{ RUST_STABLE }} fmt --all

fmt-check:
    cargo +{{ RUST_STABLE }} fmt --all --check
    
clippy:
    cargo +{{ RUST_STABLE }} clippy

clippy-fix:
    cargo +{{ RUST_STABLE }} clippy --all --fix --allow-dirty --allow-staged

clean:
    cargo clean