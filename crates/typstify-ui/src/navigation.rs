//! Navigation components for site navigation and table of contents.
//!
//! Provides Navigation, NavLink, and TableOfContents components.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// A navigation item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NavItem {
    /// Display label.
    pub label: String,

    /// Link URL.
    pub url: String,

    /// Whether this is the active/current page.
    #[serde(default)]
    pub active: bool,

    /// Child navigation items.
    #[serde(default)]
    pub children: Vec<NavItem>,
}

impl NavItem {
    /// Create a new navigation item.
    pub fn new(label: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            url: url.into(),
            active: false,
            children: Vec::new(),
        }
    }

    /// Set this item as active.
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Add child items.
    pub fn with_children(mut self, children: Vec<NavItem>) -> Self {
        self.children = children;
        self
    }
}

/// Table of contents entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TocEntry {
    /// Heading level (1-6).
    pub level: u8,

    /// Heading text.
    pub text: String,

    /// Anchor ID.
    pub id: String,
}

impl TocEntry {
    /// Create a new TOC entry.
    pub fn new(level: u8, text: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            level,
            text: text.into(),
            id: id.into(),
        }
    }
}

/// Main navigation component.
#[component]
pub fn Navigation(
    /// Navigation items.
    items: Signal<Vec<NavItem>>,
    /// Current path for active highlighting.
    #[prop(default = "/".to_string().into())]
    current_path: Signal<String>,
) -> impl IntoView {
    view! {
      <nav class="typstify-nav" aria-label="Main navigation">
        <ul class="typstify-nav-list">
          <For
            each=move || items.get()
            key=|item| item.url.clone()
            children=move |item| {
              view! { <NavLinkWithChildren item=item current_path=current_path /> }
            }
          />

        </ul>
      </nav>
    }
}

/// Navigation link component with support for child items.
#[component]
fn NavLinkWithChildren(
    /// The navigation item.
    item: NavItem,
    /// Current path for active highlighting.
    current_path: Signal<String>,
) -> impl IntoView {
    let url = item.url.clone();
    let has_children = !item.children.is_empty();
    let children_list = StoredValue::new(item.children.clone());

    let is_active = Memo::new(move |_| {
        let current = current_path.get();
        current == url || current.starts_with(&format!("{url}/"))
    });

    view! {
      <li class="typstify-nav-item" class:active=is_active>
        <a
          href=item.url.clone()
          class="typstify-nav-link"
          aria-current=move || { if is_active.get() { Some("page") } else { None } }
        >
          {item.label.clone()}
        </a>

        <Show when=move || has_children>
          <ul class="typstify-nav-children">
            <For
              each=move || children_list.get_value()
              key=|child| child.url.clone()
              children=move |child| {
                let child_url = child.url.clone();
                let is_child_active = Memo::new(move |_| {
                  let current = current_path.get();
                  current == child_url || current.starts_with(&format!("{child_url}/"))
                });
                // Only render single-level children (no recursion)
                view! {
                  <li class="typstify-nav-item" class:active=is_child_active>
                    <a
                      href=child.url.clone()
                      class="typstify-nav-link"
                      aria-current=move || {
                        if is_child_active.get() { Some("page") } else { None }
                      }
                    >

                      {child.label.clone()}
                    </a>
                  </li>
                }
              }
            />

          </ul>
        </Show>
      </li>
    }
}

/// Table of contents component.
#[component]
pub fn TableOfContents(
    /// TOC entries.
    entries: Signal<Vec<TocEntry>>,
    /// Currently active heading ID.
    #[prop(default = "".to_string().into())]
    active_id: Signal<String>,
) -> impl IntoView {
    view! {
      <nav class="typstify-toc" aria-label="Table of contents">
        <h2 class="typstify-toc-title">"On this page"</h2>
        <ul class="typstify-toc-list">
          <For
            each=move || entries.get()
            key=|entry| entry.id.clone()
            children=move |entry| {
              let id = entry.id.clone();
              let is_active = Memo::new(move |_| active_id.get() == id);
              let indent_class = format!("typstify-toc-level-{}", entry.level);
              let href = format!("#{}", entry.id);

              view! {
                <li class=indent_class class:active=is_active>
                  <a href=href class="typstify-toc-link">
                    {entry.text.clone()}
                  </a>
                </li>
              }
            }
          />

        </ul>
      </nav>
    }
}

/// Breadcrumb navigation component.
#[component]
pub fn Breadcrumbs(
    /// Breadcrumb items (label, url).
    items: Signal<Vec<(String, String)>>,
) -> impl IntoView {
    view! {
      <nav class="typstify-breadcrumbs" aria-label="Breadcrumb">
        <ol class="typstify-breadcrumb-list">
          <For
            each=move || {
              let items_vec = items.get();
              let len = items_vec.len();
              items_vec
                .into_iter()
                .enumerate()
                .map(move |(i, (label, url))| (i, label, url, i == len - 1))
                .collect::<Vec<_>>()
            }

            key=|(i, _, _, _)| *i
            children=move |(_, label, url, is_last)| {
              let label_for_fallback = label.clone();
              let label_for_link = label.clone();
              view! {
                <li class="typstify-breadcrumb-item">
                  <Show
                    when=move || !is_last
                    fallback=move || {
                      view! {
                        <span class="typstify-breadcrumb-current" aria-current="page">
                          {label_for_fallback.clone()}
                        </span>
                      }
                    }
                  >

                    <a href=url.clone() class="typstify-breadcrumb-link">
                      {label_for_link.clone()}
                    </a>
                    <span class="typstify-breadcrumb-separator" aria-hidden="true">
                      "/"
                    </span>
                  </Show>
                </li>
              }
            }
          />

        </ol>
      </nav>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nav_item_creation() {
        let item = NavItem::new("Home", "/");
        assert_eq!(item.label, "Home");
        assert_eq!(item.url, "/");
        assert!(!item.active);
        assert!(item.children.is_empty());
    }

    #[test]
    fn test_nav_item_with_active() {
        let item = NavItem::new("Home", "/").with_active(true);
        assert!(item.active);
    }

    #[test]
    fn test_nav_item_with_children() {
        let child = NavItem::new("Child", "/child");
        let parent = NavItem::new("Parent", "/parent").with_children(vec![child]);

        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0].label, "Child");
    }

    #[test]
    fn test_toc_entry_creation() {
        let entry = TocEntry::new(2, "Introduction", "introduction");
        assert_eq!(entry.level, 2);
        assert_eq!(entry.text, "Introduction");
        assert_eq!(entry.id, "introduction");
    }

    #[test]
    fn test_nav_item_serialization() {
        let item = NavItem::new("Test", "/test");
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"label\":\"Test\""));
        assert!(json.contains("\"url\":\"/test\""));
    }
}
