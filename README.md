# checkmark - Markdown Checker

CLI tool for checking Markdown files.

## Motivation

Often I write project documentation in Markdown.
To keep it acceptable and avoid manual verification I use various quality tools such as link checker, spell checker, linter and formatting tool.
There are a lot of awesome tools available to accomplish that already.
However, I want to have a single tool that does everything so that I don't have to spend time configuring and managing all these tools for every project.

## Overview

This tool combines:

- link checker(provided by [lychee](https://github.com/lycheeverse/lychee/))
- code formatting(provided by [prettier](https://github.com/prettier/prettier))
- linter(provided by [markdownlint](https://github.com/DavidAnson/markdownlint))
- spell checker(provided by [SymSpell](https://github.com/wolfgarbe/SymSpell))
- grammar checker(provided by [Sapling AI](https://sapling.ai))

within a single CLI tool that you can use locally to ensure the quality of Markdown documentation and hook up on your CI to ensure consistency within a team.

## Installation

TBD

## Usage

Recursively check all Markdown files inside the current directory:

```sh
checkmark .
```

If all files have passed checks then status code `0` will be returned.
Otherwise, issues will be printed to `stderr` and status code `1` returned.

Recursively auto-format all Markdown files inside the current directory:

```sh
checkmark . --autoformat
```

## Configuration

See all CLI options:

```sh
$ checkmark --help

Check MarkDown files for validity, formatting, grammar and links availability(web URLs as well as file links)
Usage: checkmark [OPTIONS] [ROOT_PATH]
Arguments:
  [ROOT_PATH]
          Root of your project. Recursively search for Markdown files in this folder
          [default: .]
Options:
  -i, --ignore-url <URI>...
          Comma-separated list of URI globs(web links or files) that will be ignored by link checker. You can also provide this arguments multiple times
  -a, --autoformat
          Perform auto-formatting of files. Note: All checker flags will be ignored in this mode. Note: This will modify your files!
  -h, --help
          Print help (see a summary with '-h')
  -V, --version
          Print version
```
