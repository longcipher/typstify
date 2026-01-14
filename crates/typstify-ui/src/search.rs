//! Search components for the Typstify frontend.
//!
//! Provides SearchBox, SearchResults, and SearchModal Leptos components.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// A single search result item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SearchResultItem {
    /// Result URL.
    pub url: String,

    /// Result title.
    pub title: String,

    /// Result description/snippet.
    #[serde(default)]
    pub description: Option<String>,

    /// Relevance score.
    #[serde(default)]
    pub score: f32,
}

/// Search box input component.
///
/// Provides a text input with debounced search callback.
#[component]
pub fn SearchBox(
    /// Placeholder text for the input.
    #[prop(default = "Search...".to_string())]
    placeholder: String,
    /// Signal to track the current query.
    query: RwSignal<String>,
    /// Whether search is loading.
    #[prop(default = false.into())]
    loading: Signal<bool>,
) -> impl IntoView {
    let input_ref = NodeRef::<leptos::html::Input>::new();

    // Focus input on mount
    Effect::new(move |_| {
        if let Some(input) = input_ref.get() {
            let _ = input.focus();
        }
    });

    view! {
      <div class="typstify-search-box">
        <input
          node_ref=input_ref
          type="text"
          class="typstify-search-input"
          placeholder=placeholder
          prop:value=move || query.get()
          on:input=move |ev| {
            let value = event_target_value(&ev);
            query.set(value);
          }
        />
        <Show when=move || loading.get()>
          <span class="typstify-search-spinner" aria-label="Loading"></span>
        </Show>
      </div>
    }
}

/// Search results list component.
#[component]
pub fn SearchResults(
    /// The search results to display.
    results: Signal<Vec<SearchResultItem>>,
    /// The current search query (for highlighting).
    #[prop(default = "".to_string().into())]
    query: Signal<String>,
) -> impl IntoView {
    view! {
      <div class="typstify-search-results">
        <Show
          when=move || !results.get().is_empty()
          fallback=move || {
            let q = query.get();
            if q.is_empty() {
              view! { <div class="typstify-search-empty"></div> }.into_any()
            } else {
              view! {
                <div class="typstify-search-no-results">"No results found for \"" {q} "\""</div>
              }
                .into_any()
            }
          }
        >

          <ul class="typstify-search-list">
            <For
              each=move || results.get()
              key=|item| item.url.clone()
              children=move |item| {
                view! { <SearchResultItem item=item /> }
              }
            />

          </ul>
        </Show>
      </div>
    }
}

/// Individual search result item component.
#[component]
fn SearchResultItem(
    /// The result item to display.
    item: SearchResultItem,
) -> impl IntoView {
    let description = item.description.clone();
    let has_description = description.is_some();

    view! {
      <li class="typstify-search-item">
        <a href=item.url.clone() class="typstify-search-link">
          <span class="typstify-search-title">{item.title.clone()}</span>
          <Show when=move || has_description>
            <span class="typstify-search-description">
              {description.clone().unwrap_or_default()}
            </span>
          </Show>
        </a>
      </li>
    }
}

/// Search modal component with keyboard shortcuts.
#[component]
pub fn SearchModal(
    /// Whether the modal is open.
    open: RwSignal<bool>,
    /// The search query.
    query: RwSignal<String>,
    /// The search results.
    results: Signal<Vec<SearchResultItem>>,
    /// Whether search is loading.
    #[prop(default = false.into())]
    loading: Signal<bool>,
) -> impl IntoView {
    // Close on Escape key
    let on_keydown = move |ev: web_sys::KeyboardEvent| {
        if ev.key() == "Escape" {
            open.set(false);
        }
    };

    // Close when clicking overlay
    let on_overlay_click = move |_| {
        open.set(false);
    };

    // Prevent closing when clicking modal content
    let on_content_click = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
    };

    view! {
      <Show when=move || open.get()>
        <div class="typstify-modal-overlay" on:click=on_overlay_click on:keydown=on_keydown>
          <div class="typstify-modal-content" on:click=on_content_click>
            <div class="typstify-modal-header">
              <SearchBox query=query loading=loading />
              <button
                class="typstify-modal-close"
                on:click=move |_| open.set(false)
                aria-label="Close search"
              >
                "×"
              </button>
            </div>
            <div class="typstify-modal-body">
              <SearchResults results=results query=query.into() />
            </div>
            <div class="typstify-modal-footer">
              <span class="typstify-modal-hint">"Press Esc to close"</span>
            </div>
          </div>
        </div>
      </Show>
    }
}

/// Hook to set up global keyboard shortcut for search modal.
///
/// Opens the modal when Cmd/Ctrl + K is pressed.
#[component]
#[allow(clippy::unused_unit)]
pub fn SearchShortcut(
    /// Signal to control modal open state.
    open: RwSignal<bool>,
) -> impl IntoView {
    Effect::new(move |_| {
        use wasm_bindgen::{JsCast, prelude::*};

        let open = open;
        let handler =
            Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |ev: web_sys::KeyboardEvent| {
                // Cmd+K (Mac) or Ctrl+K (Windows/Linux)
                if ev.key() == "k" && (ev.meta_key() || ev.ctrl_key()) {
                    ev.prevent_default();
                    open.set(true);
                }
            });

        let window = web_sys::window().expect("no window");
        let _ =
            window.add_event_listener_with_callback("keydown", handler.as_ref().unchecked_ref());

        // Leak the closure to keep it alive
        handler.forget();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_result_item_creation() {
        let item = SearchResultItem {
            url: "/test".to_string(),
            title: "Test Page".to_string(),
            description: Some("A test description".to_string()),
            score: 10.5,
        };

        assert_eq!(item.url, "/test");
        assert_eq!(item.title, "Test Page");
        assert!(item.description.is_some());
    }

    #[test]
    fn test_search_result_item_without_description() {
        let item = SearchResultItem {
            url: "/test".to_string(),
            title: "Test".to_string(),
            description: None,
            score: 0.0,
        };

        assert!(item.description.is_none());
    }

    #[test]
    fn test_search_result_serialization() {
        let item = SearchResultItem {
            url: "/test".to_string(),
            title: "Test".to_string(),
            description: None,
            score: 5.0,
        };

        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("\"url\":\"/test\""));
        assert!(json.contains("\"title\":\"Test\""));
    }
}
