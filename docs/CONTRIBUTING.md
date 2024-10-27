# How to contribute?

## Setting up dev environment

- Install [rust](https://rustup.rs).
- Install [nextest](https://nexte.st/docs/installation/from-source/).
- Install [just](https://github.com/casey/just).
- Install [taplo](https://taplo.tamasfe.dev/cli/installation/cargo.html).
- Install [checkmark](../#installation).

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

Run in your terminal:

```sh
just lint
```

## Git Workflow

Branching model: [Trunk Based Development](https://trunkbaseddevelopment.com).
Commit message convention: [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/).
Branch name convention: Use same prefix as for the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/), use short descriptive name of the branch, for ex. `feat/add-support-of-new-rule`.
