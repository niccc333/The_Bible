//! Full-text search: tokenize → inverted index → AND/OR query.

use std::collections::{HashMap, HashSet};

/// AND (`All`) requires every query term; `Any` requires at least one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchMode {
    #[default]
    All,
    Any,
}

/// Options for [`crate::Bible::search`].
#[derive(Debug, Clone, Copy)]
pub struct SearchOptions {
    pub mode: SearchMode,
    pub max_results: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            mode: SearchMode::All,
            max_results: 20,
        }
    }
}

/// One search hit with a display-ready snippet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchHit {
    pub verse_index: usize,
    pub reference: String,
    pub text: String,
    pub snippet: String,
}

/// Lowercase alphanumeric tokenization. `God's` → ["god", "s"].
pub fn tokenize(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect()
}

/// Verse-index → postings. Built once at load.
#[derive(Debug, Default)]
pub struct SearchIndex {
    postings: HashMap<String, Vec<usize>>,
}

impl SearchIndex {
    pub fn build(texts: &[String]) -> Self {
        let mut postings: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, text) in texts.iter().enumerate() {
            let seen: HashSet<String> = tokenize(text).into_iter().collect();
            for term in seen {
                postings.entry(term).or_default().push(i);
            }
        }
        Self { postings }
    }

    pub fn term_count(&self) -> usize {
        self.postings.len()
    }

    /// Returns verse indices in ascending (canonical) order.
    pub fn query(&self, query: &str, mode: SearchMode, max_results: usize) -> Vec<usize> {
        let terms: Vec<String> = tokenize(query);
        if terms.is_empty() {
            return Vec::new();
        }
        match mode {
            SearchMode::All => {
                // Intersect postings, smallest first.
                let mut lists: Vec<&Vec<usize>> = Vec::new();
                for t in &terms {
                    match self.postings.get(t) {
                        Some(l) => lists.push(l),
                        None => return Vec::new(),
                    }
                }
                lists.sort_by_key(|l| l.len());
                let mut result: Vec<usize> = lists[0].clone();
                for list in lists.iter().skip(1) {
                    result = intersect_sorted(&result, list);
                    if result.is_empty() {
                        break;
                    }
                }
                result.truncate(max_results);
                result
            }
            SearchMode::Any => {
                let mut set: HashSet<usize> = HashSet::new();
                for t in &terms {
                    if let Some(l) = self.postings.get(t) {
                        set.extend(l.iter().copied());
                    }
                }
                let mut result: Vec<usize> = set.into_iter().collect();
                result.sort_unstable();
                result.truncate(max_results);
                result
            }
        }
    }
}

fn intersect_sorted(a: &[usize], b: &[usize]) -> Vec<usize> {
    let mut out = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Equal => {
                out.push(a[i]);
                i += 1;
                j += 1;
            }
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
        }
    }
    out
}

/// Build a ~120-char window around the first query-term hit (case-insensitive).
pub fn make_snippet(text: &str, query_terms: &[String]) -> String {
    const RADIUS: usize = 60;
    let lower = text.to_lowercase();
    let mut pos: Option<usize> = None;
    for t in query_terms {
        if let Some(p) = lower.find(t.as_str()) {
            pos = Some(pos.map_or(p, |q| q.min(p)));
        }
    }
    let Some(p) = pos else {
        return truncate_chars(text, 120);
    };
    // Snap to char boundaries.
    let start = floor_char_boundary(text, p.saturating_sub(RADIUS));
    let end = ceil_char_boundary(text, (p + RADIUS).min(text.len()));
    let mut s = String::new();
    if start > 0 {
        s.push('…');
    }
    s.push_str(&text[start..end]);
    if end < text.len() {
        s.push('…');
    }
    s
}

fn truncate_chars(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    let end = ceil_char_boundary(s, s.floor_char_boundary(n));
    format!("{}…", &s[..end])
}

fn floor_char_boundary(s: &str, mut i: usize) -> usize {
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

fn ceil_char_boundary(s: &str, mut i: usize) -> usize {
    while i < s.len() && !s.is_char_boundary(i) {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_basic() {
        assert_eq!(
            tokenize("For God so loved!"),
            vec!["for", "god", "so", "loved"]
        );
    }

    #[test]
    fn and_requires_all_terms() {
        let idx = SearchIndex::build(&["faith hope love".to_string(), "faith only".to_string()]);
        assert_eq!(idx.query("faith hope", SearchMode::All, 10), vec![0]);
        assert_eq!(idx.query("faith hope", SearchMode::Any, 10), vec![0, 1]);
    }

    #[test]
    fn snippet_windows_hit() {
        let text = "In the beginning God created the heaven and the earth.";
        let sn = make_snippet(text, &["god".to_string()]);
        assert!(sn.contains("God"));
    }
}
