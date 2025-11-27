use atom_syndication::{Entry, Feed, Link, Person, Text};
use chrono::{DateTime, Utc};

use crate::{config::AppConfig, content::Content};

pub fn create_feed(config: &AppConfig, content: &[Content]) -> Feed {
    let mut feed = Feed::default();

    // Set feed metadata
    feed.set_title(config.site.title.clone());
    feed.set_subtitle(Text::plain(config.site.description.clone()));

    // Set feed link
    let feed_link = Link {
        href: format!("{}/{}", config.site.base_url, config.feed.filename),
        rel: "self".to_string(),
        mime_type: Some("application/atom+xml".to_string()),
        hreflang: None,
        title: None,
        length: None,
    };
    feed.set_links(vec![feed_link]);

    // Set feed ID (usually the website URL)
    feed.set_id(config.site.base_url.clone());

    // Set updated time to the most recent content
    let now = Utc::now();
    if let Some(latest_content) = content.first() {
        if let Some(date) = latest_content.metadata.get_date() {
            if let Ok(parsed_date) = DateTime::parse_from_rfc3339(date) {
                feed.set_updated(parsed_date.with_timezone(&Utc));
            } else if let Ok(parsed_date) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                feed.set_updated(parsed_date.and_hms_opt(0, 0, 0).unwrap().and_utc());
            } else {
                // Use current time as fallback
                feed.set_updated(now);
            }
        } else {
            feed.set_updated(now);
        }
    } else {
        feed.set_updated(now);
    }

    // Create entries from content
    let mut entries = Vec::new();

    for content_item in content.iter().take(config.feed.max_items) {
        // Skip draft content
        if content_item.metadata.is_draft() {
            continue;
        }

        let mut entry = Entry::default();

        // Set entry title
        entry.set_title(content_item.metadata.get_title());

        // Set entry ID and link
        let content_url = format!(
            "{}/{}",
            config.site.base_url.trim_end_matches('/'),
            content_item.slug()
        );
        entry.set_id(content_url.clone());
        entry.set_links(vec![Link {
            href: content_url,
            rel: "alternate".to_string(),
            mime_type: Some("text/html".to_string()),
            hreflang: None,
            title: None,
            length: None,
        }]);

        // Set entry content
        let summary = content_item.metadata.get_description();
        if !summary.is_empty() {
            entry.set_summary(Some(Text::html(summary)));
        }

        // Set entry author
        if let Some(author) = content_item.metadata.get_author() {
            entry.set_authors(vec![Person {
                name: author.to_string(),
                email: None,
                uri: None,
            }]);
        } else {
            // Use site author as fallback
            entry.set_authors(vec![Person {
                name: config.site.author.clone(),
                email: None,
                uri: None,
            }]);
        }

        // Set published date if available
        if let Some(date_str) = content_item.metadata.get_date() {
            // Try to parse as RFC3339 first, then as simple date
            if let Ok(fixed_date) = DateTime::parse_from_rfc3339(date_str) {
                entry.set_published(Some(fixed_date));
            } else if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                let fixed_date = naive_date
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc()
                    .fixed_offset();
                entry.set_published(Some(fixed_date));
            } else {
                // Fallback to current time if date parsing fails
                entry.set_published(Some(now.fixed_offset()));
            }
        } else {
            // Use current time as fallback
            entry.set_published(Some(now.fixed_offset()));
        }

        // Set entry categories/tags
        let categories: Vec<_> = content_item
            .metadata
            .tags
            .iter()
            .map(|tag| atom_syndication::Category {
                term: tag.clone(),
                scheme: None,
                label: Some(tag.clone()),
            })
            .collect();
        entry.set_categories(categories);

        entries.push(entry);
    }

    feed.set_entries(entries);
    feed
}
