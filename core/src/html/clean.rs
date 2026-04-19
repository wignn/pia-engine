use regex::Regex;
use std::sync::LazyLock;

static RE_SCRIPT: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<script[^>]*>.*?</script>").unwrap());
static RE_STYLE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<style[^>]*>.*?</style>").unwrap());
static RE_HTML_TAGS: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").unwrap());
static RE_CDATA: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<!\[CDATA\[(.*?)\]\]>").unwrap());
static RE_MULTI_NL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\n{3,}").unwrap());
static RE_MULTI_SPACE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"\s{2,}").unwrap());
static RE_PARAGRAPHS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?is)<p[^>]*>(.*?)</p>").unwrap());

static NOISE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)Subscribe to.*?newsletter").unwrap(),
        Regex::new(r"(?i)Sign up for.*?alerts").unwrap(),
        Regex::new(r"(?im)Follow us on.*$").unwrap(),
        Regex::new(r"(?im)Read more:.*$").unwrap(),
        Regex::new(r"(?im)Related:.*$").unwrap(),
    ]
});

/// Strip all HTML tags from a string, handling CDATA, scripts, and styles.
pub fn strip_tags(s: &str) -> String {
    let s = RE_CDATA.replace_all(s, "$1");
    let s = RE_SCRIPT.replace_all(&s, "");
    let s = RE_STYLE.replace_all(&s, "");
    let s = RE_HTML_TAGS.replace_all(&s, "");
    let s = html_escape::decode_html_entities(&s);
    s.trim().to_string()
}

/// Clean content by collapsing whitespace and removing newsletter/noise patterns.
pub fn clean_content(text: &str) -> String {
    let mut text = RE_MULTI_NL.replace_all(text, "\n\n").to_string();
    text = RE_MULTI_SPACE.replace_all(&text, " ").to_string();
    for p in NOISE_PATTERNS.iter() {
        text = p.replace_all(&text, "").to_string();
    }
    text.trim().to_string()
}

/// Extract a summary from HTML description, truncating at sentence boundaries.
pub fn extract_summary(description: &str, max_len: usize) -> String {
    let text = strip_tags(description);
    let text = clean_content(&text);

    if text.len() <= max_len {
        return text;
    }

    if let Some(cut_pos) = text[..max_len].rfind('.') {
        if cut_pos > max_len / 2 {
            return text[..=cut_pos].to_string();
        }
    }

    if let Some(cut_pos) = text[..max_len].rfind(' ') {
        if cut_pos > 0 {
            return format!("{}...", &text[..cut_pos]);
        }
    }

    format!("{}...", &text[..max_len])
}

/// Extract text from `<p>` tags in HTML content.
pub fn extract_paragraphs(html_content: &str) -> String {
    let captures: Vec<_> = RE_PARAGRAPHS.captures_iter(html_content).collect();
    if captures.is_empty() {
        return strip_tags(html_content);
    }

    let parts: Vec<String> = captures
        .iter()
        .filter_map(|cap| {
            let cleaned = strip_tags(&cap[1]).trim().to_string();
            if cleaned.is_empty() {
                None
            } else {
                Some(cleaned)
            }
        })
        .collect();

    parts.join("\n\n")
}
