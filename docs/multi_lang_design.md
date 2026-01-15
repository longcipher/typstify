# Multi-Language (i18n) Design Document

## Overview

This document describes the multi-language support design for Typstify, inspired by Zola's approach but optimized for Typstify's Rust CLI architecture.

## Design Goals

1. **Simple file naming convention** - Language suffix in filename (e.g., `hello.zh.md`)
2. **Automatic language detection** - No need to specify language in frontmatter
3. **Translation linking** - Automatically link translated versions of the same content
4. **Flexible URL structure** - Default language at root, other languages under `/{lang}/`
5. **Language-specific metadata** - Per-language site title, description, etc.
6. **hreflang support** - SEO-friendly alternate language tags

## Configuration Structure

### config.toml

```toml
[site]
title = "My Typstify Site"
description = "A multi-language blog"
base_url = "https://example.com"
default_language = "en"

# Language configurations (optional, for additional languages)
[languages.zh]
title = "我的 Typstify 站点"           # Override site title for this language
description = "一个多语言博客"          # Override description

[languages.ja]
title = "私のTypstifyサイト"
description = "多言語ブログ"
```

### Key Points

- `default_language` defines the primary language (content without suffix)
- `[languages.{code}]` sections define additional languages
- Each language can override `title`, `description`, and other site-level settings
- Languages are auto-detected from filename patterns

## File Structure Convention

### Filename Patterns

```text
content/
├── posts/
│   ├── hello-world.md          # Default language (en)
│   ├── hello-world.zh.md       # Chinese translation
│   ├── hello-world.ja.md       # Japanese translation
│   ├── getting-started.md      # English only
│   └── advanced/
│       ├── index.md            # Default language
│       └── index.zh.md         # Chinese translation
├── about.md                    # Default language
├── about.zh.md                 # Chinese translation
└── docs/
    ├── guide.typ               # Typst content (default lang)
    └── guide.zh.typ            # Typst content (Chinese)
```

### URL Generation

| File | Language | URL |
|------|----------|-----|
| `hello-world.md` | en (default) | `/posts/hello-world/` |
| `hello-world.zh.md` | zh | `/zh/posts/hello-world/` |
| `hello-world.ja.md` | ja | `/ja/posts/hello-world/` |
| `about.md` | en (default) | `/about/` |
| `about.zh.md` | zh | `/zh/about/` |

## Data Structures

### LanguageConfig

```rust
/// Configuration for a specific language.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// Display name of the language (e.g., "中文", "日本語")
    #[serde(default)]
    pub name: Option<String>,

    /// Override site title for this language
    #[serde(default)]
    pub title: Option<String>,

    /// Override site description for this language
    #[serde(default)]
    pub description: Option<String>,
}
```

### Updated SiteConfig

```rust
/// Site-wide configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteConfig {
    pub title: String,
    pub base_url: String,
    pub default_language: String,

    /// Language-specific configurations
    #[serde(default)]
    pub languages: HashMap<String, LanguageConfig>,

    // ... other fields
}
```

### Page with Translation Support

```rust
/// A fully processed page ready for rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    // ... existing fields ...

    /// Language code for this page
    pub lang: String,  // Changed from Option<String>

    /// Whether this is the default language version
    pub is_default_lang: bool,

    /// Canonical ID for linking translations (slug without lang prefix)
    pub canonical_id: String,
}
```

### SiteContent with Translation Index

```rust
/// Collected site content.
#[derive(Debug, Default)]
pub struct SiteContent {
    /// All pages indexed by URL
    pub pages: HashMap<String, Page>,

    /// Pages organized by section
    pub sections: HashMap<String, Vec<String>>,

    /// Taxonomy indices
    pub taxonomies: TaxonomyIndex,

    /// Translation index: canonical_id -> {lang -> url}
    pub translations: HashMap<String, HashMap<String, String>>,

    /// Pages organized by language: lang -> [urls]
    pub by_language: HashMap<String, Vec<String>>,
}
```

## Content Path Parsing

### Algorithm

1. Read file path (e.g., `posts/hello-world.zh.md`)
2. Extract extension (`.md`, `.typ`)
3. Extract stem (e.g., `hello-world.zh`)
4. Check if stem ends with `.{lang_code}` where `lang_code` is 2-3 lowercase letters
5. If language suffix found:
   - Extract language code (`zh`)
   - Extract base stem (`hello-world`)
6. If no suffix, use `default_language`
7. Generate:
   - `canonical_id`: section + base_stem (e.g., `posts/hello-world`)
   - `lang`: detected or default language
   - `url`: `/{lang}/{canonical_id}` or `/{canonical_id}` for default

### ContentPath Implementation

```rust
impl ContentPath {
    pub fn from_path(path: &Path, config: &Config) -> Option<Self> {
        let extension = path.extension()?.to_str()?;
        let content_type = ContentType::from_extension(extension)?;

        let stem = path.file_stem()?.to_str()?;
        let parent = path.parent().unwrap_or(Path::new(""));

        // Check for language suffix
        let (base_stem, lang, is_default_lang) = if let Some(dot_pos) = stem.rfind('.') {
            let potential_lang = &stem[dot_pos + 1..];
            if is_valid_lang_code(potential_lang) && config.has_language(potential_lang) {
                let is_default = potential_lang == config.site.default_language;
                (
                    &stem[..dot_pos],
                    potential_lang.to_string(),
                    is_default,
                )
            } else {
                (stem, config.site.default_language.clone(), true)
            }
        } else {
            (stem, config.site.default_language.clone(), true)
        };

        // Build canonical_id (without language)
        let canonical_id = if base_stem == "index" {
            parent.to_string_lossy().to_string()
        } else if parent.as_os_str().is_empty() {
            base_stem.to_string()
        } else {
            format!("{}/{}", parent.display(), base_stem)
        };

        // Build URL
        let url = if is_default_lang {
            format!("/{}", canonical_id)
        } else {
            format!("/{}/{}", lang, canonical_id)
        };

        Some(Self {
            path: path.to_path_buf(),
            lang,
            is_default_lang,
            canonical_id,
            url,
            content_type,
        })
    }
}

fn is_valid_lang_code(s: &str) -> bool {
    s.len() >= 2 
        && s.len() <= 3 
        && s.chars().all(|c| c.is_ascii_lowercase())
}
```

## Template Variables

### Language Switching

Templates can access:

```html
<!-- Current page info -->
{{ lang }}                  <!-- "en", "zh", etc. -->
{{ is_default_lang }}       <!-- true/false -->
{{ canonical_id }}          <!-- "posts/hello-world" -->

<!-- Available translations -->
{% for trans in translations %}
  <a href="{{ trans.url }}" hreflang="{{ trans.lang }}">
    {{ trans.name }}  <!-- "English", "中文" -->
  </a>
{% endfor %}

<!-- Language-specific site info -->
{{ site_title }}            <!-- Uses language override if available -->
{{ site_description }}
```

### hreflang Tags (SEO)

Generated automatically in `<head>`:

```html
<link rel="alternate" hreflang="en" href="https://example.com/posts/hello/" />
<link rel="alternate" hreflang="zh" href="https://example.com/zh/posts/hello/" />
<link rel="alternate" hreflang="x-default" href="https://example.com/posts/hello/" />
```

## Build Process Changes

### Collection Phase

1. Walk content directory, parse all files
2. Extract language from filename
3. Build `translations` index grouping pages by `canonical_id`
4. Build `by_language` index for language-specific pages

### Generation Phase

1. Generate pages as before, but with language context
2. For each page, inject `translations` list for language switcher
3. Generate language-specific auto pages:
   - `/tags/` and `/{lang}/tags/` for each language
   - `/archives/` and `/{lang}/archives/`
   - Section indices per language
4. Generate hreflang links

### RSS/Sitemap

- Generate per-language RSS feeds (`/rss.xml`, `/zh/rss.xml`)
- Sitemap includes all language versions with hreflang

## Migration from Current Structure

### Before (posts.zh/ directory)

```text
content/
├── posts/
│   └── hello-world.md
└── posts.zh/
    └── hello-world.md
```

### After (filename suffix)

```text
content/
└── posts/
    ├── hello-world.md
    └── hello-world.zh.md
```

### Benefits

1. Translations are co-located (easier to maintain)
2. Clear visual association between translations
3. No duplicate directory structures
4. Simpler file system navigation
5. Same pattern works for any content type (pages, docs, etc.)

## Implementation Plan

### Phase 1: Config Structure ✅ COMPLETE

- [x] Add `LanguageConfig` struct to `config.rs`
- [x] Move `languages` from `SiteConfig` to `Config` as `HashMap<String, LanguageConfig>`
- [x] Add helper methods: `has_language()`, `all_languages()`, `title_for_language()`, etc.
- [x] Update config parsing and tests

### Phase 2: Content Path ✅ COMPLETE

- [x] Update `ContentPath` with `lang: String`, `is_default_lang: bool`, `canonical_id: String`
- [x] Add filename-based language detection logic
- [x] Update URL generation to include language prefix for non-default languages
- [x] Update `Page` struct with new i18n fields

### Phase 3: Collection ⏳ PARTIAL

- [x] Updated `Page` struct with translation fields
- [ ] Add `translations` index to `SiteContent`
- [ ] Add `by_language` index to `SiteContent`
- [ ] Build translation index during collection

### Phase 4: Templates 🔲 TODO

- [ ] Add translation variables to template context
- [ ] Generate hreflang tags in HTML head
- [ ] Add language switcher component template

### Phase 5: Auto Pages 🔲 TODO

- [ ] Generate per-language archives (`/archives/`, `/zh/archives/`)
- [ ] Generate per-language section indices (`/posts/`, `/zh/posts/`)
- [ ] Generate per-language taxonomy pages (`/tags/`, `/zh/tags/`)

### Phase 6: RSS/Sitemap ⏳ PARTIAL

- [x] Multi-language sitemap with hreflang alternates (basic)
- [ ] Per-language RSS feeds (`/rss.xml`, `/zh/rss.xml`)
- [ ] Language-specific RSS titles/descriptions

### Phase 7: Examples ✅ COMPLETE

- [x] Update `examples/blog/config.toml` with new structure
- [x] Migrate `posts.zh/` directory to filename suffix pattern
- [x] Update documentation

## Example Usage

### Basic Multi-Language Site

```text
content/
├── index.md              # English home
├── index.zh.md           # Chinese home
├── about.md              # English about
├── about.zh.md           # Chinese about
└── posts/
    ├── hello.md          # English post
    ├── hello.zh.md       # Chinese translation
    ├── rust-tips.md      # English only
    └── china-trip.zh.md  # Chinese only
```

### Generated URLs

| File | URL |
|------|-----|
| index.md | `/` |
| index.zh.md | `/zh/` |
| posts/hello.md | `/posts/hello/` |
| posts/hello.zh.md | `/zh/posts/hello/` |
| posts/rust-tips.md | `/posts/rust-tips/` |
| posts/china-trip.zh.md | `/zh/posts/china-trip/` |

### Auto-Generated Pages

- `/tags/` - English tags index
- `/zh/tags/` - Chinese tags index
- `/archives/` - English archives
- `/zh/archives/` - Chinese archives
- `/posts/` - English posts list
- `/zh/posts/` - Chinese posts list
