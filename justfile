# Build project. All additional arguments will be passed as-is to the "cargo build".
[group('build')]
build *CARGO_BUILD_ARGS:
    cargo install cargo-auditable --locked
    cargo auditable build --workspace --all-targets {{CARGO_BUILD_ARGS}}

# Run tests. All additional arguments will be passed as-is to the "cargo nextest run".
[group('test')]
test *CARGO_NEXTEST_ARGS:
    cargo install cargo-nextest --locked
    cargo nextest run --workspace --all-targets --all-features {{CARGO_NEXTEST_ARGS}}

# Install checkmark CLI to a system. Use it to check how tool works on end-user system.
[group('test')]
install:
    cargo install --path src/checkmark_cli --locked

# Auto-format project source code.
[group('code quality')]
fmt:
    cargo fmt --all
    taplo fmt
    checkmark fmt

# Run code quality checks against project source code.
[group('code quality')]
check: install
    cargo install taplo-cli cargo-audit cargo-deny --locked
    cargo fmt --all --check
    taplo fmt --check --diff
    checkmark fmt --check
    checkmark lint
    checkmark spellcheck
    checkmark linkcheck
    cargo audit
    cargo deny check
    cargo clippy --workspace --all-targets --locked

# Generate code coverage report.
[group('code quality')]
coverage:
    cargo install cargo-tarpaulin --locked
    cargo tarpaulin --out Html --output-dir target/tarpaulin

# Cleanup project artifacts & tmp files.
[group('other')]
clean:
    cargo clean