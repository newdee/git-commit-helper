# Git Commit Helper

## Overview
Git Commit Helper is a practical tool that leverages large language models (LLMs) to analyze changes in a Git repository. It helps users generate meaningful commit messages, addressing the difficulties users may encounter when writing commit messages. At the same time, it provides a user - friendly command - line interaction experience.


## Install
```
cargo install git-commit-helper
```

## Usage
- Set `OPENAI_BASE_URL`(Optional),`OPENAI_API_KEY`(Requires) in your environment.
- Use this command after your `git add` command.
```
Usage: git-commit-helper [OPTIONS]

Options:
  -m, --model <MODEL>          [default: gpt-4o]
      --max-token <MAX_TOKEN>  [default: 2048]
  -h, --help                   Print help
  -V, --version                Print version
```

## License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
