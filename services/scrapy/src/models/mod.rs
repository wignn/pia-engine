use atlsd_contracts::scrape::ScrapedNews;

pub use atlsd_contracts::scrape::{ScrapeJob, ScrapeResult, ScrapedNews as News};

pub fn validate_job(job: ScrapeJob, fallback_id: String) -> anyhow::Result<ValidatedJob> {
    let url = job.url.trim().to_owned();
    if url.is_empty() || !(url.starts_with("http://") || url.starts_with("https://")) {
        anyhow::bail!("url must be an absolute http(s) URL");
    }
    Ok(ValidatedJob {
        id: job
            .id
            .filter(|id| !id.trim().is_empty())
            .unwrap_or(fallback_id),
        url,
    })
}

#[derive(Debug, Clone)]
pub struct ValidatedJob {
    pub id: String,
    pub url: String,
}

const _: fn(Option<String>, Option<String>, Option<String>, Option<String>, String) -> ScrapedNews =
    |title, author, published_time, content, url| ScrapedNews {
        title,
        author,
        published_time,
        content,
        url,
    };
