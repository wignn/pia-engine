mod fxstreet;
mod investing_live;

use std::time::Duration;

use crate::{models::News, scraper::Scraper};

async fn scrape_and_print(source: &str, default_url: &str) {
    let env_name = format!("SCRAPE_TEST_{}_URL", source.to_uppercase());
    let url = std::env::var(&env_name).unwrap_or_else(|_| default_url.to_owned());
    let status = async {
        let scraper = Scraper::new(Duration::from_secs(20), 4 * 1024 * 1024)?;
        match source {
            "fxstreet" => scraper.scrape_fxstreet(&url).await,
            "investing_live" => scraper.scrape_investing_live(&url).await,
            _ => anyhow::bail!("unknown test source: {source}"),
        }
    }
    .await;

    print_status(source, &url, status);
}

fn print_status(source: &str, url: &str, status: anyhow::Result<News>) {
    match status {
        Ok(news) => println!(
            "[{source}] SCRAPING BERHASIL: {} ({url})",
            news.title.as_deref().unwrap_or("judul tidak ditemukan")
        ),
        Err(error) => println!("[{source}] SCRAPING GAGAL: {error} ({url})"),
    }
}
