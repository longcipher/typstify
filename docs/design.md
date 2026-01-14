# Typstify - Design Document

> A high-performance static site generator with Typst/Markdown support and client-side search.

## 1. Overview

Typstify is a Rust-based static site generator inspired by Zola and Hugo, featuring:

- **Single Binary Distribution**: All functionality in one executable
- **Typst & Markdown Support**: First-class support for both formats
- **Client-Side Search**: WASM-powered search without backend dependencies
- **Modern Frontend**: Leptos CSR with Tailwind CSS v4

## 2. Architecture

The system is divided into two main components:

```text
┌─────────────────────────────────────────────────────────────────┐
│                        BUILD TIME                                │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐  │
│  │   Content   │───▶│  Generator  │───▶│   Static Output     │  │
│  │  (.md/.typ) │    │   (Rust)    │    │  (HTML/CSS/JS/idx)  │  │
│  └─────────────┘    └─────────────┘    └─────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                        RUNTIME                                   │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐  │
│  │   Browser   │◀──▶│ Leptos CSR  │◀──▶│   Search WASM       │  │
│  │             │    │   (WASM)    │    │   (Tantivy-lite)    │  │
│  └─────────────┘    └─────────────┘    └─────────────────────┘  │
│                            │                      │              │
│                            ▼                      ▼              │
│                     ┌─────────────────────────────────┐          │
│                     │   Static Files (CDN/Pages)      │          │
│                     │   - HTML pages                  │          │
│                     │   - Search index chunks         │          │
│                     └─────────────────────────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

## 3. Project Structure

```text
typstify/
├── Cargo.toml                    # Workspace root
├── package.json                  # Bun workspace root
├── bun.lockb                     # Single lockfile
├── Justfile                      # Task runner
├── README.md
│
├── style/                        # Global styles
│   └── main.css                  # Tailwind v4 entry
│
├── assets/                       # Static assets
│   └── favicon.ico
│
├── bin/
│   └── typstify/                 # Single CLI binary (all commands)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs           # Entry point
│           ├── cmd/
│           │   ├── mod.rs
│           │   ├── build.rs      # Build command
│           │   ├── watch.rs      # Watch/dev server command
│           │   ├── new.rs        # New content command
│           │   └── check.rs      # Check command
│           └── server.rs         # Embedded dev server
│
└── crates/
    ├── typstify-core/            # Core library
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── config.rs         # Site configuration
    │       ├── content.rs        # Content parsing
    │       └── error.rs          # Error types
    │
    ├── typstify-parser/          # Content parsers
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── markdown.rs       # pulldown-cmark wrapper
    │       ├── typst.rs          # typst crate integration
    │       ├── frontmatter.rs    # YAML/TOML parsing
    │       └── syntax.rs         # syntect highlighting
    │
    ├── typstify-generator/       # Static site generation
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── html.rs           # HTML generation
    │       ├── rss.rs            # RSS feed generation
    │       ├── sitemap.rs        # Sitemap generation
    │       └── assets.rs         # Asset processing
    │
    ├── typstify-search/          # Search index generation
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── indexer.rs        # Tantivy indexing
    │       ├── schema.rs         # Index schema
    │       └── chunker.rs        # Index chunking for CDN
    │
    ├── typstify-search-wasm/     # WASM search runtime
    │   ├── Cargo.toml
    │   ├── package.json
    │   └── src/
    │       ├── lib.rs
    │       ├── directory.rs      # Custom Directory for HTTP
    │       └── query.rs          # Query execution
    │
    └── typstify-ui/              # Leptos UI components
        ├── Cargo.toml
        ├── package.json
        └── src/
            ├── lib.rs
            ├── search.rs         # Search component
            ├── navigation.rs     # Nav components
            └── article.rs        # Article renderer
```

## 4. Build-Time Generator

### 4.1 CLI Interface

```bash
# Build the site (production)
typstify build [--output <dir>] [--config <file>] [--drafts]

# Development mode with live reload (embedded server)
typstify watch [--port <port>] [--open]

# Create new content
typstify new <path> [--template <name>]

# Check/validate configuration and content
typstify check [--strict]

# Show version and build info
typstify --version
```

**Implementation**: Use `clap` for argument parsing with derive macros. All commands in single binary.

```rust
#[derive(Parser)]
#[command(name = "typstify", version, about)]
struct Cli {
    #[arg(short, long, default_value = "config.toml")]
    config: PathBuf,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the static site for production
    Build {
        #[arg(short, long, default_value = "public")]
        output: PathBuf,
        /// Include draft posts
        #[arg(long)]
        drafts: bool,
    },
    /// Start development server with live reload
    Watch {
        #[arg(short, long, default_value_t = 3000)]
        port: u16,
        /// Open browser automatically
        #[arg(long)]
        open: bool,
    },
    /// Create new content from template
    New {
        /// Path for the new content (e.g., posts/my-article)
        path: PathBuf,
        #[arg(short, long, default_value = "post")]
        template: String,
    },
    /// Validate configuration and content
    Check {
        /// Treat warnings as errors
        #[arg(long)]
        strict: bool,
    },
}
```

### 4.2 Configuration

Site configuration in `config.toml`:

```toml
[site]
title = "My Site"
base_url = "https://example.com"
default_language = "en"
languages = ["en", "zh"]

[build]
output_dir = "public"
minify = true
syntax_theme = "OneHalfDark"

[search]
enabled = true
index_fields = ["title", "body", "tags"]
chunk_size = 65536  # 64KB chunks for Range requests

[rss]
enabled = true
limit = 20

[taxonomies]
tags = { paginate = 10 }
categories = { paginate = 10 }
```

**Implementation**: Use `config` crate with `ArcSwap` for hot-reloading in dev mode.

### 4.3 Content Parsing

#### 4.3.1 Markdown Parser

```rust
use pulldown_cmark::{Parser, Options, html};

pub struct MarkdownParser {
    options: Options,
    highlighter: SyntaxHighlighter,
}

impl MarkdownParser {
    pub fn parse(&self, content: &str) -> Result<ParsedContent> {
        let (frontmatter, body) = self.split_frontmatter(content)?;
        let metadata = self.parse_frontmatter(frontmatter)?;
        let html = self.render_html(body)?;
        Ok(ParsedContent { metadata, html })
    }
}
```

#### 4.3.2 Typst Parser

```rust
use typst::eval::Tracer;
use typst::World;

pub struct TypstParser {
    world: TypstWorld,
}

impl TypstParser {
    pub fn parse(&self, content: &str) -> Result<ParsedContent> {
        let document = typst::compile(&self.world)?;
        let html = self.document_to_html(&document)?;
        Ok(ParsedContent { metadata, html })
    }
}
```

**Note**: Typst to HTML conversion requires custom implementation since Typst primarily targets PDF. We'll implement a simplified HTML renderer for common elements.

#### 4.3.3 Syntax Highlighting

Pre-render code blocks at build time using `syntect`:

```rust
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::html::highlighted_html_for_string;

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    default_theme: String,
}

impl SyntaxHighlighter {
    pub fn highlight(&self, code: &str, lang: &str) -> Result<String> {
        let syntax = self.syntax_set
            .find_syntax_by_token(lang)
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let theme = &self.theme_set.themes[&self.default_theme];
        Ok(highlighted_html_for_string(code, &self.syntax_set, syntax, theme)?)
    }
}
```

### 4.4 Multi-language Support (i18n)

Language detection based on file structure:

```text
content/
├── posts/
│   ├── hello-world/
│   │   ├── index.md      # Default language (en)
│   │   └── index.zh.md   # Chinese translation
│   └── another-post.md   # Single language
```

```rust
pub struct ContentPath {
    pub path: PathBuf,
    pub lang: Option<String>,
    pub slug: String,
}

impl ContentPath {
    pub fn from_path(path: &Path, default_lang: &str) -> Self {
        // Parse "index.zh.md" -> lang: Some("zh")
        // Parse "index.md" -> lang: None (uses default)
    }
}
```

### 4.5 Frontmatter Schema

```yaml
---
title: "Article Title"
date: 2026-01-14
updated: 2026-01-15
draft: false
tags: ["rust", "wasm"]
categories: ["programming"]
aliases: ["/old-url", "/another-old-url"]
custom_js: ["/js/chart.js"]
custom_css: ["/css/custom.css"]
template: "post.html"
weight: 10
---
```

```rust
#[derive(Debug, Deserialize)]
pub struct Frontmatter {
    pub title: String,
    pub date: DateTime<Utc>,
    #[serde(default)]
    pub updated: Option<DateTime<Utc>>,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub custom_js: Vec<String>,
    #[serde(default)]
    pub custom_css: Vec<String>,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub weight: i32,
}
```

### 4.6 Search Index Generation

#### 4.6.1 Tantivy Schema

```rust
use tantivy::schema::*;

pub fn create_search_schema() -> Schema {
    let mut schema_builder = Schema::builder();

    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT);
    schema_builder.add_text_field("url", STRING | STORED);
    schema_builder.add_text_field("lang", STRING | STORED);
    schema_builder.add_text_field("tags", TEXT | STORED);
    schema_builder.add_date_field("date", INDEXED | STORED);

    schema_builder.build()
}
```

#### 4.6.2 Index Chunking Strategy

For efficient HTTP Range requests, split the index into chunks:

```rust
pub struct IndexChunker {
    chunk_size: usize,  // Default: 64KB
}

impl IndexChunker {
    /// Split index files into chunks and generate manifest
    pub fn chunk_index(&self, index_path: &Path, output_dir: &Path) -> Result<IndexManifest> {
        let manifest = IndexManifest::new();

        for entry in fs::read_dir(index_path)? {
            let file = entry?;
            let chunks = self.split_file(&file.path())?;
            manifest.add_file(file.file_name(), chunks);
        }

        manifest.write(output_dir.join("search-manifest.json"))?;
        Ok(manifest)
    }
}
```

**Manifest format** (`search-manifest.json`):

```json
{
  "version": 1,
  "chunk_size": 65536,
  "files": {
    "terms.idx": {
      "size": 524288,
      "chunks": ["terms.idx.0", "terms.idx.1", "terms.idx.2"]
    },
    "store.idx": {
      "size": 1048576,
      "chunks": ["store.idx.0", "store.idx.1", "store.idx.2"]
    }
  }
}
```

### 4.7 RSS Generation

```rust
use rss::{ChannelBuilder, ItemBuilder};

pub fn generate_rss(config: &Config, posts: &[Post]) -> Result<String> {
    let items: Vec<_> = posts.iter()
        .take(config.rss.limit)
        .map(|post| {
            ItemBuilder::default()
                .title(Some(post.title.clone()))
                .link(Some(format!("{}{}", config.site.base_url, post.url)))
                .description(post.summary.clone())
                .pub_date(Some(post.date.to_rfc2822()))
                .build()
        })
        .collect();

    let channel = ChannelBuilder::default()
        .title(&config.site.title)
        .link(&config.site.base_url)
        .items(items)
        .build();

    Ok(channel.to_string())
}
```

## 5. Runtime (WASM Search)

### 5.1 WASM Module Architecture

The search WASM module provides client-side search without server dependencies.

**Critical Decision**: Since `hpx` doesn't support WASM, we use `gloo-net` for browser HTTP requests.

```rust
// typstify-search-wasm/src/lib.rs

use wasm_bindgen::prelude::*;
use gloo_net::http::Request;

#[wasm_bindgen]
pub struct SearchEngine {
    manifest: IndexManifest,
    cache: scc::HashMap<String, Vec<u8>>,
}

#[wasm_bindgen]
impl SearchEngine {
    #[wasm_bindgen(constructor)]
    pub async fn new(base_url: String) -> Result<SearchEngine, JsValue> {
        let manifest = Self::load_manifest(&base_url).await?;
        Ok(Self {
            manifest,
            cache: scc::HashMap::new(),
        })
    }

    #[wasm_bindgen]
    pub async fn search(&self, query: &str, limit: usize) -> Result<JsValue, JsValue> {
        let results = self.execute_query(query, limit).await?;
        Ok(serde_wasm_bindgen::to_value(&results)?)
    }
}
```

### 5.2 Custom Directory with HTTP Range Requests

```rust
// typstify-search-wasm/src/directory.rs

use gloo_net::http::Request;
use std::ops::Range;

pub struct HttpDirectory {
    base_url: String,
    manifest: IndexManifest,
    chunk_cache: scc::HashMap<String, Vec<u8>>,
}

impl HttpDirectory {
    /// Fetch bytes from remote index using Range requests
    pub async fn read_range(&self, file: &str, range: Range<usize>) -> Result<Vec<u8>> {
        // Calculate which chunks we need
        let start_chunk = range.start / self.manifest.chunk_size;
        let end_chunk = range.end / self.manifest.chunk_size;

        let mut data = Vec::new();
        for chunk_idx in start_chunk..=end_chunk {
            let chunk_data = self.fetch_chunk(file, chunk_idx).await?;
            data.extend_from_slice(&chunk_data);
        }

        // Extract exact range from concatenated chunks
        let offset = range.start % self.manifest.chunk_size;
        Ok(data[offset..offset + range.len()].to_vec())
    }

    async fn fetch_chunk(&self, file: &str, chunk_idx: usize) -> Result<Vec<u8>> {
        let chunk_key = format!("{}.{}", file, chunk_idx);

        // Check cache first (using scc for thread-safe access)
        if let Some(entry) = self.chunk_cache.get(&chunk_key) {
            return Ok(entry.get().clone());
        }

        // Fetch from network
        let url = format!("{}/search/{}", self.base_url, chunk_key);
        let response = Request::get(&url)
            .send()
            .await?;

        let bytes = response.binary().await?;

        // Cache for future use
        let _ = self.chunk_cache.insert(chunk_key, bytes.clone());

        Ok(bytes)
    }
}
```

### 5.3 Simplified Search Index Format

For better WASM compatibility, use a simplified JSON-based index alongside Tantivy:

```rust
#[derive(Serialize, Deserialize)]
pub struct SimpleSearchIndex {
    pub entries: Vec<SearchEntry>,
    pub term_index: scc::HashMap<String, Vec<usize>>,  // term -> entry indices
}

#[derive(Serialize, Deserialize)]
pub struct SearchEntry {
    pub title: String,
    pub url: String,
    pub summary: String,
    pub lang: String,
    pub tags: Vec<String>,
    pub terms: Vec<String>,  // Pre-tokenized terms for matching
}
```

**Fallback Strategy**:

- Small sites (< 1MB index): Load entire SimpleSearchIndex
- Large sites: Use chunked Tantivy index with Range requests

## 6. Frontend (Leptos CSR)

### 6.1 Search Component

```rust
// typstify-ui/src/search.rs

use leptos::prelude::*;

#[component]
pub fn SearchBox() -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (results, set_results) = signal(Vec::<SearchResult>::new());
    let (is_loading, set_is_loading) = signal(false);

    let search_engine = expect_context::<SearchEngine>();

    // Debounced search effect
    let _ = Effect::new(move || {
        let q = query();
        if q.len() < 2 {
            set_results(vec![]);
            return;
        }

        set_is_loading(true);
        spawn_local(async move {
            if let Ok(res) = search_engine.search(&q, 10).await {
                set_results(res);
            }
            set_is_loading(false);
        });
    });

    view! {
        <div class="search-container">
            <input
                type="text"
                placeholder="Search..."
                class="search-input"
                on:input=move |ev| set_query(event_target_value(&ev))
                prop:value=query
            />
            <Show when=move || is_loading()>
                <span class="loading-indicator">"Searching..."</span>
            </Show>
            <SearchResults results=results />
        </div>
    }
}

#[component]
fn SearchResults(results: ReadSignal<Vec<SearchResult>>) -> impl IntoView {
    view! {
        <ul class="search-results">
            <For
                each=move || results()
                key=|r| r.url.clone()
                children=move |result| {
                    view! {
                        <li class="search-result-item">
                            <a href=result.url.clone()>
                                <h3>{result.title.clone()}</h3>
                                <p>{result.summary.clone()}</p>
                            </a>
                        </li>
                    }
                }
            />
        </ul>
    }
}
```

### 6.2 Custom Script Injection

```rust
// typstify-ui/src/article.rs

use leptos::prelude::*;
use leptos_meta::Script;

#[component]
pub fn Article(
    #[prop(into)] content: String,
    #[prop(default = vec![])] custom_js: Vec<String>,
    #[prop(default = vec![])] custom_css: Vec<String>,
) -> impl IntoView {
    view! {
        // Inject custom CSS
        <For
            each=move || custom_css.clone()
            key=|s| s.clone()
            children=move |href| {
                view! { <link rel="stylesheet" href=href /> }
            }
        />

        // Article content
        <article class="prose" inner_html=content.clone() />

        // Inject custom JS (deferred)
        <For
            each=move || custom_js.clone()
            key=|s| s.clone()
            children=move |src| {
                view! { <Script src=src defer=true /> }
            }
        />
    }
}
```

### 6.3 Tailwind CSS v4 Setup

**style/main.css**:

```css
@import "tailwindcss";

@theme {
  --color-primary: #3b82f6;
  --color-secondary: #64748b;
  --font-sans: "Inter", system-ui, sans-serif;
  --font-mono: "JetBrains Mono", monospace;
}

/* Search component styles */
.search-container {
  @apply relative w-full max-w-md;
}

.search-input {
  @apply w-full px-4 py-2 border border-gray-300 rounded-lg;
  @apply focus:outline-none focus:ring-2 focus:ring-primary;
}

.search-results {
  @apply absolute top-full left-0 right-0 mt-2;
  @apply bg-white border border-gray-200 rounded-lg shadow-lg;
  @apply max-h-96 overflow-y-auto;
}

.search-result-item {
  @apply p-4 border-b border-gray-100 last:border-b-0;
  @apply hover:bg-gray-50 transition-colors;
}
```

## 7. Watch Command (Embedded Dev Server)

The `typstify watch` command provides an embedded development server with live reload. This is part of the single binary - no separate server binary needed.

### 7.1 Watch Command Implementation

```rust
// bin/typstify/src/cmd/watch.rs

use notify::{Watcher, RecursiveMode, RecommendedWatcher};
use tokio::sync::broadcast;

pub struct WatchServer {
    config: Config,
    reload_tx: broadcast::Sender<()>,
}

impl WatchServer {
    pub async fn run(port: u16, open_browser: bool) -> Result<()> {
        let (reload_tx, _) = broadcast::channel(16);
        let server = Self {
            config: Config::load("config.toml")?,
            reload_tx,
        };

        // Initial build (with drafts enabled)
        server.build_with_drafts()?;

        // Setup file watcher
        let tx = server.reload_tx.clone();
        let mut watcher = RecommendedWatcher::new(
            move |res| {
                if let Ok(_) = res {
                    let _ = tx.send(());
                }
            },
            notify::Config::default().with_poll_interval(Duration::from_millis(200)),
        )?;

        watcher.watch(Path::new("content"), RecursiveMode::Recursive)?;
        watcher.watch(Path::new("templates"), RecursiveMode::Recursive)?;
        watcher.watch(Path::new("style"), RecursiveMode::Recursive)?;

        // Start embedded HTTP server with WebSocket for reload
        let app = Router::new()
            .nest_service("/", ServeDir::new(&server.config.build.output_dir))
            .route("/__livereload", get(livereload_handler))
            .with_state(server.reload_tx.clone());

        // Spawn rebuild task
        let config = server.config.clone();
        let mut rx = server.reload_tx.subscribe();
        tokio::spawn(async move {
            while rx.recv().await.is_ok() {
                tracing::info!("File changed, rebuilding...");
                if let Err(e) = rebuild(&config) {
                    tracing::error!("Build error: {e}");
                }
            }
        });

        let addr = format!("127.0.0.1:{port}");
        tracing::info!("Dev server running at http://{addr}");

        if open_browser {
            let _ = open::that(format!("http://{addr}"));
        }

        let listener = TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
```

## 8. Deployment

### 8.1 Build Output Structure

```text
public/
├── index.html
├── posts/
│   ├── hello-world/
│   │   └── index.html
│   └── ...
├── tags/
│   └── rust/
│       └── index.html
├── search/
│   ├── manifest.json
│   ├── terms.idx.0
│   ├── terms.idx.1
│   ├── store.idx.0
│   └── ...
├── pkg/
│   ├── typstify_search_wasm.js
│   └── typstify_search_wasm_bg.wasm
├── assets/
│   └── ...
├── feed.xml
└── sitemap.xml
```

### 8.2 Cloudflare Pages Configuration

**wrangler.toml** (optional, for custom headers):

```toml
[site]
bucket = "./public"

[[headers]]
for = "/search/*"
[headers.values]
Access-Control-Allow-Origin = "*"
Accept-Ranges = "bytes"
Cache-Control = "public, max-age=31536000, immutable"
```

### 8.3 GitHub Actions Workflow

```yaml
name: Build and Deploy

on:
  push:
    branches: [main]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: wasm32-unknown-unknown

      - name: Setup Bun
        uses: oven-sh/setup-bun@v1

      - name: Install wasm-pack
        run: cargo install wasm-pack

      - name: Build
        run: |
          bun install
          just build

      - name: Deploy to Cloudflare Pages
        uses: cloudflare/pages-action@v1
        with:
          apiToken: ${{ secrets.CF_API_TOKEN }}
          accountId: ${{ secrets.CF_ACCOUNT_ID }}
          projectName: typstify
          directory: public
```

## 9. Error Handling Strategy

### 9.1 Library Crates (thiserror)

```rust
// typstify-core/src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Content parsing error at {path}: {message}")]
    Parse { path: String, message: String },

    #[error("Template error: {0}")]
    Template(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### 9.2 Application (eyre)

```rust
// bin/typstify/src/main.rs

use clap::Parser;
use eyre::{Result, WrapErr};

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    init_tracing(cli.verbose);

    match cli.command {
        Commands::Build { output, drafts } => {
            cmd::build::run(&cli.config, &output, drafts)
                .wrap_err("Failed to build site")?;
        }
        Commands::Watch { port, open } => {
            cmd::watch::run(&cli.config, port, open)
                .wrap_err("Failed to start dev server")?;
        }
        Commands::New { path, template } => {
            cmd::new::run(&path, &template)
                .wrap_err("Failed to create content")?;
        }
        Commands::Check { strict } => {
            cmd::check::run(&cli.config, strict)
                .wrap_err("Validation failed")?;
        }
    }

    Ok(())
}
```

## 10. Observability

### 10.1 Tracing Setup

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_tracing() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}
```

### 10.2 Build Metrics

```rust
#[tracing::instrument(skip(config))]
pub fn build(config: &Config) -> Result<BuildStats> {
    let start = Instant::now();

    tracing::info!("Starting build");

    let pages = process_content(config)?;
    tracing::info!(count = pages.len(), "Processed content pages");

    let index_size = build_search_index(config, &pages)?;
    tracing::info!(size = index_size, "Generated search index");

    let duration = start.elapsed();
    tracing::info!(?duration, "Build completed");

    Ok(BuildStats { pages: pages.len(), duration })
}
```

## 11. Dependencies Summary

### 11.1 Core Dependencies

| Crate | Purpose | Feature |
|-------|---------|---------|
| `clap` | CLI parsing | derive |
| `config` | Configuration | toml |
| `eyre` | App error handling | - |
| `thiserror` | Lib error handling | - |
| `tracing` | Logging | - |
| `serde` | Serialization | derive |
| `tokio` | Async runtime | full |

### 11.2 Content Processing

| Crate | Purpose |
|-------|---------|
| `pulldown-cmark` | Markdown parsing |
| `typst` | Typst compilation |
| `syntect` | Syntax highlighting |
| `toml` | Frontmatter parsing |

### 11.3 Search & WASM

| Crate | Purpose |
|-------|---------|
| `tantivy` | Full-text search |
| `wasm-bindgen` | WASM bindings |
| `gloo-net` | Browser HTTP (WASM) |
| `scc` | Concurrent collections |

### 11.4 Web & Frontend

| Crate | Purpose |
|-------|---------|
| `leptos` | UI framework |
| `axum` | Dev server |
| `tower-http` | Static file serving |
| `rss` | RSS generation |

## 12. Performance Considerations

1. **Parallel Content Processing**: Use `rayon` for parallel markdown/typst compilation
2. **Incremental Builds**: Track file hashes to skip unchanged content
3. **Index Chunking**: 64KB chunks optimize for CDN caching and Range requests
4. **WASM Size**: Use `wasm-opt` to minimize WASM bundle (~200KB target)
5. **Lazy Loading**: Search WASM loads on-demand when user focuses search input

## 13. Future Enhancements

- [ ] Image optimization pipeline (WebP/AVIF conversion)
- [ ] Shortcodes system (similar to Hugo)
- [ ] Theme system with hot-swappable templates
- [ ] Plugin architecture for custom processors
- [ ] Incremental static regeneration for large sites
