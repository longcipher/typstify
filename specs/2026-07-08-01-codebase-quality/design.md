# Design: Codebase Quality — Security, Correctness, Tech Debt, Performance, Tests, DX

| Metadata | Details |
| :--- | :--- |
| **Status** | Draft |
| **Created** | 2026-07-08 |
| **Mode** | Full |
| **Priority** | P1 |
| **Planned at** | commit `210049f`, 2026-07-08 |

## Summary

> The typstify codebase (12,319 LOC Rust) has XSS vulnerabilities in its template engine and HTML generation, silent error swallowing in the build pipeline, UTF-8 panics on non-ASCII slugs, significant code duplication across crates, dead code (~700 LOC), and minimal test coverage. This spec consolidates 30 findings into a single execution plan that hardens security, fixes correctness bugs, eliminates duplication, optimizes performance, adds test coverage, and completes the CI/DX story.

## Why this matters

The template engine performs zero HTML escaping (`template.rs:94-125`), meaning any content file can inject arbitrary JavaScript into every generated page. The build pipeline silently swallows page generation failures (`build.rs:241-248`), allowing broken deploys. A UTF-8 byte-slice panic (`html.rs:100-103`) crashes builds on non-ASCII slugs. Meanwhile, 3+ copies of `strip_html`, `html_escape`, and `slugify` exist across crates, `Config` is cloned 8+ times per build, and 7 of 18 `HtmlGenerator` methods have zero tests.

## Approach

Execute in dependency order:

1. **Phase 1 — Shared utilities & security fixes** (findings 13, 1, 5, 9, 10): Extract `html_escape`, `slugify`, `strip_html` into `typstify-core`. Add auto-escaping to the template engine. Escape all user-derived HTML values.

2. **Phase 2 — Correctness fixes** (findings 2, 3, 6, 7, 11, 12, 30): Fix error propagation, UTF-8 panic, RSS paths, asset manifest JSON, wire `load_with_env`, fix fingerprint hash.

3. **Phase 3 — Tech debt cleanup** (findings 14, 15, 16, 17): Deduplicate base context, remove dead crates, consolidate search types, remove unused deps.

4. **Phase 4 — Performance** (findings 18, 19, 20, 21, 22, 23): Pass `&Config` by reference, cache constant values, optimize slugify and escape_xml.

5. **Phase 5 — Test coverage** (findings 24-29): Fix e2e tests, add unit tests for untested modules, extract shared fixtures.

6. **Phase 6 — DX & docs** (findings 31, 32, 33): Add CI workflow, complete Justfile, fix README.

## Findings

### Finding 1: XSS via unescaped template variable interpolation

- **Category:** security
- **Impact:** HIGH — stored XSS exploitable by any content author
- **Effort:** M

#### Requirements (EARS Notation)

- **[REQ-01]:** The template engine MUST HTML-escape all `{{ variable }}` substitutions by default.
- **[REQ-02]:** The template engine MUST support `{{ variable|raw }}` syntax for intentional HTML content.
- **[REQ-03]:** All existing templates MUST be audited to classify variables as escaped or raw.

#### Current state

`template.rs:94-125` performs raw string replacement with zero escaping. The `render()` method replaces `{{ var }}` with the raw context value via `result.replace_range(start..end, &value)`.

#### Approach

Add an `html_escape()` function (extracted from `markdown.rs`). In `Template::render()`, escape all values by default. Add `|raw` suffix detection: if `var_name` ends with `|raw`, skip escaping. Audit all 12+ hardcoded templates in `template.rs` to classify each variable.

#### Architecture Decisions (MADR Format)

- **AD-01:** Use pipe syntax (`|raw`) rather than a separate function (`raw()`) because it matches common template engine conventions (Jinja2, Liquid).

### Finding 2: Page generation errors silently swallowed

- **Category:** correctness
- **Impact:** HIGH — silent broken deploys
- **Effort:** S

#### Requirements (EARS Notation)

- **[REQ-04]:** The build MUST return an error when any page generation fails.
- **[REQ-05]:** The error message MUST list all failed page paths.

#### Current state

`build.rs:241-248` logs `warn!()` on failure and continues. The caller never checks if `count < pages.len()`.

#### Approach

Collect `Err` results into a `Vec<BuildError>`. After the loop, if non-empty, return `Err(BuildError::PageGenerationFailed { errors })`. The error variant should list all failed paths.

### Finding 3: UTF-8 byte-slice panic on non-ASCII slugs

- **Category:** correctness
- **Impact:** HIGH — build panic
- **Effort:** S

#### Current state

`html.rs:100-103`: `c.to_uppercase().collect::<String>() + &section[1..]` — `section[1..]` is a byte offset, not a char offset. Panics if `section` starts with a multi-byte UTF-8 character.

#### Approach

Replace with character-based slicing:

```rust
let rest = &section[c.len_utf8()..];
```

### Finding 5: Unescaped page titles in HTML attributes

- **Category:** security
- **Impact:** HIGH — attribute injection XSS
- **Effort:** S

#### Approach

Apply the shared `html_escape()` at every interpolation site in `html.rs` where `p.title`, `p.url`, or tag names appear in HTML output.

### Finding 9: Raw HTML passthrough in markdown

- **Category:** security
- **Impact:** MED — script injection from content authors
- **Effort:** S

#### Approach

Add `sanitize_html: bool` to `BuildConfig`. When enabled, strip `<script>`, `<iframe>`, `<object>` tags and event handler attributes from `Event::Html`/`Event::InlineHtml` in `markdown.rs`.

### Finding 13: Shared utilities extraction

- **Category:** tech-debt
- **Impact:** MED — correctness depends on which copy is called
- **Effort:** S

#### Approach

Create `typstify-core/src/utils.rs` with canonical `html_escape()`, `slugify()`, `strip_html()`. The `strip_html` implementation should be the `indexer.rs` version (handles script/style tags, entities, whitespace). All other crates import from `typstify-core`.

## Architecture Decisions

### AD-02: Place shared utilities in typstify-core

`typstify-core` is already depended upon by all other crates. Adding utilities here avoids a new crate and keeps the dependency tree clean.

### AD-03: Template auto-escaping with |raw opt-in

Default-escape is safer than opt-in escaping. The `|raw` suffix is a well-understood convention from Jinja2/Liquid.

### AD-04: Pass Config by reference, not clone

Generators receive `&Config` with appropriate lifetimes. The `Config` struct is not mutated during builds, so shared references are safe.

### AD-05: Remove dead crates rather than wire them in

`typstify-ui` and `typstify-search-wasm` are unused. Removing them eliminates ~700 LOC and 10+ dependencies. They can be re-added from git history when needed.

## BDD/TDD Strategy

- **Primary Language:** Rust
- **BDD Runner:** cucumber-rs (placeholder — not yet set up; scenarios serve as acceptance criteria for manual verification)
- **BDD Command:** `just bdd` (to be added)
- **Unit Test Command:** `cargo test --all-features`
- **Feature Files:** `specs/2026-07-08-01-codebase-quality/features/*.feature`
- **Outside-in Loop:** Each finding has unit tests first; BDD scenarios validate end-to-end behavior

## Code Simplification Constraints

**Ponytail Ladder (mandatory at every decision point):**

1. Does this need to exist at all? Speculative need = skip it. (YAGNI)
2. Stdlib does it? Use it.
3. Native platform feature covers it? Use it.
4. Already-installed dependency? Use it.
5. One line? One line.
6. Only then: minimum code that works.

**Mark deferrals:** Use `ponytail:` comments for deliberate simplifications with known ceilings.

**Never simplify away:** input validation, error handling, security, accessibility, anything explicitly requested.

**Additional constraints:**

- **Behavioral Contract:** Preserve existing behavior unless a listed scenario or requirement explicitly changes it.
- **Repo Standards:** Use only the coding standards established by `AGENTS.md`, `CLAUDE.md`, and the existing codebase.
- **Readability Priorities:** Prefer explicit control flow, clear names, reduced nesting. Avoid dense or clever rewrites.
- **Refactor Scope:** Limit cleanup to touched modules unless the design explicitly justifies broader refactor.

## BDD Scenario Inventory

- `features/security.feature` — Template variables are HTML-escaped by default → Task 1.1
- `features/security.feature` — Template variables with raw suffix preserve HTML → Task 1.2
- `features/security.feature` — Page titles in list pages are HTML-escaped → Task 1.3
- `features/security.feature` — Tag names in tag index are HTML-escaped → Task 1.3
- `features/security.feature` — Raw HTML in markdown is sanitized when enabled → Task 1.4
- `features/security.feature` — Typst source path is not leaked → Task 1.5
- `features/correctness.feature` — Build fails when page generation errors occur → Task 2.1
- `features/correctness.feature` — Non-ASCII section slugs do not cause panics → Task 2.2
- `features/correctness.feature` — RSS feed paths are correct → Task 2.3
- `features/correctness.feature` — Asset manifest produces valid JSON → Task 2.4
- `features/correctness.feature` — Environment variable overrides work → Task 2.5
- `features/correctness.feature` — Asset fingerprints use sufficient hash bits → Task 2.6
- `features/correctness.feature` — Dev server debounce does not lose events → Task 2.7
- `features/tech-debt.feature` — Shared HTML escape utility → Task 1.1
- `features/tech-debt.feature` — Shared slugify utility → Task 3.1
- `features/tech-debt.feature` — Shared strip_html utility → Task 3.1
- `features/tech-debt.feature` — Base template context helper → Task 3.2
- `features/tech-debt.feature` — Dead crates removed → Task 3.3
- `features/tech-debt.feature` — Duplicated search types consolidated → Task 3.4
- `features/tech-debt.feature` — Unused workspace deps removed → Task 3.5
- `features/performance.feature` — Config not cloned unnecessarily → Task 4.1
- `features/performance.feature` — base_url computed once → Task 4.2
- `features/performance.feature` — Year string computed once → Task 4.3
- `features/performance.feature` — Section nav/lang switcher cached → Task 4.4
- `features/performance.feature` — slug_from_str single-pass → Task 3.1
- `features/performance.feature` — escape_xml single-pass → Task 4.5
- `features/test.feature` — E2e tests use bundled fixtures → Task 5.1
- `features/test.feature` — robots.rs unit tests → Task 5.2
- `features/test.feature` — ContentCollector::collect tested → Task 5.3
- `features/test.feature` — HtmlGenerator methods tested → Task 5.4
- `features/test.feature` — Multi-language build tested → Task 5.5
- `features/test.feature` — Shared test fixtures → Task 5.6
- `features/dx.feature` — CI pipeline → Task 6.1
- `features/dx.feature` — Justfile complete → Task 6.2
- `features/dx.feature` — README correct Rust version → Task 6.3

## Existing Components to Reuse

- `html_escape()` exists in `markdown.rs:322` and `syntax.rs:86` — consolidate into `typstify-core`
- `slugify()` exists in `markdown.rs:330`, `typst_parser.rs:137`, `html.rs:869` — consolidate
- `strip_html()` exists in `content.rs:297`, `simple.rs:313`, `indexer.rs:211` — consolidate
- `test_config()` exists in 5 test modules — extract shared fixture
- `tempfile::TempDir` pattern in `build.rs` tests — reuse for new tests

## Verification

| Purpose   | Command                          | Expected on success |
|-----------|----------------------------------|---------------------|
| Build     | `cargo build`                    | exit 0, no warnings |
| Lint      | `just lint`                      | exit 0, no errors   |
| Tests     | `just test`                      | all pass            |
| Shear     | `cargo shear`                    | no unused deps      |
| BDD       | `just bdd` (after setup)         | all pass            |
