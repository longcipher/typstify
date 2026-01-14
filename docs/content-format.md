# Content Format Guide

Typstify supports two content formats: **Markdown** and **Typst**.

## Markdown

Markdown files use the `.md` extension and support GitHub Flavored Markdown (GFM).

### Frontmatter

Markdown files begin with YAML or TOML frontmatter:

#### YAML Frontmatter

```markdown
---
title: "My Post Title"
date: 2024-01-15
description: "A brief description"
tags: ["rust", "web"]
draft: false
---

Content starts here...
```

#### TOML Frontmatter

```markdown
+++
title = "My Post Title"
date = 2024-01-15
description = "A brief description"
tags = ["rust", "web"]
draft = false
+++

Content starts here...
```

### Frontmatter Fields

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `title` | string | Yes | Page title |
| `date` | date | No | Publication date (YYYY-MM-DD) |
| `description` | string | No | Meta description |
| `tags` | array | No | List of tags |
| `draft` | boolean | No | Mark as draft (default: false) |
| `aliases` | array | No | URL redirects |
| `author` | string | No | Author name |
| `custom_css` | array | No | Additional CSS files |
| `custom_js` | array | No | Additional JS files |

### Markdown Features

#### Headings

```markdown
# Heading 1
## Heading 2
### Heading 3
#### Heading 4
```

#### Emphasis

```markdown
*italic* or _italic_
**bold** or __bold__
***bold italic***
~~strikethrough~~
```

#### Links

```markdown
[Link text](https://example.com)
[Link with title](https://example.com "Title")
[Internal link](/docs/guide)
```

#### Images

```markdown
![Alt text](/images/photo.jpg)
![Alt text](/images/photo.jpg "Image title")
```

#### Code

Inline code: `` `code` ``

Code blocks with syntax highlighting:

````markdown
```rust
fn main() {
    println!("Hello!");
}
```
````

Supported languages: rust, python, javascript, typescript, go, c, cpp, java, and [100+ more](https://github.com/sublimehq/Packages).

#### Lists

Unordered:

```markdown
- Item 1
- Item 2
  - Nested item
```

Ordered:

```markdown
1. First
2. Second
3. Third
```

Task lists:

```markdown
- [x] Completed task
- [ ] Incomplete task
```

#### Tables

```markdown
| Header 1 | Header 2 |
|----------|----------|
| Cell 1   | Cell 2   |
| Cell 3   | Cell 4   |
```

#### Blockquotes

```markdown
> This is a blockquote.
>
> It can span multiple lines.
```

#### Footnotes

```markdown
Here's a sentence with a footnote.[^1]

[^1]: This is the footnote content.
```

#### Horizontal Rules

```markdown
---
```

## Typst

Typst files use the `.typ` extension.

### Frontmatter

Typst frontmatter uses comment syntax:

```typst
// typstify:frontmatter
// title: "Document Title"
// date: 2024-01-15
// description: "Document description"
// tags: ["technical", "docs"]
// draft: false

= Main Heading

Content starts here...
```

### Frontmatter Fields

Same fields as Markdown are supported.

### Typst Features

#### Headings

```typst
= Level 1
== Level 2
=== Level 3
```

#### Emphasis

```typst
_italic_
*bold*
_*bold italic*_
```

#### Links

```typst
#link("https://example.com")[Link text]
```

#### Code

Inline: `` `code` ``

Blocks:

````typst
```rust
fn main() {
    println!("Hello!");
}
```
````

#### Lists

Unordered:

```typst
- Item 1
- Item 2
  - Nested
```

Ordered:

```typst
+ First
+ Second
+ Third
```

#### Mathematics

Inline: `$x^2 + y^2 = z^2$`

Display:

```typst
$ integral_0^infinity e^(-x^2) dif x = sqrt(pi) / 2 $
```

#### Tables

```typst
#table(
  columns: (auto, auto),
  [*Header 1*], [*Header 2*],
  [Cell 1], [Cell 2],
)
```

## Multi-Language Content

### Directory Structure

```text
content/
├── posts/              # Default language (en)
│   ├── hello.md
│   └── guide.md
├── posts.zh/           # Chinese translations
│   ├── hello.md
│   └── guide.md
└── posts.ja/           # Japanese translations
    └── hello.md
```

### Language Detection

- Files in `posts/` → default language
- Files in `posts.{lang}/` → specified language
- The slug determines which posts are translations of each other

### Alternate Links

Typstify automatically generates `<link rel="alternate" hreflang="...">` tags for translations.

## URL Structure

### Posts

| Path | URL |
|------|-----|
| `content/posts/hello.md` | `/posts/hello/` |
| `content/posts/2024/review.md` | `/posts/2024/review/` |

### Index Files

| Path | URL |
|------|-----|
| `content/posts/_index.md` | `/posts/` |
| `content/_index.md` | `/` |

### Static Pages

| Path | URL |
|------|-----|
| `content/about.md` | `/about/` |
| `content/contact.md` | `/contact/` |

## URL Aliases

Redirect old URLs:

```yaml
---
title: "New Location"
aliases: ["/old-url", "/another-old-url"]
---
```

Typstify generates redirect HTML files:

```html
<!DOCTYPE html>
<html>
<head>
  <meta http-equiv="refresh" content="0; url=/new-location/">
</head>
</html>
```

## Draft Posts

Mark posts as drafts:

```yaml
---
title: "Work in Progress"
draft: true
---
```

- Drafts are excluded from production builds
- Include drafts with `--drafts` flag or `typstify watch`

## Custom Assets

### Per-Page CSS

```yaml
custom_css: ["syntax.css", "diagrams.css"]
```

### Per-Page JavaScript

```yaml
custom_js: ["interactive.js"]
```

Files are loaded from the assets directory.
