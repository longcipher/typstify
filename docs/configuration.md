# Configuration Reference

This document describes all available configuration options for Typstify.

## Configuration File

Typstify uses TOML for configuration. The default config file is `config.toml` in the project root.

```bash
# Use default config.toml
typstify build

# Use custom config file
typstify -c site.toml build
```

## Site Configuration

```toml
[site]
title = "My Site"
description = "A Typstify-powered site"
base_url = "https://example.com"
default_language = "en"
languages = ["en", "zh", "ja"]
```

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `title` | string | required | Site title, used in templates and RSS |
| `description` | string | `""` | Site description for meta tags |
| `base_url` | string | `""` | Production URL (with protocol) |
| `default_language` | string | `"en"` | Default content language |
| `languages` | array | `["en"]` | Supported languages for i18n |

### Examples

#### Single Language Site

```toml
[site]
title = "Tech Blog"
base_url = "https://blog.example.com"
```

#### Multi-Language Site

```toml
[site]
title = "International Blog"
base_url = "https://example.com"
default_language = "en"
languages = ["en", "zh", "ja", "ko"]
```

## Build Configuration

```toml
[build]
output_dir = "public"
minify = true
syntax_theme = "base16-ocean.dark"
drafts = false
```

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `output_dir` | string | `"public"` | Output directory for generated files |
| `minify` | boolean | `false` | Minify HTML output |
| `syntax_theme` | string | `"base16-ocean.dark"` | Syntax highlighting theme |
| `drafts` | boolean | `false` | Include draft posts in build |

### Available Syntax Themes

- `base16-ocean.dark` (default)
- `base16-ocean.light`
- `InspiredGitHub`
- `Solarized (dark)`
- `Solarized (light)`
- `base16-eighties.dark`
- `base16-mocha.dark`

### Examples

#### Production Build

```toml
[build]
output_dir = "dist"
minify = true
drafts = false
```

#### Development Build

```toml
[build]
output_dir = "public"
minify = false
drafts = true
```

## Search Configuration

```toml
[search]
enabled = true
index_fields = ["title", "body", "tags"]
chunk_size = 65536
```

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Enable search functionality |
| `index_fields` | array | `["title", "body", "tags"]` | Fields to include in search index |
| `chunk_size` | integer | `65536` | Chunk size for index files (bytes) |

### Index Fields

Available fields for indexing:

- `title` - Page title
- `body` - Page content (HTML stripped)
- `tags` - Page tags
- `description` - Page description

### Examples

#### Full-Text Search

```toml
[search]
enabled = true
index_fields = ["title", "body", "tags", "description"]
```

#### Title-Only Search

```toml
[search]
enabled = true
index_fields = ["title", "tags"]
```

#### Disable Search

```toml
[search]
enabled = false
```

## RSS Configuration

```toml
[rss]
enabled = true
limit = 20
```

### Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | boolean | `true` | Generate RSS feed |
| `limit` | integer | `20` | Maximum items in feed |

### Examples

#### Large Feed

```toml
[rss]
enabled = true
limit = 100
```

#### Disable RSS

```toml
[rss]
enabled = false
```

## Complete Example

```toml
# Full configuration example

[site]
title = "My Typstify Blog"
description = "A blog about Rust, Web, and more"
base_url = "https://blog.example.com"
default_language = "en"
languages = ["en", "zh"]

[build]
output_dir = "public"
minify = true
syntax_theme = "Solarized (dark)"
drafts = false

[search]
enabled = true
index_fields = ["title", "body", "tags"]
chunk_size = 65536

[rss]
enabled = true
limit = 50
```

## Environment Variables

Typstify respects the following environment variables:

| Variable | Description |
|----------|-------------|
| `TYPSTIFY_CONFIG` | Override config file path |
| `RUST_LOG` | Set logging level (e.g., `debug`, `info`) |

## Command-Line Overrides

Some options can be overridden via CLI:

```bash
# Override output directory
typstify build --output dist

# Include drafts
typstify build --drafts

# Custom port for dev server
typstify watch --port 8080
```
