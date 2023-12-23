# Checkmark

Checkmark is a Command Line Interface (CLI) tool that helps maintain high-quality Markdown documentation. It checks for formatting, grammatical, and spelling errors, as well as broken links.

## Features

- **Formatting**: Checks and corrects Markdown file formatting.
- **Grammar**: Checks the document for grammatical errors. Requires internet connection and OPEN_AI_API_KEY environment variable (.dotenv file is supported).
- **Review**: Reviews the document using OpenAI's API. Requires internet connection and OPEN_AI_API_KEY environment variable (.dotenv file is supported).
- **Links**: Checks the document for broken links (both web and local).
- **Spelling**: Checks the document for spelling errors.

## Installation

Provide instructions on how to install your tool here.

## Usage

```bash
checkmark [OPTIONS] [PROJECT_ROOT] <COMMAND>
```

## Commands

- `fmt`: The `fmt` command is a tool for checking and correcting Markdown file formatting. It ensures that your Markdown files adhere to a consistent style, making them easier to read and maintain. This command can automatically fix many common formatting issues, such as inconsistent indentation, incorrect header levels, and improperly formatted lists.
- `grammar`: The `grammar` command checks the document for grammatical errors. It uses OpenAI's API to provide advanced grammar checking, helping to ensure that your documentation is clear and professional. This command requires an internet connection and the OPEN_AI_API_KEY environment variable (.dotenv file is supported). Note that this command may not catch all grammatical errors, especially in complex sentences.
- `review`: The `review` command reviews the document using OpenAI's API. It provides a high-level review of your documentation, helping to catch issues that other checks might miss. This command requires an internet connection and the OPEN_AI_API_KEY environment variable (.dotenv file is supported). The review command can provide suggestions for improving the clarity, conciseness, and tone of your documentation.
  - `-l, --language <LANGUAGE>`: Specifies the language to use for the review. Defaults to 'en' (English). This option is available with the `review` command.
- `links`: The `links` command checks the document for broken links. It checks both web and local links, ensuring that your documentation is reliable and accurate. This command can catch both 404 errors from web links and broken relative links in your local project. Note that checking web links requires an internet connection.
- `spelling`: The `spelling` command checks the document for spelling errors. It helps to catch and correct spelling mistakes, improving the quality of your documentation. This command uses a built-in dictionary to check words, and it can suggest corrections for misspelled words.
  - `-d, --dictionary <FILE_PATH>`: Specifies the path to a custom dictionary file. This allows you to add your own words that are not in the built-in dictionary. This option is available with the `spelling` command.

## Arguments

- `[PROJECT_ROOT]`: Specifies the root directory of your project, a single file, or a web URL. This is where the tool will start scanning for Markdown files. If a single file or a web URL is specified, only that will be scanned. Defaults to the current directory.

## Options

- `-s, --sarif [<FILE_PATH>]`: Outputs the report to a specified file in SARIF format. This is useful for integrating with other tools that can process SARIF. If no file is specified, the report will be saved to './report.sarif'. This option is available with all commands.
- `-c, --config <FILE_PATH>`: Specifies the path to the configuration file. This allows you to customize the behavior of the tool. If this is set, configuration files in default locations will be ignored. This option is available with all commands.
- `-v, --verbose`: Enable verbose logging. This will output more detailed information about what the tool is doing, which can be helpful for debugging. This option is available with all commands.
- `-h, --help`: Print help. This option is available with all commands.
- `-V, --version`: Print version. This option is available with all commands.

## Examples

```bash
checkmark -v . fmt
checkmark --config myconfig.toml . review
```

## Contributing

Provide instructions on how to contribute to your project here.

## License

Provide information about your project's license here.
