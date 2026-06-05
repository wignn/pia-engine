UPDATE forex_news_sources
SET name = 'Market News Wire',
    slug = 'market-news-wire',
    url = '',
    updated_at = NOW()
WHERE id = 'feed-finnhub-market-news';

CREATE OR REPLACE VIEW news.forex_news_sources AS
SELECT * FROM public.forex_news_sources;

COMMENT ON VIEW news.forex_news_sources IS 'News source metadata exposed under the news schema.';
