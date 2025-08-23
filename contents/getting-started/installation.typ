// title: Installation Guide
// description: Detailed installation instructions for Typstify
// author: Typstify Team
// tags: installation, setup, requirements

= Installation Guide

This guide provides detailed instructions for installing and setting up Typstify on your system.

== System Requirements

=== Minimum Requirements

- *Operating System*: Windows 10, macOS 10.15, or Linux (Ubuntu 18.04+)
- *Memory*: 4 GB RAM minimum, 8 GB recommended
- *Storage*: 2 GB free disk space
- *Internet Connection*: Required for downloading dependencies

=== Required Software

==== Rust Toolchain

Typstify is built with Rust, so you'll need the Rust toolchain installed:

```bash
# Install Rust using rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart your terminal or run:
source $HOME/.cargo/env

# Verify installation
rustc --version
cargo --version
```

==== Node.js and npm

For CSS processing and development tools:

```bash
# On macOS using Homebrew
brew install node

# On Ubuntu/Debian
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
sudo apt-get install -y nodejs

# On Windows, download from nodejs.org

# Verify installation
node --version
npm --version
```

==== cargo-leptos

The Leptos build tool for Rust web applications:

```bash
cargo install cargo-leptos
```

== Installation Methods

=== Method 1: Clone from GitHub (Recommended)

```bash
# Clone the repository
git clone https://github.com/longcipher/typstify.git
cd typstify

# Install Rust dependencies
cargo build

# Install Node.js dependencies
bun install

# Start development server
cargo leptos watch
```

=== Method 2: Download Release

1. Go to the [releases page](https://github.com/longcipher/typstify/releases)
2. Download the latest release archive
3. Extract to your desired location
4. Follow the setup instructions in the README

=== Method 3: Using Cargo (Future)

_Coming soon:_

```bash
cargo install typstify
typstify new my-site
cd my-site
typstify serve
```

== Development Setup

=== IDE Configuration

==== VS Code (Recommended)

Install these extensions:

- `rust-analyzer`: Rust language support
- `Leptos Language Server`: Leptos-specific features
- `Tailwind CSS IntelliSense`: CSS utilities
- `Typst LSP`: Typst language support

==== Other IDEs

- *IntelliJ IDEA*: Install the Rust plugin
- *Vim/Neovim*: Use rust.vim and coc-rust-analyzer
- *Emacs*: Use rust-mode and lsp-mode

=== Environment Variables

Set up environment variables for development:

```bash
# Add to your shell profile (.bashrc, .zshrc, etc.)
export TYPSTIFY_ENV=development
export TYPSTIFY_PORT=3000
export TYPSTIFY_HOST=127.0.0.1
```

=== Git Hooks (Optional)

Set up pre-commit hooks for code quality:

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install

# Run hooks manually
pre-commit run --all-files
```

== Building for Production

=== Development Build

```bash
cargo leptos build
```

=== Release Build

```bash
cargo leptos build --release
```

=== Optimized Build

```bash
# Enable all optimizations
RUSTFLAGS="-C target-cpu=native" cargo leptos build --release
```

== Troubleshooting

=== Common Issues

==== "cargo-leptos not found"

```bash
# Make sure cargo-leptos is installed
cargo install cargo-leptos --force

# Check PATH
echo $PATH | grep -q cargo && echo "Cargo in PATH" || echo "Add ~/.cargo/bin to PATH"
```

==== "Node.js modules not found"

```bash
# Clear cache and reinstall
bun install --force
```

==== "Rust compiler errors"

```bash
# Update Rust toolchain
rustup update

# Clean build artifacts
cargo clean

# Rebuild
cargo build
```

==== "Permission denied" on Linux/macOS

```bash
# Make sure you have proper permissions
sudo chown -R $USER:$USER ~/.cargo
sudo chown -R $USER:$USER ~/.rustup
```

=== Platform-Specific Issues

==== Windows

- Install Visual Studio Build Tools if you encounter linking errors
- Use PowerShell or Git Bash instead of Command Prompt
- Consider using WSL2 for a better development experience

==== macOS

- Install Xcode Command Line Tools: `xcode-select --install`
- Use Homebrew for package management
- On Apple Silicon Macs, some dependencies might need Rosetta 2

==== Linux

- Install build essentials: `sudo apt-get install build-essential`
- Install OpenSSL development headers: `sudo apt-get install libssl-dev`
- For older distributions, you might need to compile Rust from source

=== Performance Optimization

==== Build Time

```bash
# Use parallel compilation
export CARGO_BUILD_JOBS=4

# Use faster linker (Linux)
sudo apt-get install lld
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"

# Use faster linker (macOS)
brew install michaeleisel/zld/zld
export RUSTFLAGS="-C link-arg=-fuse-ld=/usr/local/bin/zld"
```

==== Runtime Performance

```bash
# Enable link-time optimization
export RUSTFLAGS="-C lto=fat"

# Target specific CPU
export RUSTFLAGS="-C target-cpu=native"

# Use profile-guided optimization
cargo leptos build --release --profile pgo
```

== Verification

After installation, verify everything is working:

```bash
# Check Rust installation
rustc --version
cargo --version

# Check Node.js installation
node --version
npm --version

# Check cargo-leptos
cargo leptos --version

# Test build
cargo leptos build

# Test development server
cargo leptos watch
```

Visit `http://localhost:3000` to see your Typstify site running.

== Next Steps

- 📝 Create your first document
- ⚙️ Configure your site settings
- 🎨 Customize the theme
- 🚀 Deploy to production

== Getting Support

If you encounter issues during installation:

1. Check the [troubleshooting guide](/docs/troubleshooting)
2. Search [existing issues](https://github.com/longcipher/typstify/issues)
3. Create a new issue with your error details
4. Join our [community discussions](https://github.com/longcipher/typstify/discussions)
