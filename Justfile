# Development mode with live reload
dev: build-css
  cargo run -p typstify -- watch --open

# Production build
build: build-css
  cargo run -p typstify -- build

# Build CSS with Tailwind
build-css:
  bun run build:css

# Build WASM search module
build-wasm:
  cd crates/typstify-search-wasm && wasm-pack build --target web --release

# Run typstify CLI with arguments
run *ARGS:
  cargo run -p typstify -- {{ARGS}}

format:
  rumdl fmt .
  taplo fmt
  leptosfmt crates/
  cargo +nightly fmt --all
fix:
  rumdl check --fix .
lint:
  rumdl check .
  taplo fmt --check
  cargo +nightly fmt --all -- --check
  leptosfmt crates/ --check
  cargo +nightly clippy --all -- -D warnings -A clippy::derive_partial_eq_without_eq -D clippy::unwrap_used -D clippy::uninlined_format_args
  cargo machete
test:
  cargo test --all-features
test-coverage:
  cargo tarpaulin --all-features --workspace --timeout 300
check-cn:
  rg --line-number --column "\p{Han}"
# Full CI check
ci: lint test

# Publish all crates in dependency order
publish:
  for crate in typstify-core typstify-parser typstify-search typstify-search-wasm typstify-ui typstify-generator typstify; do \
    echo "Publishing $crate"; \
    cargo publish -p $crate; \
    sleep 10; \
  done