[macos]
install:
    brew install protobuf

build:
    cargo build

check:
    cargo run -- --root "."

autoformat:
    cargo run -- --root "." --autoformat

help:
    cargo run -- --root "." --help