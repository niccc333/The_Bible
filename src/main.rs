//! `cargo run` — print a random KJV verse.
//!
//! ```sh
//! cargo run
//! # Romans 5:2 — By whom also we have access by faith...
//! ```

use the_bible::Bible;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bible = Bible::embedded()?;
    println!("{}", bible.random_verse());
    Ok(())
}
