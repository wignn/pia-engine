UPDATE forex_news_articles
SET original_title = regexp_replace(
    original_title,
    '\s+[-–—]\s+(Reuters|CNBC|MarketWatch|Investing\.com|Yahoo Finance|Bloomberg|Associated Press|AP|Dow Jones)$',
    '',
    'i'
)
WHERE original_title ~* '\s+[-–—]\s+(Reuters|CNBC|MarketWatch|Investing\.com|Yahoo Finance|Bloomberg|Associated Press|AP|Dow Jones)$';
