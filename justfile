default:
    @just --list

run:
    cargo run -p editor

fmt:
    cargo fmt

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo test --all-features
