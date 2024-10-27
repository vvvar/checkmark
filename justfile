# Build project. All additional arguments will be passed as-is to the "cargo build".
[group('build')]
build *CARGO_BUILD_ARGS:
    cargo build --workspace --all-targets {{CARGO_BUILD_ARGS}}

# Cleanup build artifacts & tmp files.
[group('build')]
clean:
    cargo clean

# Run tests. All additional arguments will be passed as-is to the "cargo nextest run".
[group('test')]
test *CARGO_NEXTEST_ARGS:
    cargo nextest run --workspace --all-targets --all-features {{CARGO_NEXTEST_ARGS}}

# Auto-format project source code.
[group('code quality')]
fmt:
    cargo fmt --all
    taplo fmt
    checkmark fmt

# Run code quality checks against project source code.
[group('code quality')]
lint:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets
    taplo fmt --check --diff
    checkmark fmt --check
    checkmark lint
    checkmark spellcheck
    checkmark linkcheck
