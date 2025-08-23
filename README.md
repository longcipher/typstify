# Typstify

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A modern static site generator built with **Rust**, supporting both **Typst** and **Markdown** content with beautiful **mdBook-style layout** powered by **Tailwind CSS v4** and **DaisyUI**.

## ✨ Features

- 🚀 **Lightning Fast**: Built with Rust for incredible performance
- 📝 **Multi-Format Support**: Process both Typst (.typ) and Markdown (.md) files
- 🎨 **Modern Design**: mdBook-style layout with collapsible navigation
- 📱 **Mobile-Friendly**: Responsive design with Tailwind CSS v4 and DaisyUI
- 🌐 **Static Site Generation**: Fast, SEO-friendly static sites
- 🔧 **Configurable**: Extensive configuration options via TOML
- 📊 **RSS Feed**: Automatic RSS/Atom feed generation
- 🗺️ **Sitemap**: Auto-generated sitemap for SEO
- 🎯 **OpenGraph**: Built-in social media optimization
- 🔍 **Code Highlighting**: Beautiful syntax highlighting with Dracula theme

## 🎯 Perfect For

- **Technical Documentation**: API docs, guides, and tutorials
- **Open Source Projects**: GitHub project documentation
- **Academic Papers**: Research papers with mathematical formulas (Typst)
- **Knowledge Bases**: Internal documentation and wikis
- **Blogs**: Technical blogs with code examples

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) (for CSS compilation)

### Installation

```bash
# Clone the repository
git clone https://github.com/longcipher/typstify.git
cd typstify

# Install dependencies
just install
```

### Generate Your First Site

```bash
# Generate documentation with styles
just dev

# Or generate and serve immediately
just docs
```

Visit `http://localhost:8080` to see your site!

### Create Your First Document

Create a file in `contents/my-page.typ`:

```typst
// title: My First Page
// description: This is my first Typstify document
// tags: example, tutorial

= Welcome to Typstify

This is a paragraph with *bold* and _italic_ text.

== Mathematical Formulas

You can include beautiful math:

$ sum_(i=1)^n i = (n(n+1))/2 $

== Code Examples

```rust
fn main() {
    println!("Hello, Typstify!");
}
```

== Lists and More

- Beautiful typography
- Fast performance  
- Easy to use
```

Or create a Markdown file in `contents/my-page.md`:

```markdown
---
title: My First Markdown Page
description: This is my first Markdown document
tags: [example, tutorial, markdown]
---

# Welcome to Typstify

This is a paragraph with **bold** and *italic* text.

## Code Examples

```rust
fn main() {
    println!("Hello, Typstify!");
}
```

## Lists and More

- Beautiful typography
- Fast performance  
- Easy to use
```

## 📖 Documentation

- [Getting Started](contents/getting-started.typ) - Learn the basics
- [Installation Guide](contents/getting-started/installation.typ) - Detailed setup
- [Quick Start](contents/getting-started/quick-start.typ) - 5-minute setup
- [Rust Guide](contents/rust-guide.md) - Rust development guide
- [JavaScript Modern](contents/javascript-modern.md) - Modern JavaScript guide

## 🏗️ Project Structure

```text
typstify/
├── config.toml                  # Site configuration
├── Justfile                     # Build commands
├── contents/                    # Your Typst and Markdown documents
│   ├── getting-started.typ
│   ├── rust-guide.md
│   ├── javascript-modern.md
│   └── getting-started/
│       ├── installation.typ
│       └── quick-start.typ
├── typstify-ssg/               # Static site generator
│   ├── src/
│   │   ├── main.rs             # CLI entry point
│   │   ├── config.rs           # Configuration handling
│   │   ├── content.rs          # Content processing
│   │   ├── renderers.rs        # Typst and Markdown renderers
│   │   ├── mdbook_template.rs  # HTML template generation
│   │   └── pages.rs            # Page generation
│   └── Cargo.toml
├── style/                      # CSS and styling
│   ├── input.css              # Tailwind CSS source
│   └── output.css             # Generated CSS
├── site/                       # Generated static site
└── target/                     # Rust build artifacts
```

## ⚙️ Configuration

Configure your site in `config.toml`:

```toml
[site]
title = "Typstify Documentation"
description = "A static site generator that supports both Markdown and Typst content"
base_url = "https://typstify.dev"
author = "Typstify Team"

[build]
content_dir = "contents"
output_dir = "site"
style_dir = "style"
assets_dir = "assets"

[rendering]
syntax_highlighting = true
code_theme = "dracula"
generate_toc = true
toc_depth = 3

[features]
feed = true        # Generate RSS feed
sitemap = true     # Generate sitemap
search = false     # Search functionality (future)
opengraph = true   # Social media meta tags

[feed]
filename = "feed.xml"
max_items = 20
language = "en"

[dev]
port = 5173
watch = true
reload_port = 3002
```

## 🛠️ Development

### Available Commands

```bash
# 📖 Documentation Generation
just dev           # Generate documentation site with styles
just docs           # Generate and serve documentation
just serve          # Serve generated documentation

# ⚙️ Build & Manage
just build          # Build static site
just build-styles   # Build Tailwind CSS styles
just watch-styles   # Watch and rebuild styles
just clean          # Clean generated site and styles
just new-content    # Create new content file

# � Development Tools
just format         # Format code (taplo, cargo fmt)
just lint           # Lint code (taplo, clippy, cargo machete)

# 🛠️ Setup
just install        # Install all dependencies
```

### CSS Development

This project uses **Tailwind CSS v4** with **DaisyUI** for styling:

```bash
# Build styles
just build-styles

# Watch for style changes
just watch-styles

# Or use bun directly
bunx @tailwindcss/cli --input ./style/input.css --output ./style/output.css --watch
```

### Testing and Quality

```bash
# Format code
just format

# Run linting checks
just lint

# Clean build artifacts
just clean
```

## 🚀 Deployment

### Static Hosting

```bash
# Build for production
just build

# Deploy the site/ directory to:
# - GitHub Pages
# - Netlify
# - Vercel
# - Any static host
```

### Using Docker

```dockerfile
FROM rust:alpine as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM nginx:alpine
COPY --from=builder /app/site /usr/share/nginx/html
```

### GitHub Actions

```yaml
name: Deploy Typstify Site

on:
  push:
    branches: [ main ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: oven-sh/setup-bun@v1
    
    - name: Install dependencies
      run: |
        cargo build
        bun install
    
    - name: Build site
      run: just dev
    
    - name: Deploy to GitHub Pages
      uses: peaceiris/actions-gh-pages@v3
      with:
        github_token: ${{ secrets.GITHUB_TOKEN }}
        publish_dir: ./site
```

## 🤝 Contributing

We love contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and test thoroughly
4. Format and lint: `just format && just lint`
5. Commit: `git commit -m 'Add amazing feature'`
6. Push: `git push origin feature/amazing-feature`
7. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Typst](https://typst.app/) - Modern typesetting system
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Tailwind CSS](https://tailwindcss.com/) - Utility-first CSS framework
- [DaisyUI](https://daisyui.com/) - Beautiful component library
- [Dracula Theme](https://draculatheme.com/) - Dark theme inspiration
- [mdBook](https://rust-lang.github.io/mdBook/) - Layout inspiration

## 🌟 Show Your Support

If you like this project, please give it a ⭐ on GitHub!

## 📞 Support

- 📖 [Documentation](https://typstify.dev)
- 🐛 [Report Issues](https://github.com/longcipher/typstify/issues)
- 💬 [Discussions](https://github.com/longcipher/typstify/discussions)

---

**Built with ❤️ for the documentation community**
