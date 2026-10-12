//! Canonical book names, order, and alias resolution.

/// 66-book Protestant canon in order. Index = `book_index` used everywhere.
pub const BOOKS: [&str; 66] = [
    "Genesis",
    "Exodus",
    "Leviticus",
    "Numbers",
    "Deuteronomy",
    "Joshua",
    "Judges",
    "Ruth",
    "1 Samuel",
    "2 Samuel",
    "1 Kings",
    "2 Kings",
    "1 Chronicles",
    "2 Chronicles",
    "Ezra",
    "Nehemiah",
    "Esther",
    "Job",
    "Psalms",
    "Proverbs",
    "Ecclesiastes",
    "Song of Solomon",
    "Isaiah",
    "Jeremiah",
    "Lamentations",
    "Ezekiel",
    "Daniel",
    "Hosea",
    "Joel",
    "Amos",
    "Obadiah",
    "Jonah",
    "Micah",
    "Nahum",
    "Habakkuk",
    "Zephaniah",
    "Haggai",
    "Zechariah",
    "Malachi",
    "Matthew",
    "Mark",
    "Luke",
    "John",
    "Acts",
    "Romans",
    "1 Corinthians",
    "2 Corinthians",
    "Galatians",
    "Ephesians",
    "Philippians",
    "Colossians",
    "1 Thessalonians",
    "2 Thessalonians",
    "1 Timothy",
    "2 Timothy",
    "Titus",
    "Philemon",
    "Hebrews",
    "James",
    "1 Peter",
    "2 Peter",
    "1 John",
    "2 John",
    "3 John",
    "Jude",
    "Revelation",
];

/// Normalize user input for comparison: lowercase, strip periods, collapse whitespace.
pub fn normalize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = true; // trim leading
    for c in s.chars() {
        if c == '.' {
            continue;
        }
        if c.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(c.to_ascii_lowercase());
            prev_space = false;
        }
    }
    while out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Resolve a book name / abbreviation to its 0-based index.
///
/// Matches longest alias first so `1 john` wins over `john`.
pub fn resolve_book(name: &str) -> Option<u8> {
    let n = normalize(name);
    let nospace: String = n.chars().filter(|c| *c != ' ').collect();
    // Exact canonical match first (fast path).
    for (i, b) in BOOKS.iter().enumerate() {
        let c = normalize(b);
        if c == n {
            return Some(i as u8);
        }
        if c.chars().filter(|x| *x != ' ').collect::<String>() == nospace {
            return Some(i as u8);
        }
    }
    // Abbreviations, longest first.
    for (alias, idx) in aliases_sorted() {
        if n == *alias {
            return Some(*idx);
        }
        if alias.chars().filter(|c| *c != ' ').collect::<String>() == nospace {
            return Some(*idx);
        }
    }
    None
}

/// Split `John 3:16`-style input into `(book_index, rest)` by longest-prefix match.
pub fn split_book_prefix(input: &str) -> Option<(u8, &str)> {
    // Try every known name (canonical + alias) longest-first, case-insensitive.
    // We work on the raw string but compare normalized prefixes carefully:
    // simplest robust approach — try all prefix lengths is overkill;
    // instead try each known alias as a prefix.
    let mut best: Option<(u8, usize)> = None;
    let mut candidates: Vec<(&str, u8)> = Vec::with_capacity(220);
    for (i, b) in BOOKS.iter().enumerate() {
        candidates.push((*b, i as u8));
    }
    for (alias, idx) in aliases_sorted() {
        candidates.push((alias, *idx));
    }
    // Sort longest first so "1 john" beats "john".
    candidates.sort_by_key(|(a, _)| std::cmp::Reverse(a.len()));

    for (alias, idx) in candidates {
        if input.len() < alias.len() {
            continue;
        }
        let (head, tail) = input.split_at(alias.len());
        if !head.eq_ignore_ascii_case(alias) {
            continue;
        }
        // Next char must be end or whitespace (so "romans" doesn't match "rom").
        if !tail.is_empty() && !tail.starts_with([' ', '\t']) {
            continue;
        }
        match best {
            Some((_, len)) if len >= alias.len() => {}
            _ => best = Some((idx, alias.len())),
        }
    }
    best.map(|(idx, len)| (idx, input[len..].trim_start()))
}

fn aliases_sorted() -> &'static [(&'static str, u8)] {
    use std::sync::OnceLock;
    static CACHE: OnceLock<Vec<(&'static str, u8)>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let mut v: Vec<(&'static str, u8)> = aliases().to_vec();
        v.sort_by_key(|(a, _)| std::cmp::Reverse(a.len()));
        v
    })
}

// (alias, book_index). Keep lowercase, no periods.
fn aliases() -> &'static [(&'static str, u8)] {
    &[
        ("gen", 0),
        ("ge", 0),
        ("genesis", 0),
        ("exo", 1),
        ("exod", 1),
        ("exodus", 1),
        ("lev", 2),
        ("leviticus", 2),
        ("num", 3),
        ("numbers", 3),
        ("deut", 4),
        ("deuteronomy", 4),
        ("josh", 5),
        ("joshua", 5),
        ("judg", 6),
        ("judges", 6),
        ("jdg", 6),
        ("ruth", 7),
        ("1 sam", 8),
        ("1sam", 8),
        ("1st samuel", 8),
        ("2 sam", 9),
        ("2sam", 9),
        ("2nd samuel", 9),
        ("1 kings", 10),
        ("1 kgs", 10),
        ("1kgs", 10),
        ("2 kings", 11),
        ("2 kgs", 11),
        ("2kgs", 11),
        ("1 chron", 12),
        ("1 chr", 12),
        ("1chr", 12),
        ("1 chronicles", 12),
        ("2 chron", 13),
        ("2 chr", 13),
        ("2chr", 13),
        ("2 chronicles", 13),
        ("ezra", 14),
        ("neh", 15),
        ("nehemiah", 15),
        ("esth", 16),
        ("esther", 16),
        ("job", 17),
        ("ps", 18),
        ("psa", 18),
        ("psalm", 18),
        ("psalms", 18),
        ("prov", 19),
        ("proverbs", 19),
        ("eccl", 20),
        ("ecclesiastes", 20),
        ("song", 21),
        ("sos", 21),
        ("song of solomon", 21),
        ("song of songs", 21),
        ("isa", 22),
        ("isaiah", 22),
        ("jer", 23),
        ("jeremiah", 23),
        ("lam", 24),
        ("lamentations", 24),
        ("ezek", 25),
        ("ezekiel", 25),
        ("dan", 26),
        ("daniel", 26),
        ("hos", 27),
        ("hosea", 27),
        ("joel", 28),
        ("amos", 29),
        ("obad", 30),
        ("obadiah", 30),
        ("jonah", 31),
        ("mic", 32),
        ("micah", 32),
        ("nah", 33),
        ("nahum", 33),
        ("hab", 34),
        ("habakkuk", 34),
        ("zeph", 35),
        ("zephaniah", 35),
        ("hag", 36),
        ("haggai", 36),
        ("zech", 37),
        ("zechariah", 37),
        ("mal", 38),
        ("malachi", 38),
        ("matt", 39),
        ("matthew", 39),
        ("mark", 40),
        ("mk", 40),
        ("luke", 41),
        ("lk", 41),
        ("john", 42),
        ("jn", 42),
        ("acts", 43),
        ("rom", 44),
        ("romans", 44),
        ("1 cor", 45),
        ("1cor", 45),
        ("1 corinthians", 45),
        ("2 cor", 46),
        ("2cor", 46),
        ("2 corinthians", 46),
        ("gal", 47),
        ("galatians", 47),
        ("eph", 48),
        ("ephesians", 48),
        ("phil", 49),
        ("philippians", 49),
        ("col", 50),
        ("colossians", 50),
        ("1 thess", 51),
        ("1thess", 51),
        ("1 thessalonians", 51),
        ("2 thess", 52),
        ("2thess", 52),
        ("2 thessalonians", 52),
        ("1 tim", 53),
        ("1tim", 53),
        ("1 timothy", 53),
        ("2 tim", 54),
        ("2tim", 54),
        ("2 timothy", 54),
        ("titus", 55),
        ("tit", 55),
        ("philem", 56),
        ("philemon", 56),
        ("heb", 57),
        ("hebrews", 57),
        ("james", 58),
        ("jas", 58),
        ("1 pet", 59),
        ("1pet", 59),
        ("1 peter", 59),
        ("2 pet", 60),
        ("2pet", 60),
        ("2 peter", 60),
        ("1 john", 61),
        ("1 jn", 61),
        ("1jn", 61),
        ("2 john", 62),
        ("2 jn", 62),
        ("2jn", 62),
        ("3 john", 63),
        ("3 jn", 63),
        ("3jn", 63),
        ("jude", 64),
        ("rev", 65),
        ("revelation", 65),
        // Bare numbers that still need a chapter suffix are NOT aliases;
        // they are handled by prefix matching only when followed by numbers.
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_resolve() {
        assert_eq!(resolve_book("Genesis"), Some(0));
        assert_eq!(resolve_book("revelation"), Some(65));
        assert_eq!(resolve_book("1 John"), Some(61));
    }

    #[test]
    fn alias_resolve() {
        assert_eq!(resolve_book("Gen"), Some(0));
        assert_eq!(resolve_book("ps"), Some(18));
        assert_eq!(resolve_book("jn"), Some(42));
        assert_eq!(resolve_book("1jn"), Some(61));
    }

    #[test]
    fn prefix_split_prefers_longest() {
        let (idx, rest) = split_book_prefix("1 John 3:16").unwrap();
        assert_eq!(idx, 61);
        assert_eq!(rest, "3:16");
        let (idx, rest) = split_book_prefix("John 3:16").unwrap();
        assert_eq!(idx, 42);
        assert_eq!(rest, "3:16");
    }
}
