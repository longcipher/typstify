# Typstify SSG - Static Site Generator Commands

# Default recipe to display help
default:
    @echo "🚀 Typstify SSG Commands"
    @echo ""
    @echo "📖 Documentation Generation:"
    @echo "  just dev         - Generate documentation site with styles"
    @echo "  just docs        - Generate and serve documentation"
    @echo "  just serve       - Serve generated documentation on http://localhost:8080"
    @echo ""
    @echo "⚙️  Build & Manage:"
    @echo "  just build       - Build static site"
    @echo "  just build-styles - Build Tailwind CSS styles"
    @echo "  just watch-styles - Watch and rebuild styles"
    @echo "  just clean       - Clean generated site and styles"
    @echo "  just new-content - Create new content file"
    @echo ""
    @echo "🌐 Alternative Serve Commands:"
    @echo "  just serve-python - Serve with Python HTTP server directly"
    @echo "  just serve-release - Serve with optimized release binary"
    @echo ""
    @echo "🛠️  Setup:"
    @echo "  just install     - Install all dependencies"
    @echo ""
    @echo "🔧 Development Tools:"
    @echo "  just format      - Format code (taplo, cargo fmt)"
    @echo "  just lint        - Lint code (taplo, clippy, cargo machete)"
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
    bunx @tailwindcss/cli --input ./style/input.css --output ./style/output.css --minify
    @echo "✅ Styles built successfully"

# Watch and rebuild styles on changes
watch-styles:
    @echo "👀 Watching for style changes..."
    bunx @tailwindcss/cli --input ./style/input.css --output ./style/output.css --watch

# Build static site using typstify-ssg
build:
    @echo "🚀 Building static site with typstify-ssg..."
    cargo run --bin typstify-ssg
    @echo "✅ Static site generated in site/ directory"

# Build static site in release mode (optimized)
build-release:
    @echo "🚀 Building static site with typstify-ssg (release mode)..."
    cargo run --release --bin typstify-ssg
    @echo "✅ Static site generated in site/ directory"

# Serve the generated static site
serve:
    @echo "🌐 Serving static site on http://localhost:8080"
    cargo run --bin typstify-ssg serve --port 8080

# Serve with release binary (faster startup)
serve-release:
    @echo "🌐 Serving static site on http://localhost:8080"
    cargo run --release --bin typstify-ssg serve --port 8080

# Serve with Python HTTP server directly (alternative)
serve-python:
    @echo "🌐 Serving static site with Python on http://localhost:8080"
    cd site && python3 -m http.server 8080

# Serve SSG development site
serve-ssg-dev:
    @echo "🌐 Serving SSG site on port 8080..."
    @if [ ! -d "site" ]; then echo "❌ No site directory found. Run 'just build' first."; exit 1; fi
    cd site && python3 -m http.server 8080

# Clean generated site
clean:
    rm -rf site/
    rm -f style/output.css
    @echo "🧹 Cleaned site directory and generated styles"

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

# Format all code
format:
    @echo "🎨 Formatting code..."
    leptosfmt ./typstify-ssg/src
    taplo fmt
    cargo +nightly fmt --all
    @echo "✅ Code formatting complete"

# Lint all code
lint:
    @echo "🔍 Linting code..."
    taplo fmt --check
    cargo +nightly fmt --all -- --check
    cargo +nightly clippy --all -- -D warnings -A clippy::derive_partial_eq_without_eq -A clippy::unwrap_used -A clippy::uninlined_format_args -A clippy::manual_strip -A clippy::collapsible_if -A clippy::useless_format -A clippy::single_component_path_imports
    cargo machete
    @echo "✅ Code linting complete"
