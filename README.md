# The_Bible — KJV in Rust

Get a random verse:

```sh
cargo run
# Judges 6:36 — And Gideon said unto God, If thou wilt save Israel by mine hand, as thou hast said,
```

Library crate: reference lookup + full-text search over the King James Version.

```rust
use the_bible::{Bible, SearchOptions};

let bible = Bible::embedded().unwrap();

// Lookup: single verse, range, or whole chapter
let v = bible.lookup("John 3:16").unwrap();
println!("{v:?}"); // For God so loved the world...

let passage = bible.lookup("Luke 23:39-43").unwrap(); // 5 verses
let chapter = bible.lookup("Acts 2").unwrap();        // 47 verses

// Search (AND by default)
let hits = bible.search("faith hope", SearchOptions::default());
for h in &hits {
    println!("{} — {}", h.reference, h.snippet);
}
```

## CLI example

```sh
cargo run --example lookup -- "John 3:16"
cargo run --example lookup -- --search "faith hope" --max 5
cargo run --example lookup -- --search "faith hope" --any
```

## Data

`data/kjv.json` — KJV text as `[book][chapter][verse]` 3D array (66 books,
1189 chapters, ~31,102 verses), vendored from
[`churchstudio-org/openbible`](https://github.com/churchstudio-org/openbible)
`KJV/bible.json`. KJV text is public domain worldwide; the OpenBible
container project is MIT. Embedded at compile time via `include_str!`, so
`Bible::embedded()` needs no runtime files. Use `Bible::from_path()` for
custom files of the same shape.

## API

- `Bible::embedded() / from_path() / from_json_str()` — load
- `bible.lookup("Gen 1:1-3")` — parse + resolve (`Reference` also implements `FromStr`)
- `bible.get(book, chapter, verse)`, `get_chapter()`, `get_by_reference()` — typed access
- `bible.search(query, SearchOptions { mode: All | Any, max_results })` — inverted-index search with snippets

Reference parser handles `1 John` vs `John`, abbreviations (`Gen`, `Ps`, `Jn`,
`1jn`), whole chapters (`Acts 2`), and ranges (`John 3:16-18`).

## Dev

```sh
cargo test
cargo clippy --all-targets
```

GNU toolchain note (Windows): this repo builds with
`stable-x86_64-pc-windows-gnu` since no MSVC linker is installed.
