// typstify:frontmatter
// title: "Technical Specification"
// date: 2024-01-22T10:00:00Z
// description: "A Typst document demonstrating technical documentation"
// tags: ["typst", "technical", "spec"]
// draft: false

This document demonstrates Typstify's support for Typst documents.

== Introduction

Typst is a new markup-based typesetting system that is designed to be as powerful as LaTeX while being much easier to learn and use.

== Features

=== Mathematics

Typstify renders Typst math as HTML:

$ integral_0^infinity e^(-x^2) dif x = sqrt(pi) / 2 $

=== Code Blocks

```rust
fn fibonacci(n: u32) -> u32 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}
```

=== Lists

Ordered list:
+ First item
+ Second item
+ Third item

Unordered list:
- Item A
- Item B
- Item C

=== Tables

#table(
  columns: (auto, auto, auto),
  [*Feature*], [*Markdown*], [*Typst*],
  [Math], [KaTeX], [Native],
  [Tables], [GFM], [Native],
  [Figures], [Limited], [Full],
)

== Conclusion

Typst provides a modern alternative to Markdown for technical documentation, with native support for mathematics, tables, and complex layouts.
