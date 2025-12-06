# rssreader 📡

a lightweight rss/atom feed parser for rust.

## install

```bash
cargo install --path .
```

## library usage

```rust
use rssreader::{from_url, from_file, FeedReader, FeedSource};
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    let feed = from_url("https://blog.rust-lang.org/feed.xml")?;
    
    println!("feed: {:?}", feed.title);
    println!("entries: {}", feed.entry_count);
    
    for entry in feed.take_entries(5) {
        println!("{}", entry.get_title());
        println!("  link: {:?}", entry.get_link());
        println!("  ago: {:?}", entry.published_ago());
    }
    
    Ok(())
}
```

### advanced usage

```rust
use rssreader::{FeedReader, FeedSource};
use std::time::Duration;
use std::path::Path;

let reader = FeedReader::new()
    .with_timeout(Duration::from_secs(60))
    .with_user_agent("my-app/1.0");

let feed = reader.fetch(FeedSource::Url("https://example.com/feed.rss"))?;

let filtered = feed.filter_by_title("rust");
let by_category = feed.filter_by_category("tech");

if let Some(latest) = feed.latest() {
    println!("latest: {}", latest.get_title());
}
```

### reading from different sources

```rust
use rssreader::{from_url, from_file, from_str};

let feed1 = from_url("https://example.com/feed.rss")?;

let feed2 = from_file("./my-feed.rss")?;

let raw_xml = "<rss>...</rss>";
let feed3 = from_str(raw_xml)?;
```

### filtering entries

```rust
use chrono::{Utc, Duration};

let recent = feed.filter_after(Utc::now() - Duration::days(7));

let old = feed.filter_before(Utc::now() - Duration::days(30));

let rust_posts = feed.filter_by_title("rust");

let tech = feed.filter_by_category("technology");
```

### entry helpers

```rust
for entry in feed.entries_iter() {
    println!("title: {}", entry.get_title());
    
    if let Some(content) = entry.get_clean_content() {
        println!("content: {}", content);
    }
    
    if let Some(ago) = entry.published_ago() {
        println!("published: {}", ago);
    }
    
    for author in &entry.authors {
        println!("author: {}", author);
    }
    
    for cat in &entry.categories {
        println!("category: {}", cat);
    }
}
```

## cli usage

```bash
rssreader --source https://example.com/feed.rss

rssreader --source feed.rss

rssreader --source feed.rss --full

rssreader --source feed.rss --limit 5

rssreader --source feed.rss --query "rust"

rssreader --source feed.rss --category "tech"
```

## example output

```
$ rssreader -s "https://feeds.simplecast.com/qm_9xx0g" -l 5

📡 rssreader v0.1.0
──────────────────────────────────────────────────

fetching: https://feeds.simplecast.com/qm_9xx0g

feed: Crime Junkie
desc: Does hearing about a true crime case always leave you scouring the internet for ...
link: https://feeds.simplecast.com/qm_9xx0g

total entries: 475

──────────────────────────────────────────────────
[1] MURDERED: Patrick Shunn & Monique Patenaude
    → https://crimejunkiepodcast.com/
    ⏰ 5d ago
    👤 author

[2] MISSING: Zelig Williams
    → https://crimejunkiepodcast.com/
    ⏰ 9d ago
    👤 author

[3] MURDERED: Linda Sherman
    → https://crimejunkiepodcast.com/
    ⏰ 12d ago
    👤 author

[4] INFAMOUS: The Murder of Amber Spradlin
    → https://crimejunkiepodcast.com/
    ⏰ 19d ago
    👤 author

[5] MURDERED: Teresa Flores & Martha Mezo
    → https://crimejunkiepodcast.com/
    ⏰ 26d ago
    👤 author

──────────────────────────────────────────────────
showed 5 of 475 entries
```

## cli flags

| flag | short | description |
|------|-------|-------------|
| `--source` | `-s` | path to .rss file or url to fetch |
| `--full` | `-f` | show full content instead of just titles |
| `--limit` | `-l` | limit number of entries to show (default: 10) |
| `--query` | `-q` | filter entries by title |
| `--category` | `-c` | filter entries by category |

## api

### FeedReader

| method | description |
|--------|-------------|
| `new()` | creates a new reader with defaults |
| `with_timeout(Duration)` | sets http timeout |
| `with_user_agent(String)` | sets user agent |
| `fetch(FeedSource)` | fetches and parses a feed |
| `fetch_url(&str)` | fetches raw content from url |
| `read_file(&Path)` | reads raw content from file |
| `parse(&str)` | parses raw content |

### ParsedFeed

| method | description |
|--------|-------------|
| `entries_iter()` | iterates over entries |
| `take_entries(n)` | takes first n entries |
| `filter_by_title(&str)` | filters by title substring |
| `filter_by_category(&str)` | filters by category |
| `filter_after(DateTime)` | filters entries after date |
| `filter_before(DateTime)` | filters entries before date |
| `latest()` | gets most recent entry |
| `oldest()` | gets oldest entry |
| `is_empty()` | checks if feed has no entries |

### FeedEntry

| method | description |
|--------|-------------|
| `get_title()` | gets title or "untitled" |
| `get_link()` | gets entry link |
| `get_content()` | gets content or summary |
| `get_clean_content()` | gets html-stripped content |
| `has_content()` | checks if has content |
| `published_ago()` | gets human readable time ago |

## why rss?

- no tracking
- no algorithms deciding what you see
- just pure chronological content
- works offline with local files
- your grandpa was onto something

## license

mit - do whatever you want with it

---

made with 🦀 by [ege](https://github.com/ege0x77czz)
