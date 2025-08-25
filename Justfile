# Typstify SSG - Static Site Generator Commands

# Default recipe to display help
default:
    @echo "🚀 Typstify SSG Commands"
    @echo ""
    @echo "📖 Documentation Generation:"
    @echo "  just dev         - Generate documentation site with styles"
    @echo "  just docs        - Generate and serve documentation"
    @echo "  just serve       - Serve generated documentation"
    @echo ""
    @echo "⚙️  Build & Manage:"
    @echo "  just build       - Build static site"
    @echo "  just build-release - Build static site (release mode)"
    @echo "  just build-standalone - Build standalone binary for distribution"
    @echo "  just build-styles - Build Tailwind CSS styles"
    @echo "  just watch-styles - Watch and rebuild styles"
    @echo "  just clean       - Clean generated site and styles"
    @echo "  just new-content - Create new content file"
    @echo ""
    @echo "🛠️  Setup:"
    @echo "  just install     - Install all dependencies"
    @echo ""
    @echo "For all commands: just --list"

# Install dependencies
install:
    cargo build
    bun install

# Generate documentation site (default workflow)
dev: build-styles build
    @echo "📖 Generated documentation site with styles. Use 'just serve' to serve it."

# Generate and serve documentation
docs: build-styles build serve

# Build Tailwind CSS styles
build-styles:
    @echo "🎨 Building Tailwind CSS styles..."
    bunx tailwindcss --input ./style/input.css --output ./style/output.css --minify
    @echo "✅ Styles built successfully"

# Watch and rebuild styles on changes
watch-styles:
    @echo "👀 Watching for style changes..."
    bunx tailwindcss --input ./style/input.css --output ./style/output.css --watch

# Build static site using typstify-ssg
build: build-styles
    @echo "🚀 Building static site with typstify-ssg..."
    cargo run --bin typstify-ssg
    @echo "✅ Static site generated in site/ directory (CSS embedded in binary)"

# Build static site in release mode (optimized, with embedded CSS)
build-release: build-styles
    @echo "🚀 Building static site with typstify-ssg (release mode)..."
    cargo build --release --bin typstify-ssg
    @echo "✅ Release binary built in target/release/typstify-ssg with embedded CSS"

# Build standalone binary for distribution (includes embedded CSS)
build-standalone: build-styles
    @echo "🚀 Building standalone typstify-ssg binary..."
    cargo build --release --bin typstify-ssg
    @echo "📦 Creating standalone binary package..."
    mkdir -p dist
    cp target/release/typstify-ssg dist/
    @echo "✅ Standalone binary ready in dist/typstify-ssg"
    @echo "   This binary includes all required CSS files embedded"
    @echo "   Users can run it without external dependencies"

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
    rm -f style/output.css
    @echo "🧹 Cleaned site directory, dist directory, and generated styles"

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
