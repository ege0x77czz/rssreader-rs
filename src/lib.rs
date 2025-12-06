use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::Path;
use std::time::Duration;

pub use feed_rs::model::{Entry, Feed, Link, Text};

#[derive(Debug, Clone)]
pub struct FeedReader {
    timeout: Duration,
    user_agent: String,
}

#[derive(Debug, Clone)]
pub struct FeedEntry {
    pub title: Option<String>,
    pub link: Option<String>,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub published: Option<DateTime<Utc>>,
    pub updated: Option<DateTime<Utc>>,
    pub authors: Vec<String>,
    pub categories: Vec<String>,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct ParsedFeed {
    pub title: Option<String>,
    pub description: Option<String>,
    pub link: Option<String>,
    pub updated: Option<DateTime<Utc>>,
    pub entries: Vec<FeedEntry>,
    pub entry_count: usize,
}

#[derive(Debug, Clone)]
pub enum FeedSource<'a> {
    Url(&'a str),
    File(&'a Path),
    Raw(&'a str),
}

impl Default for FeedReader {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedReader {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            user_agent: format!("rssreader/{}", env!("CARGO_PKG_VERSION")),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    pub fn fetch(&self, source: FeedSource) -> Result<ParsedFeed> {
        let raw = match source {
            FeedSource::Url(url) => self.fetch_url(url)?,
            FeedSource::File(path) => self.read_file(path)?,
            FeedSource::Raw(content) => content.to_string(),
        };

        self.parse(&raw)
    }

    pub fn fetch_url(&self, url: &str) -> Result<String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(self.timeout)
            .user_agent(&self.user_agent)
            .build()
            .context("failed to create http client")?;

        let response = client
            .get(url)
            .send()
            .context("failed to fetch url")?;

        if !response.status().is_success() {
            anyhow::bail!("http error: {}", response.status());
        }

        response.text().context("failed to read response body")
    }

    pub fn read_file(&self, path: &Path) -> Result<String> {
        fs::read_to_string(path)
            .context(format!("failed to read file: {}", path.display()))
    }

    pub fn parse(&self, content: &str) -> Result<ParsedFeed> {
        let feed = feed_rs::parser::parse(content.as_bytes())
            .context("failed to parse feed")?;

        Ok(self.convert_feed(feed))
    }

    fn convert_feed(&self, feed: Feed) -> ParsedFeed {
        let entries: Vec<FeedEntry> = feed
            .entries
            .into_iter()
            .map(|e| self.convert_entry(e))
            .collect();

        let entry_count = entries.len();

        ParsedFeed {
            title: feed.title.map(|t| t.content),
            description: feed.description.map(|d| d.content),
            link: feed.links.first().map(|l| l.href.clone()),
            updated: feed.updated,
            entries,
            entry_count,
        }
    }

    fn convert_entry(&self, entry: Entry) -> FeedEntry {
        FeedEntry {
            title: entry.title.map(|t| t.content),
            link: entry.links.first().map(|l| l.href.clone()),
            summary: entry.summary.map(|s| s.content),
            content: entry.content.and_then(|c| c.body),
            published: entry.published,
            updated: entry.updated,
            authors: entry.authors.into_iter().map(|a| a.name).collect(),
            categories: entry.categories.into_iter().map(|c| c.term).collect(),
            id: entry.id,
        }
    }
}

impl ParsedFeed {
    pub fn entries_iter(&self) -> impl Iterator<Item = &FeedEntry> {
        self.entries.iter()
    }

    pub fn take_entries(&self, count: usize) -> Vec<&FeedEntry> {
        self.entries.iter().take(count).collect()
    }

    pub fn filter_by_title(&self, query: &str) -> Vec<&FeedEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| {
                e.title
                    .as_ref()
                    .map(|t| t.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
            })
            .collect()
    }

    pub fn filter_by_category(&self, category: &str) -> Vec<&FeedEntry> {
        let cat_lower = category.to_lowercase();
        self.entries
            .iter()
            .filter(|e| {
                e.categories
                    .iter()
                    .any(|c| c.to_lowercase().contains(&cat_lower))
            })
            .collect()
    }

    pub fn filter_after(&self, date: DateTime<Utc>) -> Vec<&FeedEntry> {
        self.entries
            .iter()
            .filter(|e| e.published.map(|p| p > date).unwrap_or(false))
            .collect()
    }

    pub fn filter_before(&self, date: DateTime<Utc>) -> Vec<&FeedEntry> {
        self.entries
            .iter()
            .filter(|e| e.published.map(|p| p < date).unwrap_or(false))
            .collect()
    }

    pub fn latest(&self) -> Option<&FeedEntry> {
        self.entries.first()
    }

    pub fn oldest(&self) -> Option<&FeedEntry> {
        self.entries.last()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl FeedEntry {
    pub fn has_content(&self) -> bool {
        self.content.is_some() || self.summary.is_some()
    }

    pub fn get_content(&self) -> Option<&str> {
        self.content.as_deref().or(self.summary.as_deref())
    }

    pub fn get_clean_content(&self) -> Option<String> {
        self.get_content().map(strip_html)
    }

    pub fn get_link(&self) -> Option<&str> {
        self.link.as_deref()
    }

    pub fn get_title(&self) -> &str {
        self.title.as_deref().unwrap_or("untitled")
    }

    pub fn published_ago(&self) -> Option<String> {
        self.published.map(|p| {
            let now = Utc::now();
            let diff = now.signed_duration_since(p);

            if diff.num_days() > 0 {
                format!("{}d ago", diff.num_days())
            } else if diff.num_hours() > 0 {
                format!("{}h ago", diff.num_hours())
            } else if diff.num_minutes() > 0 {
                format!("{}m ago", diff.num_minutes())
            } else {
                "just now".to_string()
            }
        })
    }
}

pub fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }

    result
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn from_url(url: &str) -> Result<ParsedFeed> {
    FeedReader::new().fetch(FeedSource::Url(url))
}

pub fn from_file(path: impl AsRef<Path>) -> Result<ParsedFeed> {
    FeedReader::new().fetch(FeedSource::File(path.as_ref()))
}

pub fn from_str(content: &str) -> Result<ParsedFeed> {
    FeedReader::new().fetch(FeedSource::Raw(content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html() {
        let html = "<p>hello <b>world</b></p>";
        assert_eq!(strip_html(html), "hello world");
    }

    #[test]
    fn test_reader_builder() {
        let reader = FeedReader::new()
            .with_timeout(Duration::from_secs(60))
            .with_user_agent("custom-agent/1.0");

        assert_eq!(reader.timeout, Duration::from_secs(60));
        assert_eq!(reader.user_agent, "custom-agent/1.0");
    }
}

