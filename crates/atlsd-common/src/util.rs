use std::sync::LazyLock;

use regex::Regex;

static SLUG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"[^a-z0-9]+").unwrap());

pub fn truncate_str(s: &str, max_len: usize) -> String {
    s.chars().take(max_len).collect()
}

pub fn truncate_bytes(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        s[..max_len].to_string()
    } else {
        s.to_string()
    }
}

pub fn to_slug(name: &str) -> String {
    let s = name.trim().to_lowercase();
    let s = SLUG_RE.replace_all(&s, "-");
    s.trim_matches('-').to_string()
}
