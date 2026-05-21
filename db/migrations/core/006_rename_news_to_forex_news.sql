-- Rename generic 'news_*' tables to 'forex_news_*' for consistency with 'stock_news'
ALTER TABLE IF EXISTS news_sources RENAME TO forex_news_sources;
ALTER TABLE IF EXISTS news_articles RENAME TO forex_news_articles;
ALTER TABLE IF EXISTS news_analyses RENAME TO forex_news_analyses;

-- Rename indexes
ALTER INDEX IF EXISTS idx_news_articles_processed_at RENAME TO idx_forex_news_articles_processed_at;
ALTER INDEX IF EXISTS idx_news_articles_content_hash RENAME TO idx_forex_news_articles_content_hash;
ALTER INDEX IF EXISTS idx_news_analyses_article_id RENAME TO idx_forex_news_analyses_article_id;
