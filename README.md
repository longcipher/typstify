# Typstify

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Leptos](https://img.shields.io/badge/leptos-0.8-blue.svg)](https://leptos.dev/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

A modern documentation site generator built with **Typst** and **Leptos**, featuring beautiful **Web3-inspired design** with **Dracula theme** and **DaisyUI** components.

![Typstify Preview](public/images/preview.png)

## ✨ Features

- 🚀 **Lightning Fast**: Built with Rust and WebAssembly for incredible performance
- 📝 **Typst-Powered**: Beautiful typography and mathematical formulas
- 🎨 **Web3-Ready Design**: Modern, responsive design with Dracula theme
- 🔍 **Powerful Search**: Advanced search capabilities with real-time results
- 📱 **Mobile-Friendly**: Responsive design that works on all devices
- ⚡ **Hot Reload**: Instant updates during development
- 🌐 **SSG**: Static site generation for optimal performance
- 🎯 **SEO-Optimized**: Built-in SEO optimization and meta tags

## 🎯 Perfect For

- **Blockchain Projects**: DeFi protocols, DAOs, and Web3 applications
- **API Documentation**: RESTful APIs, GraphQL, and SDK documentation
- **Technical Guides**: Developer documentation and tutorials
- **Open Source Projects**: GitHub project documentation
- **Academic Papers**: Research papers with mathematical formulas

## 🚀 Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (v18+)
- [cargo-leptos](https://github.com/leptos-rs/cargo-leptos)

### Installation

```bash
# Clone the repository
git clone https://github.com/longcipher/typstify.git
cd typstify

# Install dependencies
cargo build
bun install

# Start development server
cargo leptos watch
```

Visit `http://localhost:3000` to see your site!

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

## 📖 Documentation

- [Getting Started](contents/getting-started.typ) - Learn the basics
- [Installation Guide](contents/getting-started/installation.typ) - Detailed setup
- [Quick Start](contents/getting-started/quick-start.typ) - 5-minute setup
- [Configuration](docs/configuration.md) - Customize your site
- [Deployment](docs/deployment.md) - Go live

## 🏗️ Project Structure

```
typstify/
├── config.toml              # Site configuration
├── contents/                # Your Typst documents
│   ├── getting-started.typ
│   └── getting-started/
│       ├── installation.typ
│       └── quick-start.typ
├── typst-components/         # Leptos Typst components library
│   ├── src/
│   │   ├── components/      # UI components
│   │   ├── renderer.rs      # Typst renderer
│   │   └── types.rs         # Type definitions
│   └── Cargo.toml
├── typstify-site/           # Main frontend application
│   ├── src/
│   │   ├── app.rs          # Main app component
│   │   ├── pages/          # Page components
│   │   └── components/     # Site-specific components
│   └── Cargo.toml
├── public/                  # Static assets
├── style/                   # Styles and themes
└── dist/                    # Built site (generated)
```

## ⚙️ Configuration

Configure your site in `config.toml`:

```toml
[site]
title = "My Documentation"
description = "Amazing project documentation"
base_url = "https://my-docs.dev"

[theme]
name = "dracula"
primary_color = "#bd93f9"
secondary_color = "#ff79c6"
accent_color = "#50fa7b"

[navigation]
[[navigation.items]]
title = "Getting Started"
path = "/getting-started"
children = [
  { title = "Installation", path = "/getting-started/installation" },
  { title = "Quick Start", path = "/getting-started/quick-start" },
]

[social]
github = "https://github.com/yourusername/yourproject"
```

## 🎨 Themes

Typstify comes with beautiful themes:

- **Dracula** (default) - Dark purple theme perfect for Web3
- **Dark** - Clean dark theme
- **Light** - Minimal light theme  
- **Cyberpunk** - Neon-inspired theme
- **Synthwave** - Retro-futuristic theme

## 🛠️ Development

### Building

```bash
# Development build
cargo leptos build

# Production build
cargo leptos build --release

# Watch for changes
cargo leptos watch
```

### CSS Development

```bash
# Build Tailwind CSS
bun run build

# Watch CSS changes
bun run build:watch
```

### Testing

```bash
# Run Rust tests
cargo test

# Run end-to-end tests
cd end2end
bun test
```

## 🚀 Deployment

### Static Hosting

```bash
# Build for production
cargo leptos build --release

# Deploy the dist/ directory to:
# - GitHub Pages
# - Netlify
# - Vercel
# - Any static host
```

### Containerized Deployment

```dockerfile
FROM rust:alpine as builder
WORKDIR /app
COPY . .
RUN cargo leptos build --release

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
```

## 🤝 Contributing

We love contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and test thoroughly
4. Commit: `git commit -m 'Add amazing feature'`
5. Push: `git push origin feature/amazing-feature`
6. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Typst](https://typst.app/) - Modern typesetting system
- [Leptos](https://leptos.dev/) - Reactive web framework for Rust
- [Tailwind CSS](https://tailwindcss.com/) - Utility-first CSS framework
- [DaisyUI](https://daisyui.com/) - Beautiful component library
- [Dracula Theme](https://draculatheme.com/) - Dark theme inspiration

## 🌟 Show Your Support

If you like this project, please give it a ⭐ on GitHub!

## 📞 Support

- 📖 [Documentation](https://typstify.dev)
- 🐛 [Report Issues](https://github.com/longcipher/typstify/issues)
- 💬 [Discussions](https://github.com/longcipher/typstify/discussions)
- 📧 [Email Support](mailto:support@typstify.dev)

---

<div align="center">
  <strong>Built with ❤️ for the Web3 community</strong>
</div>
