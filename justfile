[macos]
install:
    brew install protobuf

build:
    cargo build

run:
    cargo run -- -r "."