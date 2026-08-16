#[tokio::test]
async fn scraping_status() {
    super::scrape_and_print(
        "investing_live",
        "https://www.investing.com/news/forex-news",
    )
    .await;
}
