DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'news_sources')
       AND NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'forex_news_sources') THEN
        ALTER TABLE news_sources RENAME TO forex_news_sources;
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'news_articles')
       AND NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'forex_news_articles') THEN
        ALTER TABLE news_articles RENAME TO forex_news_articles;
    END IF;

    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'news_analyses')
       AND NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'forex_news_analyses') THEN
        ALTER TABLE news_analyses RENAME TO forex_news_analyses;
    END IF;
END $$;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_news_articles_processed_at')
       AND NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_forex_news_articles_processed_at') THEN
        ALTER INDEX idx_news_articles_processed_at RENAME TO idx_forex_news_articles_processed_at;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_news_articles_content_hash')
       AND NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_forex_news_articles_content_hash') THEN
        ALTER INDEX idx_news_articles_content_hash RENAME TO idx_forex_news_articles_content_hash;
    END IF;

    IF EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_news_analyses_article_id')
       AND NOT EXISTS (SELECT 1 FROM pg_indexes WHERE indexname = 'idx_forex_news_analyses_article_id') THEN
        ALTER INDEX idx_news_analyses_article_id RENAME TO idx_forex_news_analyses_article_id;
    END IF;
END $$;
