set dotenv-load

[macos]
install:
    brew install protobuf

build:
    cargo build --release

check:
    RUST_LOG=info cargo run

autoformat:
    cargo run -- --autoformat

help:
    cargo run -- --help