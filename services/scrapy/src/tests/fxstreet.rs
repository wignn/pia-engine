#[tokio::test]
async fn scraping_status() {
    super::scrape_and_print(
        "fxstreet",
        "https://www.fxstreet.com/news/british-pound-locked-in-tight-ranges-against-us-dollar-uob-202608140713",
    )
    .await;
}
