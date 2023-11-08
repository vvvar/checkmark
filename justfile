set dotenv-load

[macos]
install:
    brew install protobuf

build:
    cargo build --release

check:
    cargo run -- --ignore-url=*shall-be-ignored*

autoformat:
    cargo run -- --autoformat .

help:
    cargo run -- --help