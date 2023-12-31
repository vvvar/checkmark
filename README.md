# Checkmark

Checkmark is a CLI tool designed to streamline your Markdown workflow. It provides a suite of features including auto-formatting, linting, AI-powered document review, link checking, spell checking, and AI-assisted document composition.

## Features

Checkmark offers a range of commands to help maintain high-quality Markdown documentation:

- **fmt**: Auto-formats all Markdown files in the project, fixing common formatting issues such as trailing whitespace and inconsistent line endings.
- **links**: Check broken links in your documents, covering both web and local file links.
- **lint**: Runs a linter (port of [markdownlint](https://github.com/DavidAnson/markdownlint), see [Roadmap](#roadmap) section for details) to ensure your Markdown files adhere to best practices.
- **review**: Uses OpenAI's API to review your documents, providing AI-powered insights and suggestions. Requires OpenAI API key.
- **compose**: Assists in composing new Markdown documents from a prompt in a context of an existing document. Powered by OpenAI. Requires OpenAI API key.
- **spelling**: Check your documents for spelling errors.
- **CI mode**: Turns off interactive prompts and outputs reports in a format suitable for CI/CD pipelines.

## Installation

Make sure you install the latest [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) version. Next, run the following command:

```bash
cargo install --git https://github.com/vvvar/checkmark.git
```

> **NOTE**: Pre-built as well as installation from crates.io is planned. See [Roadmap](#roadmap) section for details.

## Usage

Checkmark is a set of tools. Each tool has its command. Tools are:

- `fmt` - Opinionated auto-formatter and format checker.
- `links` - Link checker. Finds broken hyperlinks and mail addresses.
- `lint` - Port of [markdownlint](https://github.com/DavidAnson/markdownlint).
- `review` - AI review assistant. Reviews your documents using OpenAI's API to highlight areas that could be improved. Requires an internet connection and the OPEN_AI_API_KEY environment variable (supports .env file).
- `compose` - AI Markdown document generator. Generates a Markdown document based on user prompts and an optional context file. Requires an internet connection and the OPEN_AI_API_KEY environment variable (supports .env file).
- `spelling` - Spell checker.

Full list of commands can be displayed by running:

```bash
$ checkmark --help

A CLI tool that helps maintain high-quality Markdown documentation by checking for formatting, grammatical, and spelling errors, as well as broken links

Usage: checkmark [OPTIONS] [PROJECT_ROOT] <COMMAND>

Commands:
  fmt       Formats all Markdown files in the project. This will fix common formatting issues such as trailing whitespace, inconsistent line endings, and more
  links     Checks the document for broken links(both web and local)
  lint      Run linter
  review    Reviews the document using OpenAI's API. Requires internet connection and OPEN_AI_API_KEY environment variable(.dotenv file is supported)
  compose   Compose a file in Markdown format from a prompt
  spelling  Checks the document for spelling errors(offline)
  help      Print this message or the help of the given subcommand(s)

Arguments:
  [PROJECT_ROOT]  Sets the project root, file, or web URL for scanning Markdown files. Can also accept a Git repository. Defaults to the current directory if not specified [default: .]

Options:
      --exclude <EXCLUDE>...  List of files(wildcards) to exclude from scanning
  -c, --config <FILE_PATH>    Sets the configuration file path. Overrides default files if set
      --ci                    CI Mode: Turns off interactive prompts and outputs report in a format suitable for CI/CD pipelines
      --sarif [<FILE_PATH>]   Saves the report in SARIF format to a given file or defaults to './report.sarif' if no file is specified
      --verbose               Verbose logging: Provides detailed tool activity, useful for debugging
  -h, --help                  Print help
  -V, --version               Print version
```

## Roadmap

- [ ] Port remaining markdownlint rules
- [ ] Provide a package via crates.io
- [ ] Provide pre-built packages via `brew`, `choco` and `apt`

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).
