# How to contribute?

## Setting up dev environment

- Install [rust](https://rustup.rs).
- Install [nextest](https://nexte.st/docs/installation/from-source/).
- Install [just](https://github.com/casey/just).
- Install [taplo](https://taplo.tamasfe.dev/cli/installation/cargo.html).
- Install [checkmark](../#installation) (we check ourselves with our own tool on a CI).

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

## Known Issues & Workarounds

### Error compiling `rust-openssl` on Windows

If you see this error:

```sh
Can't locate Locale/Maketext/Simple.pm in @INC (you may need to install the Locale::Maketext::Simple module)
```

then it means that `perl` needs additional configuration. Have a look how it is set up in [GitHub Workflow](../.github/workflows/pr.yml). More information [here](https://github.com/sfackler/rust-openssl/issues/2149#issuecomment-2014064057).
