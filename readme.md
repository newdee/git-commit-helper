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

This tool supports **OpenAI, Anthropic, or Ollama** as the language model provider.  
**Configure only one provider at a time.**

### Environment Variables

- **For OpenAI** (`-p openai`, the default):
  - `OPENAI_API_KEY` (**Required**)
  - `OPENAI_BASE_URL` (*Optional*, defaults to OpenAI's official endpoint)

- **For Anthropic** (`-p anthropic`):
  - `ANTHROPIC_API_KEY` (**Required**)
  - `ANTHROPIC_BASE_URL` (*Optional*, defaults to `https://api.anthropic.com/v1`)

- **For Ollama** (`-p ollama`):
  - `OLLAMA_BASE_URL` (*Optional*, defaults to `http://localhost:11434`)

When `--model` is omitted, a sensible default is chosen per provider
(openai: `gpt-4o`, anthropic: `claude-opus-4-8`, ollama: `llama3.2`).

### Large diffs

If a staged diff is larger than `--chunk-size` bytes (default `200000`), it is split on
file boundaries and each part is summarized separately before the commit message is
generated. The parts are summarized concurrently, so large diffs stay fast.

Modern models have much larger context windows (e.g. Claude's 1M tokens), so if you use a
long-context model you can raise `--chunk-size` to fit the whole diff in a single request
and skip summarization entirely — often producing a more accurate message. The default is
kept conservative so it stays within `gpt-4o`'s context window out of the box.

### Example Command

Use this tool after running `git add`:

```
Usage: git-commit-helper [OPTIONS]

Options:
  -p, --provider <PROVIDER>      [default: openai]  (openai | anthropic | ollama)
  -m, --model <MODEL>            [default: per-provider — see above]
      --gpgsign
      --gpgsignkey <GPGSIGNKEY>  [default: ]
      --max-token <MAX_TOKEN>    [default: 2048]
      --chunk-size <CHUNK_SIZE>  [default: 200000]
  -h, --help                     Print help
  -V, --version                  Print version
```

## License
This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## 🙏 Support
If you find this project helpful, please consider giving it a ⭐️!
