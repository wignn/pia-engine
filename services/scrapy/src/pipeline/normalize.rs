use crate::models::News;

pub fn normalize(mut news: News) -> News {
    fn clean(value: &mut Option<String>) {
        if let Some(text) = value.as_mut() {
            *text = text.split_whitespace().collect::<Vec<_>>().join(" ");
            if text.is_empty() {
                *value = None;
            }
        }
    }

    clean(&mut news.title);
    clean(&mut news.author);
    clean(&mut news.published_time);
    clean(&mut news.content);
    news
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_fields() {
        let news = normalize(News {
            title: Some("  hello  world ".into()),
            author: Some(" ".into()),
            published_time: None,
            content: None,
            url: "https://example.com".into(),
        });
        assert_eq!(news.title.as_deref(), Some("hello world"));
        assert!(news.author.is_none());
    }
}
