# Typstify

[![Crates.io](https://img.shields.io/crates/v/typstify.svg)](https://crates.io/crates/typstify)
[![Docs.rs](https://docs.rs/typstify/badge.svg)](https://docs.rs/typstify)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

![typstify](https://socialify.git.ci/longcipher/typstify/image?font=Source+Code+Pro&language=1&name=1&owner=1&pattern=Circuit+Board&theme=Auto)

A high-performance static site generator with **Typst** and **Markdown** support.

## Features

- 🚀 **Blazing Fast** - Built in Rust for maximum performance
- 📝 **Dual Format** - Write in Markdown or Typst
- 🔍 **Built-in Search** - Tantivy powered full-text search with WASM runtime
- 🌐 **Multi-language** - First-class i18n support
- 🔄 **Live Reload** - Instant feedback during development
- 📊 **Syntax Highlighting** - 100+ languages supported
- 📰 **RSS & Sitemap** - Automatic feed generation
- 🎨 **Customizable** - Templates, themes, and styles

## Crates

Typstify is a modular project composed of several crates:

| Crate | Description | Documentation |
|-------|-------------|---------------|
| `typstify` | Main CLI tool and server | [![Docs](https://img.shields.io/docsrs/typstify)](https://docs.rs/typstify) |
| `typstify-core` | Core types and config | [![Docs](https://img.shields.io/docsrs/typstify-core)](https://docs.rs/typstify-core) |
| `typstify-parser` | Content parsers | [![Docs](https://img.shields.io/docsrs/typstify-parser)](https://docs.rs/typstify-parser) |
| `typstify-generator` | Generation engine | [![Docs](https://img.shields.io/docsrs/typstify-generator)](https://docs.rs/typstify-generator) |
| `typstify-search` | Search indexing | [![Docs](https://img.shields.io/docsrs/typstify-search)](https://docs.rs/typstify-search) |
| `typstify-search-wasm` | WASM search runtime | [![Docs](https://img.shields.io/docsrs/typstify-search-wasm)](https://docs.rs/typstify-search-wasm) |
| `typstify-ui` | UI components | [![Docs](https://img.shields.io/docsrs/typstify-ui)](https://docs.rs/typstify-ui) |

## Installation

### From Cargo

```bash
cargo install typstify
```

### From Source

```bash
git clone https://github.com/longcipher/typstify
cd typstify
cargo install --path bin/typstify
```

## Quick Start

### 1. Create a New Site

```bash
mkdir my-site && cd my-site
```

### 2. Create Configuration

Create `config.toml`:

```toml
[site]
title = "My Blog"
base_url = "https://example.com"

[build]
output_dir = "public"
```

### 3. Create Content

```bash
mkdir -p content/posts
typstify new posts/hello-world
```

### 4. Build & Preview

```bash
typstify watch --open
```

Your site is now running at `http://127.0.0.1:3000`!

## Commands

| Command | Description |
|---------|-------------|
| `typstify build` | Build the site for production |
| `typstify watch` | Start dev server with live reload |
| `typstify new <path>` | Create new content from template |
| `typstify check` | Validate configuration and content |

### Build Options

```bash
typstify build --output dist    # Custom output directory
typstify build --drafts         # Include draft posts
```

### Watch Options

```bash
typstify watch --port 8080      # Custom port
typstify watch --open           # Open browser automatically
```

### Global Options

```bash
typstify -c site.toml build     # Custom config file
typstify -v build               # Verbose output
typstify -vvv build             # Debug output
```

## Project Structure

```text
my-site/
├── config.toml         # Site configuration
├── content/            # Content files
│   ├── posts/          # Blog posts (Markdown/Typst)
│   ├── docs/           # Documentation
│   └── about.md        # Static page
├── templates/          # HTML templates (optional)
├── style/              # CSS/Tailwind (optional)
├── assets/             # Static assets
└── public/             # Generated output
```

## Content Formats

### Markdown

```markdown
---
title: "My Post"
date: 2024-01-15
tags: ["rust", "web"]
---

# Hello World

Your content here...
```

### Typst

```typst
// typstify:frontmatter
// title: "Technical Doc"
// date: 2024-01-15

= Introduction

Your Typst content here...
```

## Configuration Reference

See [docs/configuration.md](docs/configuration.md) for full configuration options.

### Basic Configuration

```toml
[site]
title = "My Site"
description = "Site description"
base_url = "https://example.com"
default_language = "en"
languages = ["en", "zh"]

[build]
output_dir = "public"
minify = false
syntax_theme = "base16-ocean.dark"
drafts = false

[search]
enabled = true
index_fields = ["title", "body", "tags"]

[rss]
enabled = true
limit = 20
```

## Documentation

- [Configuration Reference](docs/configuration.md)
- [Content Format Guide](docs/content-format.md)
- [Design Document](docs/design.md)

## Architecture

Typstify is organized as a Cargo workspace:

| Crate | Description |
|-------|-------------|
| `typstify` | CLI binary |
| `typstify-core` | Configuration and content types |
| `typstify-parser` | Markdown/Typst parsing |
| `typstify-generator` | HTML generation and build |
| `typstify-search` | Search indexing (Tantivy) |
| `typstify-search-wasm` | WASM search runtime |
| `typstify-ui` | Leptos UI components |

## Development

### Prerequisites

- Rust 1.75+
- Bun (for CSS/JS tooling)
- wasm-pack (for WASM builds)

### Setup

```bash
git clone https://github.com/longcipher/typstify
cd typstify
just dev
```

### Commands

```bash
just format    # Format code
just lint      # Run linters
just test      # Run all tests
just build     # Production build
```

## License

Apache License 2.0 - see [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) before submitting a PR.
