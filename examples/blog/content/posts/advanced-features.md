---
title: "Advanced Typstify Features"
date: 2024-01-25T10:00:00Z
description: "Explore advanced features like custom templates, taxonomies, and multi-language support."
tags: ["advanced", "features", "i18n"]
aliases: ["/old-url/advanced"]
custom_css: ["custom.css"]
draft: false
---

This post covers some of the more advanced features available in Typstify.

## Taxonomies

Typstify automatically generates taxonomy pages for tags and categories:

- `/tags/` - List of all tags
- `/tags/rust/` - All posts tagged with "rust"
- `/categories/` - List of all categories

### Custom Taxonomies

You can define custom taxonomies in your frontmatter:

```yaml
series: "Learning Rust"
difficulty: "intermediate"
```

## URL Aliases

Redirect old URLs to new ones:

```yaml
aliases: ["/old-url", "/another-old-url"]
```

Typstify generates redirect HTML files automatically.

## Multi-Language Support

### Filename-Based i18n

Typstify uses filename suffixes for multi-language content:

```text
content/
├── posts/
│   ├── hello.md        # Default language (en)
│   └── hello.zh.md     # Chinese version
```

### Language-Specific Config

```toml
[site]
default_language = "en"

[languages.en]
name = "English"

[languages.zh]
name = "中文"
title = "我的博客"
```

## Custom Assets

### Per-Page CSS

```yaml
custom_css: ["syntax.css", "diagrams.css"]
```

### Per-Page JavaScript

```yaml
custom_js: ["interactive.js"]
```

## Table of Contents

Typstify automatically extracts headings to generate a table of contents.

### Nested Headings

## Level 2

### Level 3

#### Level 4

All these are included in the TOC.

## Syntax Highlighting

Typstify uses Syntect for syntax highlighting with many themes:

- `base16-ocean.dark`
- `InspiredGitHub`
- `Solarized (dark)`
- `Solarized (light)`

Configure in `config.toml`:

```toml
[build]
syntax_theme = "base16-ocean.dark"
```

## Search

The built-in search uses a WASM runtime for client-side full-text search:

- No server required
- Works offline
- Supports highlighting

Press `Cmd/Ctrl + K` to open the search modal.
