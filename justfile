# Build project. All additional arguments will be passed as-is to the "cargo build".
[group('build')]
build *CARGO_BUILD_ARGS:
    cargo build --workspace --all-targets {{CARGO_BUILD_ARGS}}

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
    cargo clippy --workspace --all-targets
    cargo fmt --all --check
    cargo audit
    taplo fmt --check --diff
    checkmark fmt --check
    checkmark lint
    checkmark spellcheck
    checkmark linkcheck

# Setup dev tools.
[group('other')]
setup:
    cargo install cargo-nextest --locked
    cargo install taplo-cli --locked
    cargo install --path src/checkmark_cli --locked
    cargo install cargo-audit --locked

# Install checkmark CLI to a system. Use it to check how tool works on end-user system.
[group('other')]
install:
    cargo install --path src/checkmark_cli --locked

# Cleanup project artifacts & tmp files.
[group('other')]
clean:
    cargo clean