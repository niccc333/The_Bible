//! `Reference` parsing: "John 3:16", "Gen 1:1-3", "Acts 2", ...

use crate::books::{split_book_prefix, BOOKS};
use crate::error::BibleError;
use std::fmt;
use std::str::FromStr;

/// A parsed Bible reference.
///
/// - `verse_start == None` → whole chapter (e.g. `Acts 2`)
/// - `verse_end == None` or equal to start → single verse
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reference {
    pub book: u8,
    pub chapter: u32,
    pub verse_start: Option<u32>,
    pub verse_end: Option<u32>,
}

impl Reference {
    pub fn single(book: u8, chapter: u32, verse: u32) -> Self {
        Self {
            book,
            chapter,
            verse_start: Some(verse),
            verse_end: Some(verse),
        }
    }

    pub fn chapter(book: u8, chapter: u32) -> Self {
        Self {
            book,
            chapter,
            verse_start: None,
            verse_end: None,
        }
    }

    pub fn range(book: u8, chapter: u32, start: u32, end: u32) -> Self {
        let (lo, hi) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        Self {
            book,
            chapter,
            verse_start: Some(lo),
            verse_end: Some(hi),
        }
    }

    pub fn book_name(&self) -> &'static str {
        BOOKS[self.book as usize]
    }

    pub fn is_chapter(&self) -> bool {
        self.verse_start.is_none()
    }
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.verse_start, self.verse_end) {
            (None, _) => write!(f, "{} {}", self.book_name(), self.chapter),
            (Some(s), Some(e)) if s != e => {
                write!(f, "{} {}:{}-{}", self.book_name(), self.chapter, s, e)
            }
            (Some(s), _) => write!(f, "{} {}:{}", self.book_name(), self.chapter, s),
        }
    }
}

impl FromStr for Reference {
    type Err = BibleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(BibleError::InvalidReference(s.to_string()));
        }
        let (book, rest) =
            split_book_prefix(s).ok_or_else(|| BibleError::UnknownBook(s.to_string()))?;
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(BibleError::InvalidReference(s.to_string()));
        }
        // Whole chapter: "2" — digits only, no colon.
        if !rest.contains(':') {
            let chapter: u32 = rest
                .parse()
                .map_err(|_| BibleError::InvalidReference(s.to_string()))?;
            if chapter == 0 {
                return Err(BibleError::InvalidReference(s.to_string()));
            }
            return Ok(Reference::chapter(book, chapter));
        }
        // Chapter:verse[-verse]
        let (chap_str, verse_str) = rest
            .split_once(':')
            .ok_or_else(|| BibleError::InvalidReference(s.to_string()))?;
        let chapter: u32 = chap_str
            .trim()
            .parse()
            .map_err(|_| BibleError::InvalidReference(s.to_string()))?;
        if chapter == 0 {
            return Err(BibleError::InvalidReference(s.to_string()));
        }
        let verse_str = verse_str.trim();
        if verse_str.is_empty() {
            return Err(BibleError::InvalidReference(s.to_string()));
        }
        if let Some((a, b)) = verse_str.split_once('-') {
            let start: u32 = a
                .trim()
                .parse()
                .map_err(|_| BibleError::InvalidReference(s.to_string()))?;
            let end: u32 = b
                .trim()
                .parse()
                .map_err(|_| BibleError::InvalidReference(s.to_string()))?;
            if start == 0 || end == 0 {
                return Err(BibleError::InvalidReference(s.to_string()));
            }
            return Ok(Reference::range(book, chapter, start, end));
        }
        let verse: u32 = verse_str
            .parse()
            .map_err(|_| BibleError::InvalidReference(s.to_string()))?;
        if verse == 0 {
            return Err(BibleError::InvalidReference(s.to_string()));
        }
        Ok(Reference::single(book, chapter, verse))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(s: &str) -> Reference {
        s.parse().unwrap()
    }

    #[test]
    fn single_verse() {
        let x = r("John 3:16");
        assert_eq!(x.book, 42);
        assert_eq!(x.chapter, 3);
        assert_eq!(x.verse_start, Some(16));
        assert_eq!(x.to_string(), "John 3:16");
    }

    #[test]
    fn numbered_book() {
        let x = r("1 John 1:9");
        assert_eq!(x.book, 61);
        assert_eq!(x.to_string(), "1 John 1:9");
    }

    #[test]
    fn abbreviations() {
        assert_eq!(r("Gen 1:1").book, 0);
        assert_eq!(r("ps 23:1").book, 18);
        assert_eq!(r("jn 3:16").book, 42);
    }

    #[test]
    fn whole_chapter() {
        let x = r("Acts 2");
        assert!(x.is_chapter());
        assert_eq!(x.to_string(), "Acts 2");
    }

    #[test]
    fn verse_range() {
        let x = r("Luke 23:39-43");
        assert_eq!(x.verse_start, Some(39));
        assert_eq!(x.verse_end, Some(43));
        assert_eq!(x.to_string(), "Luke 23:39-43");
    }

    #[test]
    fn rejects_garbage() {
        assert!("".parse::<Reference>().is_err());
        assert!("John".parse::<Reference>().is_err());
        assert!("Nope 3:16".parse::<Reference>().is_err());
        assert!("John 0:1".parse::<Reference>().is_err());
    }
}
