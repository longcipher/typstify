---
title: "Getting Started with Typstify"
date: 2024-01-20T10:00:00Z
description: "Learn how to set up and use Typstify for your static site."
tags: ["tutorial", "guide"]
draft: false
---

This guide will walk you through setting up your first Typstify site.

## Installation

First, install Typstify using Cargo:

```bash
cargo install typstify
```

## Creating a New Site

Create a new directory for your site:

```bash
mkdir my-site
cd my-site
```

Initialize the configuration:

```bash
typstify new config.toml
```

## Writing Content

### Markdown Posts

Create your first post:

```bash
typstify new posts/my-first-post
```

This creates a new Markdown file with frontmatter:

```markdown
---
title: "My First Post"
date: 2024-01-20
draft: true
tags: []
---

Write your content here.
```

### Typst Documents

For more complex documents, use Typst:

```bash
typstify new docs/technical-doc --template typst
```

## Building Your Site

Build for production:

```bash
typstify build
```

Or start the development server:

```bash
typstify watch --open
```

## Configuration

Edit `config.toml` to customize your site. See the [configuration reference](/docs/configuration) for all options.

## Next Steps

- Read the [content format guide](/docs/content-format)
- Explore the [theme customization](/docs/themes)
- Set up [search](/docs/search)
