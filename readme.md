# Git Commit Helper

## Overview
Git Commit Helper is a practical tool that leverages large language models (LLMs) and uses the `async-openai` library to analyze changes in a Git repository, helping users generate meaningful commit messages. It employs `clap` to provide a user-friendly command-line interface, and also utilizes `git2`, `reqwest`, `serde`, and `tokio` to handle Git operations, network requests, serialization, and asynchronous tasks respectively.

## Project Information
- **Name**: git-commit-helper
- **Version**: 0.1.0
- **Author**: coding@dfine.tech
- **License**: MIT
- **Edition**: 2024

## Dependencies
### Main Dependencies
- **async-openai**: 0.28.1 - Used to generate commit messages.
- **clap**: 4.5.37 (with `derive` feature) - Provides a user-friendly command-line interface.
- **git2**: 0.20.2 - Handles Git operations.
- **reqwest**: 0.12.15 (with `blocking` and `json` features) - Manages network requests.
- **serde**: 1.0.219 (with `derive` feature) - Used for serialization.
- **serde_json**: 1.0.140 - Handles JSON data.
- **tokio**: 1.45.0 (with `full` feature) - Manages asynchronous tasks.

## Usage
(Here you can add specific usage instructions for the tool, such as how to install and run it.)

## Contributing
(If you welcome contributions, you can add guidelines on how others can contribute to this project.)

## License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
