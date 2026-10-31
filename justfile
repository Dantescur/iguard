default:
    @just --list

fmt:
    cargo fmt --check

check:
    cargo check --locked

test:
    cargo test --locked

build:
    cargo build --release --locked

run:
    cargo run --locked

verify: fmt check test
