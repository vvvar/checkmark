# Full project workflow. Run this before making PR.
default: clean fmt build test audit lint coverage install

# Build project. Additional arguments will be passed as-is to the "cargo build".
[group('build')]
build *CARGO_BUILD_ARGS:
    cargo install cargo-auditable --locked
    cargo auditable build --workspace --all-targets {{ CARGO_BUILD_ARGS }}

# Run tests. Additional arguments will be passed as-is to the "cargo nextest run".
[group('test')]
test *CARGO_NEXTEST_ARGS:
    cargo install cargo-nextest --locked
    cargo nextest run --workspace --all-targets --all-features {{ CARGO_NEXTEST_ARGS }}

# Install checkmark CLI to a system and make it available via terminal.
[group('test')]
install:
    cargo install --path src/checkmark_cli --locked

# Perform audit of the dependency tree.
[group('code quality')]
audit:
    cargo install cargo-audit cargo-deny cargo-udeps --locked
    cargo audit
    cargo deny check
    cargo udeps --all-targets

# Generate code coverage report.
[group('code quality')]
coverage:
    cargo install cargo-tarpaulin --locked
    cargo tarpaulin --out Html --output-dir target/tarpaulin

# Auto-format source code.
[group('code quality')]
fmt:
    cargo fmt
    taplo fmt
    checkmark fmt
    just --unstable --fmt

# Run various linters against source code.
[group('code quality')]
lint: install
    cargo install taplo-cli --locked
    cargo fmt --all --check
    taplo fmt --check --diff
    checkmark fmt --check
    checkmark lint
    checkmark spellcheck
    checkmark linkcheck
    just --unstable --fmt --check   
    cargo clippy --workspace --all-targets --locked

# Cleanup project artifacts & tmp files.
[group('other')]
clean:
    cargo clean
