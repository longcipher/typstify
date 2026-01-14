//! Article component for rendering page content.
//!
//! Provides safe HTML rendering with custom CSS/JS injection.

use leptos::prelude::*;

/// Article component properties.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ArticleData {
    /// The HTML content to render.
    pub content: String,

    /// Page title.
    pub title: String,

    /// Custom CSS files to include.
    pub custom_css: Vec<String>,

    /// Custom JS files to include.
    pub custom_js: Vec<String>,
}

/// Article component for rendering HTML content.
///
/// Renders pre-rendered HTML content with optional custom CSS and JS.
#[component]
pub fn Article(
    /// The article data to render.
    data: Signal<ArticleData>,
) -> impl IntoView {
    view! {
      <article class="typstify-article">
        // Custom CSS links
        <For
          each=move || data.get().custom_css.clone()
          key=|css| css.clone()
          children=move |css| {
            view! { <link rel="stylesheet" href=css /> }
          }
        />

        // Article header
        <header class="typstify-article-header">
          <h1 class="typstify-article-title">{move || data.get().title.clone()}</h1>
        </header>

        // Article content (rendered HTML)
        <div class="typstify-article-content" inner_html=move || data.get().content.clone()></div>

        // Custom JS scripts (deferred)
        <For
          each=move || data.get().custom_js.clone()
          key=|js| js.clone()
          children=move |js| {
            view! { <script src=js defer=true></script> }
          }
        />

      </article>
    }
}

/// Article metadata component.
#[component]
pub fn ArticleMeta(
    /// Publication date.
    #[prop(optional)]
    date: Option<String>,
    /// Reading time in minutes.
    #[prop(optional)]
    reading_time: Option<u32>,
    /// Tags.
    #[prop(default = vec![])]
    tags: Vec<String>,
) -> impl IntoView {
    let has_date = date.is_some();
    let date_value = date.clone();
    let has_reading_time = reading_time.is_some();
    let reading_time_value = reading_time.unwrap_or(0);
    let has_tags = !tags.is_empty();
    let tags_list = StoredValue::new(tags);

    view! {
      <div class="typstify-article-meta">
        <Show when=move || has_date>
          <time class="typstify-article-date">{date_value.clone()}</time>
        </Show>

        <Show when=move || has_reading_time>
          <span class="typstify-article-reading-time">{reading_time_value} " min read"</span>
        </Show>

        <Show when=move || has_tags>
          <div class="typstify-article-tags">
            <For
              each=move || tags_list.get_value()
              key=|tag| tag.clone()
              children=move |tag| {
                let href = format!("/tags/{}", tag.to_lowercase());
                view! {
                  <a href=href class="typstify-tag">
                    {tag}
                  </a>
                }
              }
            />

          </div>
        </Show>
      </div>
    }
}

/// Prose wrapper for styled article content.
#[component]
pub fn Prose(
    /// Children content.
    children: Children,
) -> impl IntoView {
    view! { <div class="typstify-prose">{children()}</div> }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_article_data_default() {
        let data = ArticleData::default();
        assert!(data.content.is_empty());
        assert!(data.title.is_empty());
        assert!(data.custom_css.is_empty());
        assert!(data.custom_js.is_empty());
    }

    #[test]
    fn test_article_data_creation() {
        let data = ArticleData {
            content: "<p>Hello</p>".to_string(),
            title: "Test Article".to_string(),
            custom_css: vec!["style.css".to_string()],
            custom_js: vec!["script.js".to_string()],
        };

        assert_eq!(data.title, "Test Article");
        assert_eq!(data.custom_css.len(), 1);
        assert_eq!(data.custom_js.len(), 1);
    }
}
