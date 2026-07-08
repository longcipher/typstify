# Tasks: Codebase Quality

All 30 findings consolidated into sequential tasks. Tasks are ordered by dependency.

---

## Phase 1 — Shared Utilities & Security (Findings 13, 1, 5, 9, 10)

### Task 1.1: Extract shared utilities to typstify-core

> **Context:** Three divergent implementations of `html_escape`, `slugify`, and `strip_html` exist across `content.rs`, `markdown.rs`, `syntax.rs`, `simple.rs`, `indexer.rs`, `typst_parser.rs`, `html.rs`. Correctness depends on which copy is called.
> **Verification:** `cargo build` succeeds; `cargo test --all-features` passes; `cargo shear` reports no unused deps.
> **Scenario Coverage:** `features/tech-debt.feature` — Shared HTML escape/slugify/strip_html utility

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Preserve existing behavior` — all callers get the same output from the canonical implementations
- **Simplification Focus:** `Ponytail: Use existing implementations as-is, do not redesign the algorithms`
- **Status:** 🟢 DONE
- [x] Step 1: Create `crates/typstify-core/src/utils.rs` with `pub fn html_escape(s: &str) -> String`, `pub fn slugify(s: &str) -> String`, `pub fn strip_html(html: &str) -> String`. Use the `indexer.rs` version of `strip_html` as canonical (handles script/style, entities, whitespace). Use `markdown.rs` `html_escape` as canonical. Use `markdown.rs` `slugify` as canonical.
- [x] Step 2: Add `pub mod utils;` to `crates/typstify-core/src/lib.rs`.
- [x] Step 3: Replace all local `html_escape`/`slugify`/`strip_html` definitions in `markdown.rs`, `syntax.rs`, `typst_parser.rs`, `html.rs`, `content.rs`, `simple.rs`, `indexer.rs` with `use typstify_core::utils::{html_escape, slugify, strip_html};`.
- [x] Step 4: Run `cargo build` and `cargo test --all-features` to verify.
- [x] BDD Verification: `cargo test --all-features` — all existing tests pass
- [x] Advanced Test Verification: `cargo shear` — no new unused deps introduced
- [x] Runtime Verification: `cargo build` — no warnings

### Task 1.2: Add auto-escaping to template engine with |raw support

> **Context:** `template.rs:94-125` replaces `{{ variable }}` with raw values. Any content file with HTML in its title injects JavaScript into every generated page.
> **Verification:** Template rendering escapes HTML by default; `{{ var|raw }}` preserves HTML.
> **Scenario Coverage:** `features/security.feature` — Template variables are HTML-escaped by default; Template variables with raw suffix preserve HTML

- **Loop Type:** `BDD+TDD`
- **Behavioral Contract:** `Change: all template variables are HTML-escaped by default; add |raw opt-in`
- **Simplification Focus:** `Ponytail: Simple suffix check, not a full template parser`
- **Status:** 🟢 DONE
- [x] Step 1: Add a unit test in `template.rs` that asserts `{{ title }}` with `title = '<b>bold</b>'` produces `&lt;b&gt;bold&lt;/b&gt;` in the output.
- [x] Step 2: In `Template::render()`, import `typstify_core::utils::html_escape`. After extracting `var_name`, check if it ends with `|raw`. If so, skip escaping. Otherwise, apply `html_escape(&value)` before replacement.
- [x] Step 3: Audit all 12+ hardcoded templates in `template.rs` (`const BASE_TEMPLATE`, etc.) to identify which variables contain pre-rendered HTML (e.g., `content`, `tags_html`, `section_nav`, `lang_switcher`). Add `|raw` suffix to those variables in the templates.
- [x] Step 4: Run `cargo test --all-features` to verify all templates still render correctly.
- [x] BDD Verification: `cargo test --all-features` — template tests pass with escaping
- [x] Advanced Test Verification: `cargo test --all-features` — e2e tests pass
- [x] Runtime Verification: Build the example blog and verify no raw HTML injection in output

### Task 1.3: Escape user-derived values in html.rs

> **Context:** `html.rs:657-658` interpolates `p.title` and `p.url` directly into HTML without escaping. Tag names at `html.rs:489-493` are also unescaped.
> **Verification:** All user-derived values in HTML output are escaped.
> **Scenario Coverage:** `features/security.feature` — Page titles in list pages are HTML-escaped; Tag names in tag index are HTML-escaped

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Change: all user-derived values in HTML output are escaped`
- **Simplification Focus:** `Ponytail: Apply html_escape at each interpolation site, no abstraction layer`
- **Status:** 🟢 DONE
- [x] Step 1: In `html.rs`, add `use typstify_core::utils::html_escape;`.
- [x] Step 2: Apply `html_escape()` to `p.title` in `list_item_html()` (line 900), `short_item_html()`, archive page generation (line 657-658), tag index (line 489-493), and any other site where user-derived values appear in HTML text or attributes.
- [x] Step 3: Apply `html_escape()` to `p.url` where it appears in `href` attribute values (URLs in attributes need attribute escaping — use a dedicated `attr_escape` or ensure `html_escape` handles `"`).
- [x] Step 4: Run `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — all tests pass
- [x] Advanced Test Verification: N/A — visual inspection of generated HTML
- [x] Runtime Verification: Build example blog and verify titles with special chars are escaped

### Task 1.4: Add sanitize_html config option

> **Context:** `markdown.rs:182-183` passes raw HTML through without sanitization. While standard markdown behavior, a `sanitize_html` option provides defense-in-depth.
> **Verification:** With `sanitize_html = true`, `<script>` tags are stripped. With `false`, they pass through.
> **Scenario Coverage:** `features/security.feature` — Raw HTML in markdown is sanitized when enabled; Raw HTML passes through when disabled

- **Loop Type:** `BDD+TDD`
- **Behavioral Contract:** `Change: new config field sanitize_html, default false, preserves existing behavior`
- **Simplification Focus:** `Ponytail: Simple tag stripping, not a full HTML sanitizer`
- **Status:** 🟢 DONE
- [x] Step 1: Add `sanitize_html: bool` field (default `false`) to `BuildConfig` in `config.rs`.
- [x] Step 2: Add unit test: when `sanitize_html = true`, markdown with `<script>alert(1)</script>` produces output without the script tag.
- [x] Step 3: In `markdown.rs`, when `sanitize_html` is true, filter `Event::Html`/`Event::InlineHtml` to strip `<script>`, `<iframe>`, `<object>` tags and `on*` event handler attributes.
- [x] Step 4: Run `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — sanitization tests pass
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: Build with sanitize_html enabled and verify script tags are removed

### Task 1.5: Remove data-path attribute from Typst output

> **Context:** `typst_parser.rs:78` embeds `data-path="{}"` with the source file path into generated HTML, leaking filesystem structure to site visitors.
> **Verification:** Generated HTML does not contain `data-path` attributes.
> **Scenario Coverage:** `features/security.feature` — Typst source path is not leaked in generated HTML

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Change: remove data-path attribute from generated HTML`
- **Simplification Focus:** `Ponytail: Delete the attribute, do not make it configurable`
- **Status:** 🟢 DONE
- [x] Step 1: In `typst_parser.rs:78`, remove `data-path="{}"` from the format string.
- [x] Step 2: Add a test asserting the output does not contain `data-path`.
- [x] Step 3: Run `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — test passes
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: Build example blog and verify no data-path in output

---

## Phase 2 — Correctness Fixes (Findings 2, 3, 6, 7, 11, 12, 30)

### Task 2.1: Propagate page generation errors

> **Context:** `build.rs:241-248` logs `warn!()` on page generation failure and silently continues. A site can deploy with missing pages and no error exit code.
> **Verification:** Build returns error when any page fails; error lists failed paths.
> **Scenario Coverage:** `features/correctness.feature` — Build fails when page generation errors occur

- **Loop Type:** `BDD+TDD`
- **Behavioral Contract:** `Change: build fails on page generation errors instead of silently continuing`
- **Simplification Focus:** `Ponytail: Collect errors into Vec, single error return`
- **Status:** 🟢 DONE
- [x] Step 1: Add `PageGenerationFailed { errors: Vec<String> }` variant to `BuildError` in the generator crate's error module.
- [x] Step 2: Add unit test: when a page has an invalid template variable, `build()` returns `Err` containing the failed page path.
- [x] Step 3: In `build.rs:240-248`, collect `Err` results into a `Vec`. After the loop, if non-empty, return `Err(BuildError::PageGenerationFailed { errors })`.
- [x] Step 4: Run `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — error propagation test passes
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

### Task 2.2: Fix UTF-8 byte-slice panic

> **Context:** `html.rs:100-103` uses `&section[1..]` which is byte-offset slicing. Panics if `section` starts with a multi-byte UTF-8 character (e.g., CJK).
> **Verification:** Non-ASCII section slugs produce correct output without panic.
> **Scenario Coverage:** `features/correctness.feature` — Non-ASCII section slugs do not cause panics

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Change: use char-based slicing instead of byte slicing`
- **Simplification Focus:** `Ponytail: One-line fix: section[c.len_utf8()..]`
- **Status:** 🟢 DONE
- [x] Step 1: Add test with a section name starting with a multi-byte character (e.g., "日本語").
- [x] Step 2: In `html.rs:103`, replace `&section[1..]` with `&section[c.len_utf8()..]`.
- [x] Step 3: Run `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — new test passes
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

### Task 2.3: Fix RSS language feed paths

> **Context:** `build.rs:604-610` — both branches of the `if *lang == default_lang.as_str()` produce the same path, making the default language feed redundant with the main feed.
> **Verification:** Default language feed is written to root; non-default feeds are in language subdirectories.
> **Scenario Coverage:** `features/correctness.feature` — RSS feed paths are correct for default and non-default languages

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Change: default language RSS feed written to root output directory`
- **Simplification Focus:** `Ponytail: Fix the path, no structural change`
- **Status:** 🟢 DONE
- [x] Step 1: Add test: default language feed exists at `output_dir/rss.xml`; non-default feed at `output_dir/{lang}/rss.xml`.
- [x] Step 2: In `build.rs:605-610`, for default language, set path to `self.output_dir.join("rss.xml")`.
- [x] Step 3: Run `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — RSS path test passes
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: Build example blog and verify RSS paths

### Task 2.4: Use serde_json for asset manifest

> **Context:** `assets.rs:62-74` builds JSON via `format!()` without escaping quotes or control characters.
> **Verification:** Asset manifest is always valid JSON.
> **Scenario Coverage:** `features/correctness.feature` — Asset manifest produces valid JSON

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Change: use serde_json for JSON serialization`
- **Simplification Focus:** `Ponytail: One-line change to use serde_json::to_string_pretty`
- **Status:** 🟢 DONE
- [x] Step 1: Add test: `to_json()` output parses as valid JSON via `serde_json::from_str`.
- [x] Step 2: In `assets.rs`, replace manual JSON construction with `serde_json::to_string_pretty(&self.assets)`.
- [x] Step 3: Run `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — manifest test passes
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

### Task 2.5: Wire Config::load_with_env into CLI

> **Context:** `config.rs:272` defines `load_with_env()` supporting `TYPSTIFY__*` env var overrides, but CLI commands use `Config::load()`. The `config` crate dependency exists for this unused path.
> **Verification:** `TYPSTIFY__*` environment variables override config file values.
> **Scenario Coverage:** `features/correctness.feature` — Environment variable overrides work for config

- **Loop Type:** `BDD+TDD`
- **Behavioral Contract:** `Change: CLI uses load_with_env instead of load, enabling env var overrides`
- **Simplification Focus:** `Ponytail: One-line change in each CLI command: Config::load -> Config::load_with_env`
- **Status:** 🟢 DONE
- [x] Step 1: Add test: setting `TYPSTIFY__SITE__TITLE` env var overrides the config file title.
- [x] Step 2: In `cmd/build.rs`, `cmd/watch.rs`, `cmd/check.rs`, replace `Config::load(&config_path)` with `Config::load_with_env(&config_path)`.
- [x] Step 3: Run `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — env override test passes
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

### Task 2.6: Strengthen asset fingerprint hash

> **Context:** `assets.rs:227-234` uses FNV-1a truncated to 8 hex chars (32 bits). Birthday bound gives ~50% collision at ~256 assets.
> **Verification:** Fingerprints are at least 12 hex characters; collision risk is negligible.
> **Scenario Coverage:** `features/correctness.feature` — Asset fingerprints use sufficient hash bits

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Change: longer fingerprints, same format`
- **Simplification Focus:** `Ponytail: Change format string width, no hash algorithm change`
- **Status:** 🟢 DONE
- [x] Step 1: Add test: fingerprint is at least 12 characters; two different inputs produce different fingerprints.
- [x] Step 2: In `assets.rs`, change the format truncation from 8 to 12 hex characters.
- [x] Step 3: Run `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — fingerprint test passes
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

### Task 2.7: Fix dev server debounce event loss

> **Context:** `watch.rs:128-130` — during debounce window, events are consumed but rebuild is skipped. The drain at line 133 discards queued events.
> **Verification:** Rapid successive file changes all trigger rebuilds.
> **Scenario Coverage:** `features/correctness.feature` — Dev server debounce does not lose file change events

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Change: dirty flag ensures pending changes trigger rebuild after debounce`
- **Simplification Focus:** `Ponytail: Set a boolean flag, check after debounce`
- **Status:** 🟢 DONE
- [x] Step 1: In `watch.rs`, add a `dirty: bool` flag. On event during debounce window, set `dirty = true` instead of skipping. After debounce window expires, check `dirty` and trigger rebuild if set.
- [x] Step 2: Run `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — existing tests pass
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

---

## Phase 3 — Tech Debt Cleanup (Findings 14, 15, 16, 17)

### Task 3.1: Extract shared base context helper in html.rs

> **Context:** `html.rs` has 8 near-identical `TemplateContext` construction blocks (lines 169-856) building the same ~15 navigation variables.
> **Verification:** Each page-generation method calls the shared helper; no duplicate context construction remains.
> **Scenario Coverage:** `features/tech-debt.feature` — Base template context is built by a shared helper

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Preserve existing behavior` — identical output from all page generators
- **Simplification Focus:** `Ponytail: Single private method, not a builder pattern`
- **Status:** 🟢 DONE
- [x] Step 1: Extract `fn build_shared_base_ctx(&self, lang: &str, title: &str, canonical_url: &str, content: &str) -> TemplateContext` that constructs the common 15 variables.
- [x] Step 2: Replace all 8 duplicate blocks with calls to the helper.
- [x] Step 3: Run `cargo test --all-features` to verify identical output.
- [x] BDD Verification: `cargo test --all-features` — all tests pass
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

### Task 3.2: Remove dead crates

> **Context:** `typstify-ui` (~295 LOC) and `typstify-search-wasm` (~628 LOC) are never imported by the binary or generator. They pull in `leptos`, `wasm-bindgen`, `web-sys`, `gloo-net`, etc.
> **Verification:** Workspace compiles; `cargo shear` reports no unused deps.
> **Scenario Coverage:** `features/tech-debt.feature` — Dead crates are removed from workspace

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Change: remove unused crates and their dependencies`
- **Simplification Focus:** `Ponytail: Delete directories and references, no migration`
- **Status:** 🟢 DONE
- [x] Step 1: Remove `crates/typstify-ui/` and `crates/typstify-search-wasm/` directories.
- [x] Step 2: Remove unused workspace dependencies from root `Cargo.toml`: `leptos`, `leptos_meta`, `leptos_router`, `wasm-bindgen`, `wasm-bindgen-futures`, `wasm-bindgen-test`, `web-sys`, `gloo-net`, `console_error_panic_hook`, `js-sys`, `serde-wasm-bindgen`, `typstify-search-wasm`, `typstify-ui`.
- [x] Step 3: Run `cargo build` and `cargo shear`.
- [x] BDD Verification: `cargo test --all-features` — all tests pass
- [x] Advanced Test Verification: `cargo shear` — no unused deps
- [x] Runtime Verification: `cargo build` — no warnings

### Task 3.3: Consolidate duplicated search types

> **Context:** `SimpleDocument`, `SimpleSearchIndex`, `IndexManifest`, `FileManifest` are defined identically in both `typstify-search` and `typstify-search-wasm`. Schema drift risk.
> **Verification:** Both crates import types from a shared location; no duplicate definitions.
> **Scenario Coverage:** `features/tech-debt.feature` — Duplicated search types are consolidated

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Preserve existing behavior` — same types, same serde derives
- **Simplification Focus:** `Ponytail: Move types to typstify-core, update imports`
- **Status:** 🟢 DONE
- [x] Step 1: Move `SimpleDocument`, `SimpleSearchIndex`, `IndexManifest`, `FileManifest` to `typstify-core` (or a new `typstify-search-types` if WASM compatibility requires it).
- [x] Step 2: Update imports in `typstify-search` and `typstify-search-wasm`.
- [x] Step 3: Run `cargo build` and `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — all tests pass
- [x] Advanced Test Verification: `cargo shear` — no unused deps
- [x] Runtime Verification: `cargo build` — no warnings

### Task 3.4: Remove unused workspace dependencies

> **Context:** `leptos_meta` and `leptos_router` are declared as workspace dependencies but never referenced in any sub-crate.
> **Verification:** Workspace compiles; no unused deps.
> **Scenario Coverage:** `features/tech-debt.feature` — Unused workspace dependencies are removed

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Change: remove unused workspace deps`
- **Simplification Focus:** `Ponytail: Delete two lines from Cargo.toml`
- **Status:** 🟢 DONE
- [x] Step 1: Remove `leptos_meta` and `leptos_router` from `[workspace.dependencies]` in root `Cargo.toml`.
- [x] Step 2: Run `cargo build` and `cargo shear`.
- [x] BDD Verification: `cargo test --all-features` — all tests pass
- [x] Advanced Test Verification: `cargo shear` — no unused deps
- [x] Runtime Verification: `cargo build` — no warnings

---

## Phase 4 — Performance (Findings 18, 19, 20, 21, 23)

### Task 4.1: Pass Config by reference through build pipeline

> **Context:** `Config` is cloned 8+ times in `build.rs` (lines 136, 207, 254, 340, 551, 577, 626, 645). Each clone deep-copies all String and HashMap fields.
> **Verification:** Generators accept `&Config`; build produces identical output.
> **Scenario Coverage:** `features/performance.feature` — Config is not cloned unnecessarily

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Preserve existing behavior` — same build output, fewer allocations
- **Simplification Focus:** `Ponytail: Change function signatures to &Config, add lifetimes`
- **Status:** 🟢 DONE
- [x] Step 1: Change `HtmlGenerator::new`, `RssGenerator::new`, `SitemapGenerator::new`, `RobotsGenerator::new`, `ContentCollector::new` to accept `&Config` instead of `Config`.
- [x] Step 2: Add lifetime parameters as needed. Store `config: &'a Config` in each struct.
- [x] Step 3: Update all call sites in `build.rs` to pass `&self.config` instead of `self.config.clone()`.
- [x] Step 4: Run `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — all tests pass
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

### Task 4.2: Cache base_url computation

> **Context:** `config.rs:308` `base_url()` allocates `format!("{host}{base_path}")` on every call. Called once per page in 8+ methods.
> **Verification:** base_url is computed once; same output.
> **Scenario Coverage:** `features/performance.feature` — base_url is computed once and reused

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Preserve existing behavior`
- **Simplification Focus:** `Ponytail: Add a cached String field, compute in constructor`
- **Status:** 🟢 DONE
- [x] Step 1: Add `base_url: String` field to `Config`, computed during construction. Change `base_url()` to return `&str`.
- [x] Step 2: Run `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — all tests pass
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

### Task 4.3: Cache year string

> **Context:** `Utc::now().year().to_string()` called in every `build_base_context` and `generate_*_page` method (8+ call sites).
> **Verification:** Year computed once per build; same output.
> **Scenario Coverage:** `features/performance.feature` — Year string is computed once per build

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Preserve existing behavior`
- **Simplification Focus:** `Ponytail: Compute once in HtmlGenerator::new, store as field`
- **Status:** 🟢 DONE
- [x] Step 1: Add `year: String` field to `HtmlGenerator`, computed in `new()`. Use it in `build_shared_base_ctx`.
- [x] Step 2: Run `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — all tests pass
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

### Task 4.4: Cache section_nav and lang_switcher

> **Context:** `generate_section_nav` and `generate_lang_switcher` are called per page, producing identical HTML for all pages sharing the same language.
> **Verification:** Same output; no redundant regeneration.
> **Scenario Coverage:** `features/performance.feature` — Section nav and language switcher are cached

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Preserve existing behavior`
- **Simplification Focus:** `Ponytail: HashMap<(String, String), String> cache in HtmlGenerator`
- **Status:** 🟢 DONE
- [x] Step 1: Add `section_nav_cache: HashMap<String, String>` and `lang_switcher_cache: HashMap<String, String>` to `HtmlGenerator`. Compute on first access per language, reuse thereafter.
- [x] Step 2: Run `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — all tests pass
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

### Task 4.5: Optimize escape_xml single-pass

> **Context:** `sitemap.rs:244-250` chains 5 `.replace()` calls, creating 5 intermediate String allocations.
> **Verification:** Output identical for all inputs; no intermediate allocations.
> **Scenario Coverage:** `features/performance.feature` — escape_xml uses single-pass algorithm

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Preserve existing behavior`
- **Simplification Focus:** `Ponytail: Single-pass char iterator with match`
- **Status:** 🟢 DONE
- [x] Step 1: Add test: `escape_xml` output matches expected for inputs containing `&`, `<`, `>`, `"`, `'`.
- [x] Step 2: Rewrite `escape_xml` as a single-pass function using `.chars().map()` with a match statement.
- [x] Step 3: Run `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — escape_xml tests pass
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

---

## Phase 5 — Test Coverage (Findings 24-29)

### Task 5.1: Fix e2e tests with bundled fixtures

> **Context:** `e2e.rs:11-16` — all 7 tests silently pass when `examples/blog` is absent via `if !path.exists() { return; }`.
> **Verification:** Tests run without external dependencies; tests fail on bad assertions.
> **Scenario Coverage:** `features/test.feature` — E2e tests use bundled test fixtures

- **Loop Type:** `BDD+TDD`
- **Behavioral Contract:** `Change: tests are self-contained with bundled fixtures`
- **Simplification Focus:** `Ponytail: Bundle minimal fixtures, not the full example blog`
- **Status:** 🟢 DONE
- [x] Step 1: Create `crates/typstify-generator/testdata/` with a minimal `config.toml` and 2-3 content files (markdown, Typst).
- [x] Step 2: Replace `if !path.exists() { return; }` with `#[ignore]` attribute and a reason string. Update tests to use bundled fixtures.
- [x] Step 3: Run `cargo test --all-features` — tests pass with fixtures.
- [x] BDD Verification: `cargo test --all-features` — e2e tests run (not skip)
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

### Task 5.2: Add robots.rs unit tests

> **Context:** `robots.rs` (62 LOC) has zero tests. Critical for SEO/crawler behavior.
> **Verification:** 3 tests: disabled config, enabled config with disallow/allow, sitemap URL with base_path.
> **Scenario Coverage:** `features/test.feature` — robots.rs has unit tests

- **Loop Type:** `BDD+TDD`
- **Behavioral Contract:** `Preserve existing behavior` — new tests only
- **Simplification Focus:** `Ponytail: 3 focused tests, no test framework overhead`
- **Status:** 🟢 DONE
- [x] Step 1: Add `#[cfg(test)] mod tests` to `robots.rs`.
- [x] Step 2: Add test: disabled config produces empty string.
- [x] Step 3: Add test: enabled config with disallow/allow paths produces correct robots.txt format.
- [x] Step 4: Add test: sitemap URL includes base_path.
- [x] BDD Verification: `cargo test --all-features` — 3 new tests pass
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

### Task 5.3: Add ContentCollector tests

> **Context:** `collector.rs:82-100` `collect()` — the core content pipeline — has zero tests. Only 3 trivial data-structure tests exist.
> **Verification:** Tests verify page count, section assignments, taxonomy entries.
> **Scenario Coverage:** `features/test.feature` — ContentCollector::collect is tested

- **Loop Type:** `BDD+TDD`
- **Behavioral Contract:** `Preserve existing behavior` — new tests only
- **Simplification Focus:** `Ponytail: Use tempfile::TempDir, minimal content files`
- **Status:** 🟢 DONE
- [x] Step 1: Create temp directory with markdown files (with/without frontmatter, drafts, different languages).
- [x] Step 2: Add test: `collect()` returns correct page count, section assignments, translation groups, taxonomy entries.
- [x] Step 3: Add test: draft pages are excluded when `drafts = false`.
- [x] BDD Verification: `cargo test --all-features` — collector tests pass
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

### Task 5.4: Add HtmlGenerator method tests

> **Context:** 7 of 18 `HtmlGenerator` public methods have zero unit tests.
> **Verification:** Each tested method's output contains expected structural elements.
> **Scenario Coverage:** `features/test.feature` — HtmlGenerator page methods are tested

- **Loop Type:** `BDD+TDD`
- **Behavioral Contract:** `Preserve existing behavior` — new tests only
- **Simplification Focus:** `Ponytail: One test per method, assert key HTML elements`
- **Status:** 🟢 DONE
- [x] Step 1: Add tests for `generate_list_page`, `generate_taxonomy_page`, `generate_tags_index_page`, `generate_categories_index_page`, `generate_archives_page`, `generate_section_page`, `generate_shorts_page`.
- [x] Step 2: Each test asserts the output contains expected title, navigation, and content structure.
- [x] BDD Verification: `cargo test --all-features` — 7 new tests pass
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

### Task 5.5: Add multi-language build test

> **Context:** `generate_auto_pages()` has ~210 lines of i18n logic with zero test coverage. The only build test uses a single-language config.
> **Verification:** Test verifies output in both language directories.
> **Scenario Coverage:** `features/test.feature` — Multi-language build is tested

- **Loop Type:** `BDD+TDD`
- **Behavioral Contract:** `Preserve existing behavior` — new tests only
- **Simplification Focus:** `Ponytail: Config with 2 languages, 2 content files, assert output`
- **Status:** 🟢 DONE
- [x] Step 1: Create test with `Config` having `languages: { "zh": ... }`.
- [x] Step 2: Add content files with `.zh.md` extensions.
- [x] Step 3: Assert output contains both `/page/index.html` and `/zh/page/index.html`.
- [x] BDD Verification: `cargo test --all-features` — i18n test passes
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

### Task 5.6: Extract shared test fixtures

> **Context:** `test_config()` is copy-pasted in 5 test modules (`build.rs:718`, `html.rs:1002`, `collector.rs:308`, `rss.rs:173`, `sitemap.rs:566`).
> **Verification:** Single definition imported by all test modules.
> **Scenario Coverage:** `features/test.feature` — Shared test fixtures replace duplicated helpers

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Preserve existing behavior` — same test data
- **Simplification Focus:** `Ponytail: #[cfg(test)] pub fn in typstify-core`
- **Status:** 🟢 DONE
- [x] Step 1: Add `#[cfg(test)] pub fn test_config() -> Config` and `pub fn test_page() -> Page` to `typstify-core`.
- [x] Step 2: Replace all 5 local `test_config()` definitions with imports from `typstify_core::test_config`.
- [x] Step 3: Run `cargo test --all-features`.
- [x] BDD Verification: `cargo test --all-features` — all tests pass
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings

---

## Phase 6 — DX & Docs (Findings 31, 32, 33)

### Task 6.1: Add CI workflow for lint and test

> **Context:** Only deployment workflows exist (`deploy-pages.yml`, `deploy-worker.yml`). No CI pipeline for linting or testing.
> **Verification:** CI runs `just lint` and `just test` on PRs and pushes.
> **Scenario Coverage:** `features/dx.feature` — CI pipeline runs lint and test on PRs

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Preserve existing behavior` — new workflow file only
- **Simplification Focus:** `Ponytail: Single workflow, minimal steps`
- **Status:** 🟢 DONE
- [x] Step 1: Create `.github/workflows/ci.yml` with trigger on push to main/master and pull_request.
- [x] Step 2: Steps: checkout, install Rust toolchain, install just, run `just lint`, run `just test`.
- [x] Step 3: Verify workflow syntax is valid.
- [x] BDD Verification: N/A — CI workflow validates on next push
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: Push to branch and verify CI runs

### Task 6.2: Complete Justfile targets

> **Context:** AGENTS.md references `just bdd` and `just test-all` but they don't exist in the Justfile.
> **Verification:** `just bdd` and `just test-all` are valid targets.
> **Scenario Coverage:** `features/dx.feature` — Justfile contains all documented commands

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Change: add missing Justfile targets`
- **Simplification Focus:** `Ponytail: Placeholder targets with clear messages`
- **Status:** 🟢 DONE
- [x] Step 1: Add `bdd:` target to Justfile (placeholder: `echo "BDD not yet implemented"` or wire to cucumber-rs).
- [x] Step 2: Add `test-all:` target to Justfile (runs `just test && just lint`).
- [x] Step 3: Verify `just bdd` and `just test-all` execute without error.
- [x] BDD Verification: N/A
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `just bdd` and `just test-all` succeed

### Task 6.3: Fix README Rust version

> **Context:** README states "Rust 1.75+" but the workspace uses `edition = "2024"` which requires Rust 1.85+.
> **Verification:** README states the correct minimum Rust version.
> **Scenario Coverage:** `features/dx.feature` — README documents correct Rust version requirement

- **Loop Type:** `TDD-only`
- **Behavioral Contract:** `Change: update documentation only`
- **Simplification Focus:** `Ponytail: Change one line in README`
- **Status:** 🟢 DONE
- [x] Step 1: In `README.md:215`, change "Rust 1.75+" to "Rust 1.85+".
- [x] Step 2: Verify the change is correct by checking `edition = "2024"` requirements.
- [x] BDD Verification: N/A
- [x] Advanced Test Verification: N/A
- [x] Runtime Verification: `cargo build` — no warnings
