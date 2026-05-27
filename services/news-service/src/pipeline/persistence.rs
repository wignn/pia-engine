use sqlx::{PgPool, Row};

use super::analysis::ArticleAnalysis;
use super::sources::NewsSource;
use super::text::ParsedArticle;

pub async fn insert_forex_article(
    pool: &PgPool,
    source: &NewsSource,
    article: &ParsedArticle,
    analysis: &ArticleAnalysis,
) -> anyhow::Result<usize> {
    let inserted = sqlx::query(
        "WITH inserted AS (
            INSERT INTO news.forex_news_articles (source_id, content_hash, original_url, original_title, original_content, summary, is_processed, processed_at, published_at)
            VALUES ($1, $2, $3, $4, $5, $6, TRUE, NOW(), $7)
            ON CONFLICT (content_hash) DO NOTHING
            RETURNING id
        ), analysis AS (
            INSERT INTO news.forex_news_analyses (article_id, sentiment, impact_level, currency_pairs)
            SELECT id, $8, $9, $10 FROM inserted
            RETURNING article_id
        )
        SELECT COUNT(*)::BIGINT FROM inserted",
    )
    .bind(&source.id)
    .bind(&article.content_hash)
    .bind(&article.url)
    .bind(&article.title)
    .bind(article.summary.as_deref())
    .bind(article.summary.as_deref())
    .bind(article.published_at)
    .bind(&analysis.sentiment)
    .bind(&analysis.impact_level)
    .bind(&analysis.currency_pairs)
    .fetch_one(pool)
    .await?
    .try_get::<i64, _>(0)?;

    Ok(inserted as usize)
}
