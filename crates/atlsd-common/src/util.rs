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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_str_respects_character_boundaries() {
        assert_eq!(truncate_str("abcdef", 3), "abc");
        assert_eq!(truncate_str("éclair", 2), "éc");
    }

    #[test]
    fn truncate_bytes_keeps_short_strings() {
        assert_eq!(truncate_bytes("short", 10), "short");
        assert_eq!(truncate_bytes("abcdef", 3), "abc");
    }

    #[test]
    fn to_slug_normalizes_whitespace_and_symbols() {
        assert_eq!(to_slug("  Hello, ATLSD World!  "), "hello-atlsd-world");
        assert_eq!(to_slug("---Already---Slug---"), "already-slug");
    }
}
