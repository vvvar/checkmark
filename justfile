set dotenv-load

[macos]
install:
    brew install protobuf

release:
    cargo build --release

check:
    RUST_LOG=info cargo run -- /Users/vvoinov/Documents/repos/md-checker/tests/data/lists.md

autoformat:
    RUST_LOG=info cargo run -- $(pwd) --autoformat

test:
    RUST_LOG=info cargo test

help:
    RUST_LOG=info cargo run -- --help