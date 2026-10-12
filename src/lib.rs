//! `the_bible`: KJV library — reference lookup + full-text search.
//!
//! ```rust
//! use the_bible::{Bible, SearchOptions};
//!
//! let bible = Bible::embedded().unwrap();
//! let hits = bible.lookup("John 3:16").unwrap();
//! assert!(hits[0].text.starts_with("For God so loved"));
//!
//! let results = bible.search("faith hope", SearchOptions::default());
//! assert!(!results.is_empty());
//! ```

pub mod books;
pub mod error;
pub mod reference;
pub mod search;

pub use books::BOOKS;
pub use error::BibleError;
pub use reference::Reference;
pub use search::{SearchHit, SearchMode, SearchOptions};

use search::{make_snippet, tokenize, SearchIndex};
use std::collections::HashMap;
use std::path::Path;

/// One verse with canonical address.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verse {
    /// 0-based index into [`BOOKS`].
    pub book: u8,
    /// 1-based chapter.
    pub chapter: u32,
    /// 1-based verse.
    pub verse: u32,
    pub text: String,
}

impl Verse {
    pub fn book_name(&self) -> &'static str {
        BOOKS[self.book as usize]
    }

    pub fn reference(&self) -> String {
        format!("{} {}:{}", self.book_name(), self.chapter, self.verse)
    }
}

impl std::fmt::Display for Verse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} — {}", self.reference(), self.text)
    }
}

/// The whole Bible: flat verse store + lookup map + search index.
#[derive(Debug)]
pub struct Bible {
    verses: Vec<Verse>,
    lookup: HashMap<(u8, u32, u32), usize>,
    index: SearchIndex,
}

impl Bible {
    /// Load from a JSON string shaped `[[["verse", ...]]]`
    /// (`[book][chapter][verse]`, all 1-based in address, 0-based in JSON).
    pub fn from_json_str(json: &str) -> Result<Self, BibleError> {
        let data: Vec<Vec<Vec<String>>> =
            serde_json::from_str(json).map_err(|e| BibleError::Load(e.to_string()))?;
        Self::from_3d_array(data)
    }

    /// Load from a `data/kjv.json` path.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, BibleError> {
        let raw =
            std::fs::read_to_string(path.as_ref()).map_err(|e| BibleError::Load(e.to_string()))?;
        Self::from_json_str(&raw)
    }

    /// Load the vendored KJV compiled into the binary (`data/kjv.json`).
    pub fn embedded() -> Result<Self, BibleError> {
        Self::from_json_str(include_str!("../data/kjv.json"))
    }

    fn from_3d_array(data: Vec<Vec<Vec<String>>>) -> Result<Self, BibleError> {
        if data.len() != BOOKS.len() {
            return Err(BibleError::Load(format!(
                "expected {} books, found {}",
                BOOKS.len(),
                data.len()
            )));
        }
        let mut verses = Vec::new();
        let mut lookup = HashMap::new();
        for (b, chapters) in data.into_iter().enumerate() {
            for (c, chapter) in chapters.into_iter().enumerate() {
                for (v, text) in chapter.into_iter().enumerate() {
                    if text.trim().is_empty() {
                        continue;
                    }
                    let key = (b as u8, (c + 1) as u32, (v + 1) as u32);
                    lookup.insert(key, verses.len());
                    verses.push(Verse {
                        book: b as u8,
                        chapter: (c + 1) as u32,
                        verse: (v + 1) as u32,
                        text,
                    });
                }
            }
        }
        if verses.is_empty() {
            return Err(BibleError::Load("no verses found".to_string()));
        }
        let texts: Vec<String> = verses.iter().map(|v| v.text.clone()).collect();
        let index = SearchIndex::build(&texts);
        Ok(Self {
            verses,
            lookup,
            index,
        })
    }

    // ---- counts ----

    pub fn verse_count(&self) -> usize {
        self.verses.len()
    }

    pub fn book_count(&self) -> usize {
        BOOKS.len()
    }

    pub fn chapter_count(&self) -> usize {
        let mut n = 0;
        let mut prev: Option<(u8, u32)> = None;
        for v in &self.verses {
            let cur = (v.book, v.chapter);
            if prev != Some(cur) {
                n += 1;
                prev = Some(cur);
            }
        }
        n
    }

    pub fn indexed_terms(&self) -> usize {
        self.index.term_count()
    }

    // ---- lookup ----

    /// Single verse by numeric address.
    pub fn get(&self, book: u8, chapter: u32, verse: u32) -> Result<&Verse, BibleError> {
        self.lookup
            .get(&(book, chapter, verse))
            .map(|&i| &self.verses[i])
            .ok_or_else(|| {
                BibleError::NotFound(format!(
                    "{} {}:{}",
                    BOOKS.get(book as usize).unwrap_or(&"?"),
                    chapter,
                    verse
                ))
            })
    }

    /// Whole chapter, in verse order.
    pub fn get_chapter(&self, book: u8, chapter: u32) -> Result<Vec<&Verse>, BibleError> {
        let out: Vec<&Verse> = self
            .verses
            .iter()
            .filter(|v| v.book == book && v.chapter == chapter)
            .collect();
        if out.is_empty() {
            return Err(BibleError::NotFound(format!(
                "{} {}",
                BOOKS.get(book as usize).unwrap_or(&"?"),
                chapter
            )));
        }
        Ok(out)
    }

    /// Resolve a parsed [`Reference`] (single verse, range, or whole chapter).
    pub fn get_by_reference(&self, r: &Reference) -> Result<Vec<&Verse>, BibleError> {
        match (r.verse_start, r.verse_end) {
            (None, _) => self.get_chapter(r.book, r.chapter),
            (Some(s), Some(e)) if s != e => {
                let (lo, hi) = if s <= e { (s, e) } else { (e, s) };
                let mut out = Vec::new();
                for v in lo..=hi {
                    out.push(self.get(r.book, r.chapter, v)?);
                }
                Ok(out)
            }
            (Some(s), _) => Ok(vec![self.get(r.book, r.chapter, s)?]),
        }
    }

    /// Parse `John 3:16` and resolve. Most convenient for callers.
    pub fn lookup(&self, s: &str) -> Result<Vec<&Verse>, BibleError> {
        let r: Reference = s.parse()?;
        self.get_by_reference(&r)
    }

    /// A uniformly random verse. Powers the `cargo run` binary.
    pub fn random_verse(&self) -> &Verse {
        use rand::Rng as _;
        let i = rand::rng().random_range(0..self.verses.len());
        &self.verses[i]
    }

    // ---- search ----

    /// Full-text search. Returns at most `options.max_results` hits in canonical order.
    pub fn search(&self, query: &str, options: SearchOptions) -> Vec<SearchHit> {
        let terms = tokenize(query);
        if terms.is_empty() {
            return Vec::new();
        }
        self.index
            .query(query, options.mode, options.max_results)
            .into_iter()
            .map(|i| {
                let v = &self.verses[i];
                SearchHit {
                    verse_index: i,
                    reference: v.reference(),
                    snippet: make_snippet(&v.text, &terms),
                    text: v.text.clone(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bible() -> Bible {
        Bible::embedded().expect("vendored kjv.json must load")
    }

    #[test]
    fn loads_expected_shape() {
        let b = bible();
        assert_eq!(b.book_count(), 66);
        assert_eq!(b.chapter_count(), 1189);
        // KJV is 31,102; allow off-by-one across public-domain editions.
        assert!(
            (31_100..=31_110).contains(&b.verse_count()),
            "verse_count={}",
            b.verse_count()
        );
    }

    #[test]
    fn lookup_john_3_16() {
        let b = bible();
        let hits = b.lookup("John 3:16").unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].text.starts_with("For God so loved the world"));
    }

    #[test]
    fn lookup_range_and_chapter() {
        let b = bible();
        let range = b.lookup("Luke 23:39-43").unwrap();
        assert_eq!(range.len(), 5);
        let chap = b.lookup("Acts 2").unwrap();
        assert_eq!(chap.len(), 47);
    }

    #[test]
    fn random_verse_is_in_range_and_varies() {
        use std::collections::HashSet;
        let b = bible();
        let seen: HashSet<String> = (0..50).map(|_| b.random_verse().reference()).collect();
        assert!(seen.len() > 1, "50 draws should not all be the same verse");
    }

    #[test]
    fn search_finds_genesis() {
        let b = bible();
        let hits = b.search("In the beginning God created", SearchOptions::default());
        assert!(!hits.is_empty());
        assert_eq!(hits[0].reference, "Genesis 1:1");
    }
}
