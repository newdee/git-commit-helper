# Git Commit Helper [![GitHub stars](https://img.shields.io/github/stars/newdee/git-commit-helper.svg?style=social)](https://github.com/newdee/git-commit-helper)

## Overview
Git Commit Helper is a practical tool that leverages large language models (LLMs) to analyze changes in a Git repository. It helps users generate meaningful commit messages, addressing the difficulties users may encounter when writing commit messages. At the same time, it provides a user - friendly command - line interaction experience.

## Preview
You can use the `git-commit-helper` command directly to generate meaningful commit messages. Additionally, you can also use it within `lazygit` to quickly submit commits. Here is a preview video of using `git-commit-helper` in `lazygit`.

[![asciicast](https://asciinema.org/a/718306.svg)](https://asciinema.org/a/718306)

## Install
You can either download the corresponding package released in the `release` section or use the following command to install:

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

## 🙏 Support
If you find this project helpful, please consider giving it a ⭐️!
