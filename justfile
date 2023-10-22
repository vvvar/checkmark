[macos]
install:
    brew install protobuf

build:
    cargo build

check:
    cargo run -- .

autoformat:
    cargo run -- --autoformat .

help:
    cargo run -- --help