//! `cargo run --example lookup -- "John 3:16"`
//! `cargo run --example lookup -- --search "faith hope"`

use the_bible::{Bible, SearchMode, SearchOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bible = Bible::embedded()?;
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("usage:");
        eprintln!("  lookup \"John 3:16\"");
        eprintln!("  lookup --search \"faith hope\" [--any] [--max 10]");
        std::process::exit(2);
    }

    if args[0] == "--search" {
        let query = args.get(1).cloned().unwrap_or_default();
        let mode = if args.iter().any(|a| a == "--any") {
            SearchMode::Any
        } else {
            SearchMode::All
        };
        let max = args
            .windows(2)
            .find(|w| w[0] == "--max")
            .and_then(|w| w[1].parse().ok())
            .unwrap_or(10);
        for hit in bible.search(
            &query,
            SearchOptions {
                mode,
                max_results: max,
            },
        ) {
            println!("{} — {}", hit.reference, hit.snippet);
        }
        return Ok(());
    }

    let reference = args.join(" ");
    match bible.lookup(&reference) {
        Ok(verses) => {
            for v in verses {
                println!("{v}");
            }
            Ok(())
        }
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
