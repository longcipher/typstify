# Typstify - Task List

> Detailed, actionable tasks organized by development phases.

## Phase 0: Project Initialization

### P0.1 Workspace Setup

- [x] **P0.1.1** Create root `Cargo.toml` with workspace configuration
  - Define `[workspace]` with members: `bin/*`, `crates/*`
  - Set resolver = "3", edition = "2024"
  - Add all dependencies to `[workspace.dependencies]` (versions only, no features)

- [x] **P0.1.2** Create root `package.json` for Bun workspace

  ```json
  {
    "name": "typstify",
    "private": true,
    "workspaces": ["crates/typstify-ui", "crates/typstify-search-wasm"]
  }
  ```

- [x] **P0.1.3** Initialize directory structure

  ```text
  mkdir -p bin/typstify/src/cmd
  mkdir -p crates/typstify-core/src
  mkdir -p crates/typstify-parser/src
  mkdir -p crates/typstify-generator/src
  mkdir -p crates/typstify-search/src
  mkdir -p crates/typstify-search-wasm/src
  mkdir -p crates/typstify-ui/src
  mkdir -p style
  mkdir -p assets
  ```

- [x] **P0.1.4** Create individual crate `Cargo.toml` files
  - Each must use `version.workspace = true`, `edition.workspace = true`
  - Dependencies use `dep.workspace = true` with optional `features = [...]`

- [x] **P0.1.5** Setup Tailwind CSS v4
  - Create `style/main.css` with `@import "tailwindcss"` and `@theme` block
  - Add `tailwindcss` v4 to root `package.json` devDependencies
  - Configure `bun run build:css` script

### P0.2 Justfile Configuration

- [x] **P0.2.1** Update `Justfile` with build commands

  ```just
  # Development mode with live reload
  dev: build-css build-wasm
    cargo run -p typstify -- watch --open

  # Production build
  build: build-css build-wasm
    cargo run -p typstify -- build

  # Build CSS
  build-css:
    bun run build:css

  # Build WASM search module
  build-wasm:
    cd crates/typstify-search-wasm && wasm-pack build --target web --release

  # Run typstify CLI
  run *ARGS:
    cargo run -p typstify -- {{ARGS}}

  # Existing commands...
  format:
    # ...
  ```

### P0.3 CI/CD Setup

- [ ] **P0.3.1** Create `.github/workflows/ci.yml`
  - Install Rust stable + wasm32-unknown-unknown target
  - Install Bun
  - Run `just lint`, `just test`

- [ ] **P0.3.2** Create `.github/workflows/deploy.yml`
  - Build full site
  - Deploy to Cloudflare Pages

---

## Phase 1: Core Library (`typstify-core`)

### P1.1 Error Types

- [x] **P1.1.1** Create `crates/typstify-core/src/error.rs`
  - Define `CoreError` enum using `thiserror`
  - Variants: `Config`, `Parse`, `Template`, `Io`, `Search`
  - Implement `From` traits for common error types

### P1.2 Configuration

- [x] **P1.2.1** Create `crates/typstify-core/src/config.rs`
  - Define `Config` struct with nested sections:
    - `SiteConfig`: title, base_url, default_language, languages
    - `BuildConfig`: output_dir, minify, syntax_theme
    - `SearchConfig`: enabled, index_fields, chunk_size
    - `RssConfig`: enabled, limit
  - Implement `Config::load(path: &Path) -> Result<Self>`
  - Use `config` crate with TOML format

- [ ] **P1.2.2** Add hot-reload support for dev mode
  - Wrap `Config` in `ArcSwap`
  - Implement `ConfigWatcher` using `notify` crate
  - Expose `config()` function that returns current config

### P1.3 Content Types

- [x] **P1.3.1** Create `crates/typstify-core/src/content.rs`
  - Define `ContentType` enum: `Markdown`, `Typst`
  - Define `ContentPath` struct with path, lang, slug parsing
  - Define `ParsedContent` struct: metadata, html, raw_content
  - Define `Page` struct: url, title, date, content, etc.

- [x] **P1.3.2** Create `crates/typstify-core/src/frontmatter.rs`
  - Define `Frontmatter` struct with all fields (serde)
  - Support both YAML and TOML frontmatter
  - Handle optional fields with defaults

### P1.4 Tests

- [x] **P1.4.1** Add unit tests for config loading
- [x] **P1.4.2** Add unit tests for content path parsing
- [x] **P1.4.3** Add unit tests for frontmatter parsing

---

## Phase 2: Content Parser (`typstify-parser`)

### P2.1 Markdown Parser

- [x] **P2.1.1** Create `crates/typstify-parser/src/markdown.rs`
  - Implement `MarkdownParser` struct
  - Configure `pulldown-cmark` options (tables, footnotes, strikethrough)
  - Split frontmatter from body content
  - Implement HTML rendering with custom code block handler

- [x] **P2.1.2** Integrate syntax highlighting
  - Create `SyntaxHighlighter` in `src/syntax.rs`
  - Load `syntect` syntax set and themes
  - Process code blocks during markdown parsing
  - Support language detection from code fence

### P2.2 Typst Parser

- [x] **P2.2.1** Create `crates/typstify-parser/src/typst_parser.rs`
  - Implement `TypstParser` struct
  - Extract frontmatter from Typst comment syntax
  - Extract TOC from heading patterns
  - Note: Full compilation deferred to generator phase

- [ ] **P2.2.2** Implement Typst-to-HTML conversion (deferred to generator)
  - Create `TypstHtmlRenderer` for common elements:
    - Headings (h1-h6)
    - Paragraphs
    - Lists (ordered/unordered)
    - Code blocks
    - Images and figures
    - Tables
    - Math blocks (render as KaTeX-compatible HTML)
  - Handle unknown elements with fallback

### P2.3 Parser Interface

- [x] **P2.3.1** Create unified parser interface in `src/lib.rs`

  ```rust
  pub trait ContentParser {
      fn parse(&self, content: &str, path: &Path) -> Result<ParsedContent>;
  }
  ```

- [x] **P2.3.2** Implement `ParserRegistry`
  - Auto-detect parser from file extension
  - `.md` → MarkdownParser
  - `.typ` → TypstParser

### P2.4 Tests

- [x] **P2.4.1** Add markdown parsing tests
  - Test frontmatter extraction
  - Test code block highlighting
  - Test various markdown features

- [x] **P2.4.2** Add Typst parsing tests
  - Test basic element conversion
  - Test TOC extraction
  - Test error handling

---

## Phase 3: Static Site Generator (`typstify-generator`)

### P3.1 HTML Generation

- [x] **P3.1.1** Create `crates/typstify-generator/src/html.rs`
  - Implement `HtmlGenerator` struct
  - Generate page HTML with metadata injection
  - Handle URL alias generation (redirect HTML files)
  - Inject custom JS/CSS from frontmatter

- [x] **P3.1.2** Create base HTML template system (`src/template.rs`)
  - Define `Template` struct with variable interpolation
  - Implement default templates: base, page, post, list, taxonomy, redirect
  - Support optional variables with `{{ var? }}` syntax
  - Use string interpolation (lightweight, no heavy template engines)

### P3.2 Content Collection

- [x] **P3.2.1** Create `crates/typstify-generator/src/collector.rs`
  - Implement `ContentCollector` to walk content directory
  - Parse all content files in parallel using `rayon`
  - Build page hierarchy from directory structure
  - Handle draft filtering based on config

- [x] **P3.2.2** Implement taxonomy collection
  - Extract tags and categories from pages
  - Build taxonomy term pages
  - Generate taxonomy list pages with pagination

### P3.3 Feed Generation

- [x] **P3.3.1** Create `crates/typstify-generator/src/rss.rs`
  - Implement RSS feed generation using `rss` crate
  - Sort posts by date, apply limit from config
  - Generate per-language feeds if multi-language

- [x] **P3.3.2** Create `crates/typstify-generator/src/sitemap.rs`
  - Generate XML sitemap with all pages
  - Include lastmod dates
  - Handle multi-language alternate links

### P3.4 Asset Processing

- [x] **P3.4.1** Create `crates/typstify-generator/src/assets.rs`
  - Copy static assets to output directory
  - Handle asset fingerprinting (content hash in filename)
  - Generate asset manifest for cache busting

### P3.5 Build Orchestrator

- [x] **P3.5.1** Create `crates/typstify-generator/src/build.rs`
  - Implement `Builder` struct
  - Orchestrate full build pipeline:
    1. Clean output directory
    2. Collect and parse content
    3. Generate HTML pages
    4. Generate taxonomy pages
    5. Generate redirects
    6. Generate RSS/sitemap
    7. Copy assets
  - Return `BuildStats` with metrics

- [ ] **P3.5.2** Implement incremental build support (deferred)
  - Track file modification times
  - Skip unchanged content
  - Invalidate dependents on change

### P3.6 Tests

- [x] **P3.6.1** Add HTML generation tests (7 tests)
- [x] **P3.6.2** Add taxonomy/collector tests (3 tests)
- [x] **P3.6.3** Add RSS/sitemap generation tests (8 tests)
- [x] **P3.6.4** Add integration test for full build (3 tests)

---

## Phase 4: Search Index (`typstify-search`)

### P4.1 Tantivy Schema

- [x] **P4.1.1** Create `crates/typstify-search/src/schema.rs`
  - Define search schema with fields:
    - `title` (TEXT | STORED)
    - `body` (TEXT)
    - `url` (STRING | STORED)
    - `lang` (STRING | STORED | FAST)
    - `tags` (TEXT | STORED)
    - `date` (DATE | STORED | FAST)
  - Export schema creation function
  - Register custom tokenizers (lowercase normalization)

### P4.2 Indexer

- [x] **P4.2.1** Create `crates/typstify-search/src/indexer.rs`
  - Implement `SearchIndexer` struct
  - Create index from pages collection
  - Handle text extraction from HTML (strip tags, scripts, styles)
  - Configure tokenizers for supported languages

- [x] **P4.2.2** Implement index commit and optimization
  - Commit index after all documents added
  - Merge segments for optimal query performance
  - Calculate and log index statistics via `IndexStats`

### P4.3 Index Chunking

- [x] **P4.3.1** Create `crates/typstify-search/src/chunker.rs`
  - Implement `IndexChunker` struct
  - Split large index files into chunks (default 64KB)
  - Generate chunk manifest JSON
  - Support reassembling chunks for verification

- [x] **P4.3.2** Generate search manifest

  ```rust
  pub struct IndexManifest {
      pub version: u32,
      pub chunk_size: usize,
      pub total_size: u64,
      pub files: HashMap<String, FileManifest>,
  }

  pub struct FileManifest {
      pub size: usize,
      pub chunks: Vec<String>,
  }
  ```

### P4.4 Simplified Index (Fallback)

- [x] **P4.4.1** Create `crates/typstify-search/src/simple.rs`
  - Implement `SimpleSearchIndex` for small sites
  - JSON-based format, loads entirely in WASM
  - Pre-tokenize terms at build time
  - Target < 500KB for most sites (MAX_SIMPLE_INDEX_SIZE)
  - Support AND-based multi-term search

### P4.5 Tests

- [x] **P4.5.1** Add schema tests (4 tests)
- [x] **P4.5.2** Add indexer tests with sample content (6 tests)
- [x] **P4.5.3** Add chunker tests verifying byte alignment (5 tests)
- [x] **P4.5.4** Add simple index tests (7 tests)

---

## Phase 5: WASM Search Runtime (`typstify-search-wasm`)

### P5.1 WASM Setup

- [x] **P5.1.1** Configure `Cargo.toml` for WASM

  ```toml
  [lib]
  crate-type = ["cdylib", "rlib"]

  [dependencies]
  wasm-bindgen = { workspace = true }
  wasm-bindgen-futures = { workspace = true }
  js-sys = { workspace = true }
  gloo-net = { workspace = true }
  serde-wasm-bindgen = { workspace = true }
  console_error_panic_hook = { workspace = true }
  scc = { workspace = true }
  ```

- [x] **P5.1.2** Create `package.json` for WASM crate
  - Configure wasm-pack build output
  - Add to root Bun workspace

### P5.2 HTTP Directory

- [x] **P5.2.1** Create `crates/typstify-search-wasm/src/directory.rs`
  - Implement `HttpDirectory` struct
  - Load manifest on initialization via async `new()`
  - Implement chunk caching using `scc::HashMap`

- [x] **P5.2.2** Implement Range request fetching
  - Calculate required chunk indices from byte range
  - Fetch chunks via `gloo-net` Request
  - Concatenate and extract exact byte range
  - Cache fetched chunks with `load_chunk()`

### P5.3 Search Engine

- [x] **P5.3.1** Create `crates/typstify-search-wasm/src/lib.rs`
  - Implement module exports and initialization
  - Expose `get_version()` and `is_ready()` helpers
  - Export `SimpleSearchEngine` with `#[wasm_bindgen]`
  - Return results as JS-compatible objects via `serde_wasm_bindgen`

- [x] **P5.3.2** Create `crates/typstify-search-wasm/src/query.rs`
  - Implement `SearchQuery::parse()` for query parsing
  - Implement `score_document()` for relevance scoring
  - Implement `generate_snippet()` for result highlighting
  - Return `SearchResults` with scored results

### P5.4 Simple Search Fallback

- [x] **P5.4.1** Implement simple search for small indices (`simple.rs`)
  - Load entire `SimpleSearchIndex` JSON via `SimpleSearchEngine::load()`
  - Implement basic term matching with inverted index
  - Score results by term frequency (title matches weighted higher)
  - Support `search()` and `fromJson()` methods

### P5.5 Tests

- [x] **P5.5.1** Add unit tests (18 tests passing)
  - Directory tests (3): manifest deserialization, error handling
  - Query tests (7): parsing, scoring, snippets, serialization
  - Simple search tests (6): search, multi-results, empty queries
  - Lib tests (2): version, ready check
- [ ] **P5.5.2** Add browser integration tests (deferred - requires wasm-pack setup)

---

## Phase 6: UI Components (`typstify-ui`)

### P6.1 Leptos Setup

- [x] **P6.1.1** Configure `Cargo.toml` for Leptos CSR

  ```toml
  [dependencies]
  leptos = { workspace = true, features = ["csr"] }
  ```

- [x] **P6.1.2** Create `package.json` for UI crate
  - Add to root Bun workspace

### P6.2 Search Component

- [x] **P6.2.1** Create `crates/typstify-ui/src/search.rs`
  - Implement `SearchBox` component
  - Add debounced input handling (300ms delay)
  - Show loading state during search
  - Handle empty/error states

- [x] **P6.2.2** Implement `SearchResults` component
  - Render list of search results
  - Display title, URL, and summary snippet
  - Highlight matching terms
  - Handle "no results" message

- [x] **P6.2.3** Implement `SearchModal` component
  - Keyboard-triggered modal (Cmd/Ctrl + K)
  - Focus trap within modal
  - Close on Escape or outside click

### P6.3 Article Component

- [x] **P6.3.1** Create `crates/typstify-ui/src/article.rs`
  - Implement `Article` component
  - Render HTML content safely
  - Inject custom CSS links
  - Inject custom JS scripts (deferred)

### P6.4 Navigation Components

- [x] **P6.4.1** Create `crates/typstify-ui/src/navigation.rs`
  - Implement `Navigation` component
  - Handle active link highlighting
  - Support multi-level navigation

- [x] **P6.4.2** Implement `TableOfContents` component
  - Extract headings from article
  - Generate anchor links
  - Highlight current section on scroll

### P6.5 Styling

- [ ] **P6.5.1** Create component styles in `style/main.css`
  - Search container styles
  - Result item styles
  - Modal overlay styles
  - Navigation styles
  - Article prose styles

### P6.6 Tests

- [x] **P6.6.1** Add component unit tests (10 tests passing)
- [ ] **P6.6.2** Add Leptos component integration tests (deferred - requires browser)

---

## Phase 7: CLI Binary (`typstify`)

> Single binary with all commands. No separate server binary.

### P7.1 CLI Structure

- [x] **P7.1.1** Create `bin/typstify/Cargo.toml`

  ```toml
  [package]
  name = "typstify"
  version.workspace = true
  edition.workspace = true

  [[bin]]
  name = "typstify"
  path = "src/main.rs"

  [dependencies]
  typstify-core = { workspace = true }
  typstify-parser = { workspace = true }
  typstify-generator = { workspace = true }
  typstify-search = { workspace = true }
  clap = { workspace = true, features = ["derive"] }
  eyre = { workspace = true }
  color-eyre = { workspace = true }
  tracing = { workspace = true }
  tracing-subscriber = { workspace = true, features = ["env-filter"] }
  tokio = { workspace = true, features = ["full"] }
  axum = { workspace = true }
  tower-http = { workspace = true, features = ["fs", "cors"] }
  notify = { workspace = true }
  open = { workspace = true }
  ```

- [x] **P7.1.2** Create `bin/typstify/src/main.rs`
  - Define `Cli` struct with clap derive
  - Define `Commands` enum: `Build`, `Watch`, `New`, `Check`
  - Global flags: `--config`, `--verbose`
  - Initialize tracing based on verbosity
  - Dispatch to command modules

- [x] **P7.1.3** Create `bin/typstify/src/cmd/mod.rs`
  - Export all command modules

  ```rust
  pub mod build;
  pub mod watch;
  pub mod new;
  pub mod check;
  ```

### P7.2 Build Command

- [x] **P7.2.1** Create `bin/typstify/src/cmd/build.rs`
  - Load configuration from `--config` path
  - Call `typstify-generator::Builder::build()`
  - Handle `--drafts` flag to include draft posts
  - Print build statistics (pages, time, index size)
  - Return proper exit code on failure

### P7.3 Watch Command (Embedded Dev Server)

- [x] **P7.3.1** Create `bin/typstify/src/cmd/watch.rs`
  - Initial build with drafts enabled
  - Setup `notify` file watcher on content/templates/style
  - Debounce file events (200ms)
  - Rebuild on change and log results

- [x] **P7.3.2** Create `bin/typstify/src/server.rs`
  - Setup axum Router
  - Serve static files from output directory
  - Add `/__livereload` SSE endpoint for reload notifications
  - Inject livereload script into HTML responses

- [x] **P7.3.3** Implement live reload client
  - Create JS snippet for EventSource connection
  - Auto-reload page on server event
  - Inject script into `</body>` during dev builds

- [x] **P7.3.4** Handle `--open` flag
  - Use `open` crate to launch default browser
  - Open `http://127.0.0.1:{port}` after server starts

### P7.4 New Command

- [x] **P7.4.1** Create `bin/typstify/src/cmd/new.rs`
  - Parse path argument to determine location
  - Support `--template` flag: `post`, `page`, `typst`
  - Generate frontmatter with current date
  - Create file with appropriate extension (.md or .typ)
  - Print created file path

### P7.5 Check Command

- [x] **P7.5.1** Create `bin/typstify/src/cmd/check.rs`
  - Validate configuration file syntax
  - Check all content files parse correctly
  - Verify frontmatter required fields
  - Check for broken internal links
  - Report warnings and errors
  - `--strict` treats warnings as errors

### P7.6 Initialization & Error Handling

- [x] **P7.6.1** Setup error handling
  - Install `color_eyre` for colored error reports
  - Configure panic hook for debugging
  - Wrap all errors with context using `.wrap_err()`

- [x] **P7.6.2** Setup tracing
  - Initialize `tracing_subscriber` with fmt layer
  - Map `--verbose` count to log level:
    - 0: `warn`
    - 1: `info`
    - 2: `debug`
    - 3+: `trace`

### P7.7 Tests

- [x] **P7.7.1** Add CLI argument parsing tests (7 tests)
- [ ] **P7.7.2** Add build command integration tests (deferred - requires sample content)
- [ ] **P7.7.3** Add watch command tests (deferred - requires async test harness)
- [ ] **P7.7.4** Add new command tests (deferred - requires temp directory)
- [ ] **P7.7.5** Add check command tests (deferred - requires sample content)

---

## Phase 8: Integration & Polish

### P8.1 End-to-End Testing

- [x] **P8.1.1** Create sample site in `examples/`
  - Multiple markdown posts
  - Typst document
  - Multi-language content
  - Various frontmatter configurations

- [x] **P8.1.2** Add E2E test suite
  - Build sample site
  - Verify HTML output
  - Verify search index
  - Verify RSS/sitemap

### P8.2 Documentation

- [x] **P8.2.1** Create `README.md`
  - Project overview
  - Installation instructions
  - Quick start guide
  - Configuration reference

- [x] **P8.2.2** Create `docs/configuration.md`
  - Full configuration options
  - Examples for each option

- [x] **P8.2.3** Create `docs/content-format.md`
  - Markdown features
  - Typst features
  - Frontmatter reference

### P8.3 Performance Optimization

- [ ] **P8.3.1** Profile build performance
  - Use `cargo-flamegraph`
  - Identify bottlenecks
  - Optimize hot paths

- [ ] **P8.3.2** Optimize WASM size
  - Configure `wasm-opt` in build
  - Enable LTO for release builds
  - Target < 200KB gzipped

- [ ] **P8.3.3** Implement caching strategies
  - Content hash-based caching
  - Incremental build support
  - Search index chunk caching

### P8.4 Error Messages

- [ ] **P8.4.1** Improve error messages
  - Add file paths to errors
  - Add line numbers where applicable
  - Suggest fixes for common issues

---

## Phase 9: Release Preparation

### P9.1 Versioning

- [ ] **P9.1.1** Configure version management
  - Use `cargo-release` for versioning
  - Setup changelog generation

### P9.2 Distribution

- [ ] **P9.2.1** Setup binary releases
  - GitHub Actions for multi-platform builds
  - Linux (x86_64, aarch64)
  - macOS (x86_64, aarch64)
  - Windows (x86_64)

- [ ] **P9.2.2** Publish to crates.io
  - Ensure all crates have proper metadata
  - Add license files
  - Add repository links

### P9.3 Example Site

- [ ] **P9.3.1** Create demo site
  - Showcase all features
  - Deploy to Cloudflare Pages
  - Link from README

---

## Dependency Reference

### Workspace Dependencies (root Cargo.toml)

```toml
[workspace.dependencies]
# CLI & Config
clap = "4.5.57"
config = "0.15.19"
eyre = "0.6.12"
color-eyre = "0.6.5"
thiserror = "2.0.18"

# Async & Runtime
tokio = "1.49.0"

# Serialization
serde = "1.0.228"
serde_json = "1.0.149"
toml = "0.9.11"

# Content Processing
pulldown-cmark = "0.13.0"
typst = "0.14.2"
syntect = "5.3.0"

# Search
tantivy = "0.25.0"

# WASM
wasm-bindgen = "0.2.108"
gloo-net = "0.6.0"
serde-wasm-bindgen = "0.6.5"
console_error_panic_hook = "0.1.7"

# Web & Frontend
leptos = "0.8.15"
leptos_meta = "0.8.5"
leptos_router = "0.8.11"

# Dev Server (embedded in CLI)
axum = "0.8.8"
tower-http = "0.6.8"

# Utilities
scc = "3.5.6"
arc-swap = "1.8.1"
rayon = "1.11.0"
notify = "8.2.0"
chrono = "0.4.43"
rss = "2.0.12"
open = "5.3.3"

# Observability
tracing = "0.1.44"
tracing-subscriber = "0.3.22"
```

---

## Notes

### Critical Constraints

1. **Single Binary** - All commands in one `typstify` binary, no separate server
2. **No Backend** - Pure static site generator, dev server is for local preview only
3. **No `anyhow`** - Use `eyre` for apps, `thiserror` for libs
4. **No `reqwest` in WASM** - Use `gloo-net` for browser HTTP
5. **No `trunk`** - Use `wasm-pack` for WASM build
6. **English only** - All code, comments, and docs in English
7. **Workspace inheritance** - Sub-crates use `workspace = true`

### Testing Strategy

- Unit tests in same file as code (`#[cfg(test)] mod tests`)
- Integration tests in `tests/` directory
- E2E tests using sample site in `examples/`
- WASM tests via `wasm-pack test --headless`

### Development Commands

```bash
just dev          # Start watch mode with live reload
just build        # Production build
just format       # Format all code
just lint         # Run all linters
just test         # Run all tests
just build-wasm   # Build WASM module only
just build-css    # Build Tailwind CSS only
just run build    # Run build command directly
just run watch    # Run watch command directly
just run new posts/my-article  # Create new content
just run check    # Validate config and content
```
