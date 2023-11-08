set dotenv-load

[macos]
install:
    brew install protobuf

build:
    cargo build --release

check:
    cargo run -- --ignore-url=*does-not-exist* --ignore-url=http://exaaaaaampleee.com

autoformat:
    cargo run -- --autoformat .

help:
    cargo run -- --help