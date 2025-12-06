use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use rssreader::{from_file, from_url, FeedEntry, ParsedFeed};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author = "ege <https://github.com/ege0x77czz>")]
#[command(version, about = "a chill rss feed reader for the terminal", long_about = None)]
struct Args {
    #[arg(short, long)]
    source: String,

    #[arg(short, long, default_value_t = false)]
    full: bool,

    #[arg(short, long, default_value_t = 10)]
    limit: usize,

    #[arg(short, long)]
    query: Option<String>,

    #[arg(short, long)]
    category: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    print_header();

    let feed = fetch_feed(&args.source)?;
    print_feed_info(&feed);

    let entries = apply_filters(&feed, &args);
    print_entries(&entries, args.full, args.limit);

    print_footer(entries.len(), feed.entry_count);

    Ok(())
}

fn fetch_feed(source: &str) -> Result<ParsedFeed> {
    if source.starts_with("http://") || source.starts_with("https://") {
        println!("{} {}\n", "fetching:".blue().bold(), source);
        from_url(source)
    } else {
        let path = PathBuf::from(source);
        println!("{} {}\n", "reading:".blue().bold(), path.display());
        from_file(&path)
    }
}

fn apply_filters<'a>(feed: &'a ParsedFeed, args: &Args) -> Vec<&'a FeedEntry> {
    if let Some(ref query) = args.query {
        return feed.filter_by_title(query);
    }

    if let Some(ref category) = args.category {
        return feed.filter_by_category(category);
    }

    feed.entries_iter().collect()
}

fn print_header() {
    println!("{}", "\n📡 rssreader v0.1.0".cyan().bold());
    println!("{}\n", "─".repeat(50).dimmed());
}

fn print_feed_info(feed: &ParsedFeed) {
    if let Some(ref title) = feed.title {
        println!("{} {}", "feed:".green().bold(), title);
    }
    if let Some(ref desc) = feed.description {
        let short_desc = if desc.len() > 80 {
            format!("{}...", &desc[..80])
        } else {
            desc.clone()
        };
        println!("{} {}", "desc:".green().bold(), short_desc);
    }
    if let Some(ref link) = feed.link {
        println!("{} {}", "link:".green().bold(), link);
    }
    println!(
        "\n{} {}\n",
        "total entries:".yellow().bold(),
        feed.entry_count
    );
    println!("{}", "─".repeat(50).dimmed());
}

fn print_entries(entries: &[&FeedEntry], full: bool, limit: usize) {
    for (i, entry) in entries.iter().take(limit).enumerate() {
        let num = format!("[{}]", i + 1).cyan();
        println!("{} {}", num, entry.get_title().white().bold());

        if let Some(link) = entry.get_link() {
            println!("    {} {}", "→".dimmed(), link.dimmed());
        }

        if let Some(ago) = entry.published_ago() {
            println!("    {} {}", "⏰".dimmed(), ago.dimmed());
        }

        if !entry.categories.is_empty() {
            let cats = entry.categories.join(", ");
            println!("    {} {}", "📁".dimmed(), cats.dimmed());
        }

        if !entry.authors.is_empty() {
            let authors = entry.authors.join(", ");
            println!("    {} {}", "👤".dimmed(), authors.dimmed());
        }

        if full {
            if let Some(content) = entry.get_clean_content() {
                let preview = if content.len() > 300 {
                    format!("{}...", &content[..300])
                } else {
                    content
                };
                println!("\n    {}\n", preview.dimmed());
            }
        }

        println!();
    }
}

fn print_footer(shown: usize, total: usize) {
    println!("{}", "─".repeat(50).dimmed());
    println!(
        "{}",
        format!("showed {} of {} entries", shown.min(total), total).dimmed()
    );
}
