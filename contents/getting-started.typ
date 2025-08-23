// title: Getting Started with Typstify
// description: Learn how to get started with Typstify, a modern documentation site generator
// author: Typstify Team
// tags: getting-started, installation, setup

= Getting Started with Typstify

Welcome to Typstify! This guide will help you get up and running with your first documentation site.

== What is Typstify?

Typstify is a modern static site generator designed specifically for creating beautiful documentation websites. It combines the power of:

- *Typst*: A modern typesetting system for scientific documents
- *Leptos*: A fast, reactive web framework for Rust
- *Tailwind CSS*: A utility-first CSS framework
- *DaisyUI*: Beautiful components for Tailwind CSS

== Why Choose Typstify?

=== 🚀 Performance First
Built with Rust and WebAssembly, Typstify delivers lightning-fast performance both during build time and runtime.

=== 📝 Beautiful Typography
Leverage Typst's powerful typesetting capabilities to create stunning documentation with mathematical formulas, diagrams, and more.

=== 🎨 Web3-Ready Design
Modern, responsive design with Dracula theme that's perfect for blockchain and decentralized application documentation.

=== 🔍 Powerful Search
Advanced search capabilities help users find content quickly and efficiently.

== Prerequisites

Before you begin, make sure you have the following installed:

- *Rust* (latest stable version)
- *Node.js* (version 18 or higher)
- *cargo-leptos* for building Leptos applications

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install cargo-leptos
cargo install cargo-leptos

# Install Node.js dependencies
bun install
```

== Quick Start

=== 1. Clone the Repository

```bash
git clone https://github.com/longcipher/typstify.git
cd typstify
```

=== 2. Install Dependencies

```bash
# Install Rust dependencies
cargo build

# Install Node.js dependencies for Tailwind CSS
bun install
```

=== 3. Start Development Server

```bash
# Start the development server with hot reloading
cargo leptos watch
```

Your site will be available at `http://localhost:3000`.

=== 4. Create Your First Document

Create a new file in the `contents/` directory:

```typst
// title: My First Document
// description: This is my first Typstify document
// tags: example, tutorial

= My First Document

This is a paragraph in my first Typstify document.

== Subsection

You can create subsections and add content like:

- Lists
- *Bold text*
- _Italic text_
- `Code snippets`

=== Math Formulas

You can also include mathematical formulas:

$ sum_(i=1)^n i = (n(n+1))/2 $

=== Code Blocks

```rust
fn main() {
    println!("Hello, Typstify!");
}
```
```

== Configuration

Typstify uses a `config.toml` file in the root directory for configuration. Here's an example:

```toml
[site]
title = "My Documentation"
description = "My awesome documentation site"
base_url = "https://mydocs.dev"

[theme]
name = "dracula"
primary_color = "#bd93f9"
secondary_color = "#ff79c6"

[navigation]
[[navigation.items]]
title = "Getting Started"
path = "/getting-started"

[[navigation.items]]
title = "Documentation"
path = "/docs"
```

== Next Steps

Now that you have Typstify running, here's what you can do next:

1. *Explore the Documentation*: Read through the full documentation to understand all features
2. *Customize Your Theme*: Modify the theme colors and styling to match your brand
3. *Add Content*: Start writing your documentation using Typst syntax
4. *Configure Navigation*: Set up your site's navigation structure
5. *Deploy Your Site*: Learn how to deploy your documentation to various platforms

== Getting Help

If you need help or have questions:

- 📖 Read the full documentation
- 🐛 Report issues on GitHub
- 💬 Join our community discussions
- 📧 Contact the maintainers

Happy documenting! 🎉
