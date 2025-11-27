# Typstify SSG - Static Site Generator Commands

# Default recipe to display help
default:
    @echo "🚀 Typstify SSG Commands"
    @echo ""
    @echo "📖 Documentation Generation:"
    @echo "  just dev         - Generate documentation site"
    @echo "  just docs        - Generate and serve documentation"
    @echo "  just serve       - Serve generated documentation"
    @echo ""
    @echo "⚙️  Build & Manage:"
    @echo "  just build       - Build static site"
    @echo "  just build-release - Build static site (release mode)"
    @echo "  just build-standalone - Build standalone binary for distribution"
    @echo "  just clean       - Clean generated site"
    @echo "  just new-content - Create new content file"
    @echo ""
    @echo "🛠️  Setup:"
    @echo "  just install     - Install all dependencies"
    @echo ""
    @echo "For all commands: just --list"

# Install dependencies
install:
    cargo build

# Generate documentation site (default workflow)
dev: build
    @echo "📖 Generated documentation site. Use 'just serve' to serve it."

# Generate and serve documentation
docs: build serve

# Build static site using typstify-ssg
build:
    @echo "🚀 Building static site with typstify-ssg..."
    cargo run --bin typstify-ssg
    @echo "✅ Static site generated in site/ directory"

# Build static site in release mode (optimized, with embedded CSS)
build-release:
    @echo "🚀 Building static site with typstify-ssg (release mode)..."
    cargo build --release --bin typstify-ssg
    @echo "✅ Release binary built in target/release/typstify-ssg"

# Build standalone binary for distribution (includes embedded CSS)
build-standalone:
    @echo "🚀 Building standalone typstify-ssg binary..."
    cargo build --release --bin typstify-ssg
    @echo "📦 Creating standalone binary package..."
    mkdir -p dist
    cp target/release/typstify-ssg dist/
    @echo "✅ Standalone binary ready in dist/typstify-ssg"

# Serve the generated static site
serve:
    @echo "🌐 Serving static site..."
    cargo run --bin typstify-ssg serve

# Serve with release binary (faster startup)
serve-release:
    @echo "🌐 Serving static site..."
    cargo run --release --bin typstify-ssg serve

# Clean generated site
clean:
    rm -rf site/
    rm -rf dist/
    @echo "🧹 Cleaned site directory and dist directory"

# Create a new content file
new-content name:
    @echo "Creating new content file: contents/{{name}}.typ"
    @touch contents/{{name}}.typ
    @echo '// title: {{name}}' >> contents/{{name}}.typ
    @echo '// description: Description for {{name}}' >> contents/{{name}}.typ
    @echo '// tags: example' >> contents/{{name}}.typ
    @echo '' >> contents/{{name}}.typ
    @echo '= {{name}}' >> contents/{{name}}.typ
    @echo '' >> contents/{{name}}.typ
    @echo 'Content for {{name}}.' >> contents/{{name}}.typ
    @echo "✅ Created contents/{{name}}.typ"

# Set up the project for first time
setup: install
    @echo "🎉 Typstify SSG setup complete!"
    @echo "Run 'just dev' to generate documentation"
