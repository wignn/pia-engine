ALTER TABLE forex_news_articles
    ADD COLUMN IF NOT EXISTS media_url TEXT;

COMMENT ON COLUMN forex_news_articles.media_url IS 'Main media/image URL extracted from news article';

-- Refresh view so media_url column is exposed in news.forex_news_articles
CREATE OR REPLACE VIEW news.forex_news_articles AS
SELECT * FROM public.forex_news_articles;
