// title: Quick Start Guide
// description: Get up and running with Typstify in 5 minutes
// author: Typstify Team
// tags: quickstart, tutorial, guide

= Quick Start Guide

Get your documentation site running in just 5 minutes! This guide walks you through creating your first Typstify site.

== Step 1: Prerequisites Check

Before we begin, make sure you have:

- ✅ Rust (latest stable)
- ✅ Node.js (v18+)
- ✅ Git
- ✅ A terminal/command prompt

Not installed? Check our [installation guide](/getting-started/installation).

== Step 2: Create Your Site

```bash
# Clone the Typstify template
git clone https://github.com/longcipher/typstify.git my-docs
cd my-docs

# Install dependencies
cargo build
bun install
```

== Step 3: Start Development Server

```bash
# Start the development server with hot reloading
cargo leptos watch
```

🎉 Your site is now running at `http://localhost:3000`!

== Step 4: Create Your First Page

Create a new file in the `contents/` directory:

```bash
# Create a new document
touch contents/hello-world.typ
```

Add some content:

```typst
// title: Hello World
// description: My first Typstify document
// tags: example, hello-world

= Hello World

Welcome to my first Typstify document!

== Getting Started

This is a simple example of what you can do with Typstify:

=== Lists

- Easy to write
- Beautiful output
- Markdown-like syntax

=== Code Blocks

```rust
fn main() {
    println!("Hello, Typstify!");
}
```

=== Mathematics

You can include mathematical formulas:

$ E = m c^2 $

=== Tables

| Feature | Status |
|---------|--------|
| Fast | ✅ |
| Beautiful | ✅ |
| Easy to use | ✅ |
```

== Step 5: Configure Your Site

Edit the `config.toml` file to customize your site:

```toml
[site]
title = "My Awesome Documentation"
description = "Documentation for my amazing project"
base_url = "https://my-docs.dev"

[theme]
name = "dracula"
primary_color = "#bd93f9"
secondary_color = "#ff79c6"

[navigation]
[[navigation.items]]
title = "Home"
path = "/"

[[navigation.items]]
title = "Hello World"
path = "/hello-world"

[[navigation.items]]
title = "Getting Started"
path = "/getting-started"
children = [
  { title = "Installation", path = "/getting-started/installation" },
  { title = "Quick Start", path = "/getting-started/quick-start" },
]
```

== Step 6: Build for Production

When you're ready to deploy:

```bash
# Build the optimized production version
cargo leptos build --release

# Your site will be in the dist/ directory
ls dist/
```

== What's Next?

=== 📝 Learn Typst Syntax

Typst is similar to Markdown but more powerful:

```typst
// Comments start with //

= Heading 1
== Heading 2
=== Heading 3

*Bold text*
_Italic text_
`Code text`

- Bullet list
  - Nested item

1. Numbered list
2. Another item

#link("https://example.com")[Link text]

#image("path/to/image.png")
```

=== 🎨 Customize Your Theme

Modify your theme in `config.toml`:

```toml
[theme]
name = "dracula"  # or "dark", "light", "cyberpunk", "synthwave"
primary_color = "#your-color"
secondary_color = "#your-color"
accent_color = "#your-color"
```

=== 📁 Organize Your Content

Structure your content with directories:

```
contents/
├── index.typ           # Home page
├── getting-started/
│   ├── installation.typ
│   └── quick-start.typ
├── guides/
│   ├── writing.typ
│   └── deployment.typ
└── reference/
    ├── api.typ
    └── config.typ
```

=== 🔍 Enable Search

Search is automatically enabled! Users can:

- Use the search box in the header
- Press `Ctrl/Cmd + K` for quick search
- Browse the `/search` page

=== 🚀 Deploy Your Site

Deploy to popular platforms:

```bash
# GitHub Pages
bun run build
# Copy dist/ to your gh-pages branch

# Netlify
# Connect your repo and set build command: cargo leptos build --release

# Vercel
# Import your repo and use the Rust preset
```

== Common Tasks

=== Adding a New Page

1. Create a new `.typ` file in `contents/`
2. Add frontmatter with title and metadata
3. Add the page to navigation in `config.toml`
4. Write your content using Typst syntax

=== Adding Images

```typst
// Place images in public/images/
#image("/images/my-image.png", width: 80%)
```

=== Creating Links

```typst
// Link to other pages
#link("/getting-started")[Getting Started Guide]

// External links
#link("https://typst.app/")[Typst Website]
```

=== Adding Math

```typst
// Inline math
The formula $E = m c^2$ is famous.

// Display math
$ integral_0^infinity e^(-x^2) dif x = sqrt(pi)/2 $
```

== Troubleshooting

=== Site Won't Start

```bash
# Check if ports are in use
lsof -i :3000

# Kill process if needed
kill -9 PID

# Try a different port
LEPTOS_SITE_ADDR=127.0.0.1:3001 cargo leptos watch
```

=== Build Errors

```bash
# Clean and rebuild
cargo clean
cargo build

# Update dependencies
cargo update
```

=== Style Issues

```bash
# Rebuild CSS
bun run build

# Clear browser cache
# Ctrl+Shift+R (or Cmd+Shift+R on Mac)
```

== Getting Help

Need help? Here are your options:

- 📖 Read the [full documentation](/docs)
- 🐛 [Report a bug](https://github.com/longcipher/typstify/issues)
- 💬 [Ask questions](https://github.com/longcipher/typstify/discussions)
- 🔍 [Search existing issues](https://github.com/longcipher/typstify/issues)

== Example Sites

Check out these example sites built with Typstify:

- [Official Documentation](https://typstify.dev)
- [Blockchain Project Docs](https://example-blockchain.dev)
- [API Reference Site](https://api-docs.example.com)

== Tips and Tricks

=== Development Workflow

```bash
# Start development in one terminal
cargo leptos watch

# In another terminal, watch CSS changes
bun run build:watch

# Use --open to automatically open browser
cargo leptos watch --open
```

=== Content Organization

- Use descriptive filenames: `user-authentication.typ` not `auth.typ`
- Group related content in folders
- Use consistent frontmatter across all documents
- Keep navigation structure flat when possible

=== Performance Tips

- Optimize images before adding them
- Use WebP format for better compression
- Keep documents focused and not too long
- Use proper heading hierarchy for better SEO

Congratulations! You now have a working Typstify site. Start creating amazing documentation! 🎉
