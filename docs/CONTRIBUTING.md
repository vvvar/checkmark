# How to contribute?

## Setting up dev environment

Essentials:

- Install [rust](https://rustup.rs).
- Install [just](https://github.com/casey/just).

## Building this project

Run in your terminal:

```sh
just build
```

## Running tests

Run in your terminal:

```sh
just test
```

## Running code quality checks

### Audit project dependencies

Run in your terminal:

```sh
just audit
```

### Generate code coverage report

Run in your terminal:

```sh
just coverage
```

This will generate HTML code coverage report in the `target/tarpaulin`.

### Lint

Run in your terminal:

```sh
just lint
```

## Checking how tool will work on the end-user machine

Run in your terminal:

```sh
just install
```

This will install checkmark on your PC.

## Git Workflow

Branching model: [Trunk Based Development](https://trunkbaseddevelopment.com).
Commit message convention: [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).
Branch name convention: Use same prefix as for the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/), use short descriptive name of the branch, for ex. `feat/add-support-of-new-rule`.

## Known Issues & Workarounds

### Error compiling `rust-openssl` on Windows

If you see this error:

```sh
Can't locate Locale/Maketext/Simple.pm in @INC (you may need to install the Locale::Maketext::Simple module)
```

then it means that `perl` needs additional configuration. Have a look how it is set up in [GitHub Workflow](../.github/workflows/pr.yml). More information [here](https://github.com/sfackler/rust-openssl/issues/2149#issuecomment-2014064057).
