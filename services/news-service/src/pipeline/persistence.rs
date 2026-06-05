use serde_json::json;
use sqlx::{PgPool, Row};

use super::analysis::ArticleAnalysis;
use super::sources::NewsSource;
use super::text::ParsedArticle;

fn geosignal_event_id(content_hash: &str) -> String {
    format!("forex:{content_hash}")
}

fn severity_score(impact_level: &str) -> f64 {
    match impact_level.trim().to_lowercase().as_str() {
        "high" => 0.75,
        "medium" => 0.5,
        "low" => 0.25,
        _ => 0.25,
    }
}

fn sentiment_score(sentiment: &str) -> f64 {
    match sentiment.trim().to_lowercase().as_str() {
        "positive" => 1.0,
        "negative" => -1.0,
        _ => 0.0,
    }
}

fn affected_assets(currency_pairs: &str) -> Vec<String> {
    currency_pairs
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_uppercase)
        .collect()
}

fn asset_impact(assets: &[String], severity_score: f64) -> serde_json::Value {
    serde_json::Value::Object(
        assets
            .iter()
            .map(|asset| (asset.clone(), json!(severity_score)))
            .collect(),
    )
}

struct GeopoliticalEnrichment {
    category: &'static str,
    country: Option<&'static str>,
    region: Option<&'static str>,
    location_scope: &'static str,
}

fn geopolitical_enrichment(text: &str) -> GeopoliticalEnrichment {
    let lower = text.to_lowercase();
    let (country, region) = if lower.contains("china") {
        (Some("China"), Some("Asia"))
    } else if lower.contains("united states") || lower.contains(" u.s.") || lower.contains(" us ") {
        (Some("United States"), Some("North America"))
    } else if lower.contains("middle east") || lower.contains("red sea") {
        (None, Some("Middle East"))
    } else {
        (None, None)
    };

    let category = if lower.contains("shipping")
        || lower.contains("supply chain")
        || lower.contains("red sea")
    {
        "supply_chain"
    } else if lower.contains("tariff") || lower.contains("trade") || lower.contains("sanction") {
        "trade"
    } else if lower.contains("conflict") || lower.contains("war") || lower.contains("attack") {
        "conflict"
    } else if lower.contains("oil") || lower.contains("opec") || lower.contains("energy") {
        "energy"
    } else {
        "market_news"
    };

    GeopoliticalEnrichment {
        category,
        country,
        region,
        location_scope: if country.is_some() {
            "country"
        } else if region.is_some() {
            "region"
        } else {
            "global"
        },
    }
}

pub async fn insert_forex_article(
    pool: &PgPool,
    source: &NewsSource,
    article: &ParsedArticle,
    analysis: &ArticleAnalysis,
) -> anyhow::Result<usize> {
    let assets = affected_assets(&analysis.currency_pairs);
    let severity_score = severity_score(&analysis.impact_level);
    let enrichment = geopolitical_enrichment(&article.analysis_text());
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
        ), geosignal AS (
            INSERT INTO news.geosignals (event_id, timestamp, source, source_url, title, summary, category, country, region, location_scope, severity_score, sentiment_score, confidence_score, affected_assets, asset_impact, freshness)
            SELECT $11, COALESCE($7, NOW()), $12, $3, $4, $6, $17, $18, $19, $20, $13, $14, 0.5, $15, $16, 'fresh'
            FROM inserted
            ON CONFLICT (event_id) DO NOTHING
            RETURNING event_id
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
    .bind(geosignal_event_id(&article.content_hash))
    .bind(&source.name)
    .bind(severity_score)
    .bind(sentiment_score(&analysis.sentiment))
    .bind(&assets)
    .bind(asset_impact(&assets, severity_score))
    .bind(enrichment.category)
    .bind(enrichment.country)
    .bind(enrichment.region)
    .bind(enrichment.location_scope)
    .fetch_one(pool)
    .await?
    .try_get::<i64, _>(0)?;

    Ok(inserted as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_analysis_fields_to_geosignal_values() {
        assert_eq!(severity_score("low"), 0.25);
        assert_eq!(severity_score("medium"), 0.5);
        assert_eq!(severity_score("high"), 0.75);
        assert_eq!(severity_score("unknown"), 0.25);
        assert_eq!(sentiment_score("positive"), 1.0);
        assert_eq!(sentiment_score("negative"), -1.0);
        assert_eq!(sentiment_score("neutral"), 0.0);
        assert_eq!(sentiment_score("unknown"), 0.0);
    }

    #[test]
    fn parses_currency_pairs_into_affected_assets() {
        assert_eq!(
            affected_assets("EURUSD, XAUUSD, , USDCAD"),
            vec!["EURUSD", "XAUUSD", "USDCAD"]
        );
    }

    #[test]
    fn namespaces_forex_geosignal_event_id() {
        assert_eq!(geosignal_event_id("abc123"), "forex:abc123");
    }

    #[test]
    fn infers_country_and_region_from_article_text() {
        let enrichment = geopolitical_enrichment(
            "Oil rises as China demand and Red Sea shipping risks intensify",
        );
        assert_eq!(enrichment.country, Some("China"));
        assert_eq!(enrichment.region, Some("Asia"));
        assert_eq!(enrichment.location_scope, "country");
    }

    #[test]
    fn infers_geopolitical_category_from_article_text() {
        assert_eq!(
            geopolitical_enrichment("Red Sea shipping disruption lifts oil").category,
            "supply_chain"
        );
        assert_eq!(
            geopolitical_enrichment("New tariffs hit China trade flows").category,
            "trade"
        );
        assert_eq!(
            geopolitical_enrichment("Middle East conflict sends gold higher").category,
            "conflict"
        );
        assert_eq!(
            geopolitical_enrichment("Oil inventories tighten after OPEC cuts").category,
            "energy"
        );
        assert_eq!(
            geopolitical_enrichment("Euro gains after ECB commentary").category,
            "market_news"
        );
    }
}
