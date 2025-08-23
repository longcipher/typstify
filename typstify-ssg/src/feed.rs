use atom_syndication::{Feed, Entry, Link, Person, Content as AtomContent, Text, TextType};
use crate::config::BuildConfig;
use crate::content::Content;

pub fn create_feed(config: &BuildConfig, content: &[Content]) -> Feed {
    let mut feed = Feed::default();
    
    // Set feed metadata
    feed.set_title(config.website_name);
    if config.website_tagline.is_some() {
        feed.set_subtitle(Text::plain(config.website_tagline.unwrap()));
    }
    
    // Set feed link
    let feed_link = Link {
        href: format!("{}/atom.xml", config.absolute_url()),
        rel: "self".to_string(),
        mime_type: Some("application/atom+xml".to_string()),
        hreflang: None,
        title: None,
        length: None,
    };
    feed.set_links(vec![feed_link]);
    
    // Set feed ID (usually the website URL)
    feed.set_id(config.absolute_url());
    
    // Set updated time to the most recent content
    if let Some(latest_content) = content.first() {
        if let Some(date) = latest_content.meta().get_date() {
            if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(date) {
                feed.set_updated(parsed_date.with_timezone(&chrono::Utc));
            }
        }
    }
    
    // Create entries from content
    let mut entries = Vec::new();
    
    for content_item in content.iter().take(20) { // Limit to 20 most recent items
        let mut entry = Entry::default();
        
        // Set entry title
        entry.set_title(content_item.meta().get_title());
        
        // Set entry ID and link
        let content_url = format!("{}/{}", config.absolute_url(), content_item.slug());
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
        let summary = content_item.meta().get_description();
        if !summary.is_empty() {
            entry.set_summary(Text::html(summary));
        }
        
        // Set entry author
        if let Some(author) = content_item.meta().get_author() {
            entry.set_authors(vec![Person {
                name: author.to_string(),
                email: None,
                uri: None,
            }]);
        }
        
        // Set entry published/updated dates
        if let Some(date) = content_item.meta().get_date() {
            if let Ok(parsed_date) = chrono::DateTime::parse_from_rfc3339(date) {
                let utc_date = parsed_date.with_timezone(&chrono::Utc);
                entry.set_published(Some(utc_date.into()));
                entry.set_updated(utc_date);
            }
        }
        
        // Set entry categories/tags
        let categories: Vec<_> = content_item.meta().tags.iter()
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
