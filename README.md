# [ai\_messenger][repository-github-url]

> A Rust-based multi-provider AI messaging service. 🤖

[![“Build” workflow status](https://github.com/ChristianGrete/ai\_messenger/actions/workflows/build.yml/badge.svg)](https://github.com/ChristianGrete/ai\_messenger/actions/workflows/build.yml)
[![“Check” workflow status](https://github.com/ChristianGrete/ai\_messenger/actions/workflows/check.yml/badge.svg)](https://github.com/ChristianGrete/ai\_messenger/actions/workflows/check.yml)
[![Buy Me a Coffee](https://img.shields.io/badge/Buy\_me\_a\_coffee-222?logo=buymeacoffee\&logoColor=222\&labelColor=fd0)](https://buymeacoffee.com/christiangrete)

**ai\_messenger** is a Rust-based web service designed to communicate with various AI providers. It prioritizes privacy and uses an adapter-based design to seamlessly integrate different vendors. Its API is UX-focused, modeled after popular messaging platforms like _WhatsApp_, _Signal_, and _Messages_.

## Getting started

### Installation

Clone this repository and run:

```sh
cargo install --path .
```

Then, check if the binary is installed:

```sh
ai_messenger help # Print the CLI usage
```

### Usage

Start the service by running:

```sh
ai_messenger serve # Start the API server
```

### Configuration

ai_messenger uses a TOML configuration file. It searches for config files in this order:

1. `./ai_messenger.toml` (current directory)
2. `~/.ai_messenger.toml` (home directory)
3. Platform-specific location (e.g., `~/Library/Preferences/com.christiangrete.ai_messenger.toml` on macOS)

You can also specify a custom config file:

```sh
ai_messenger serve --config path/to/custom.toml
```

## License

This project is licensed under **MIT-NC** (MIT _Non-Commercial_ License).

- Free for personal and non-commercial use.
- Commercial use requires a separate license from the copyright holder.

See the [LICENSE][repository-license-url] file for full terms.

---

Copyright © 2025 ([MIT-NC][repository-license-url]) [Christian Grete][repository-owner-url]

[repository-github-url]: https://github.com/ChristianGrete/ai\_messenger
[repository-license-url]: LICENSE
[repository-owner-url]: https://christiangrete.com
