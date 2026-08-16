CREATE TABLE IF NOT EXISTS market.exchanges (
    exchange_code TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    operating_mic TEXT,
    country TEXT,
    currency TEXT,
    timezone TEXT NOT NULL,
    regular_open TIME,
    regular_close TIME,
    working_days TEXT[] NOT NULL DEFAULT ARRAY['Mon','Tue','Wed','Thu','Fri'],
    raw JSONB NOT NULL DEFAULT '{}'::jsonb,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS market.exchange_holidays (
    exchange_code TEXT NOT NULL REFERENCES market.exchanges(exchange_code) ON DELETE CASCADE,
    holiday_date DATE NOT NULL,
    name TEXT NOT NULL,
    holiday_type TEXT NOT NULL DEFAULT 'holiday',
    is_open BOOLEAN NOT NULL DEFAULT FALSE,
    raw JSONB NOT NULL DEFAULT '{}'::jsonb,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (exchange_code, holiday_date)
);

CREATE TABLE IF NOT EXISTS market.symbol_exchange_map (
    symbol TEXT PRIMARY KEY,
    exchange_code TEXT NOT NULL REFERENCES market.exchanges(exchange_code) ON DELETE CASCADE,
    provider_symbol TEXT,
    asset_type TEXT NOT NULL DEFAULT 'unknown',
    name TEXT,
    currency TEXT,
    raw JSONB NOT NULL DEFAULT '{}'::jsonb,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_exchange_holidays_date
    ON market.exchange_holidays(holiday_date);

CREATE INDEX IF NOT EXISTS idx_symbol_exchange_map_exchange
    ON market.symbol_exchange_map(exchange_code);

INSERT INTO market.exchanges (exchange_code, name, operating_mic, country, currency, timezone, regular_open, regular_close, working_days)
VALUES
    ('US', 'USA Exchanges', 'XNAS,XNYS', 'USA', 'USD', 'America/New_York', '09:30:00', '16:00:00', ARRAY['Mon','Tue','Wed','Thu','Fri']),
    ('IDX', 'Indonesia Stock Exchange', 'XIDX', 'Indonesia', 'IDR', 'Asia/Jakarta', '09:00:00', '16:00:00', ARRAY['Mon','Tue','Wed','Thu','Fri']),
    ('JP', 'Japan Exchange Group', 'XTKS', 'Japan', 'JPY', 'Asia/Tokyo', '09:00:00', '15:30:00', ARRAY['Mon','Tue','Wed','Thu','Fri']),
    ('HK', 'Hong Kong Exchanges', 'XHKG', 'Hong Kong', 'HKD', 'Asia/Hong_Kong', '09:30:00', '16:00:00', ARRAY['Mon','Tue','Wed','Thu','Fri']),
    ('CN', 'Shanghai Stock Exchange', 'XSHG', 'China', 'CNY', 'Asia/Shanghai', '09:30:00', '15:00:00', ARRAY['Mon','Tue','Wed','Thu','Fri']),
    ('KR', 'Korea Exchange', 'XKRX', 'South Korea', 'KRW', 'Asia/Seoul', '09:00:00', '15:30:00', ARRAY['Mon','Tue','Wed','Thu','Fri']),
    ('SG', 'Singapore Exchange', 'XSES', 'Singapore', 'SGD', 'Asia/Singapore', '09:00:00', '17:00:00', ARRAY['Mon','Tue','Wed','Thu','Fri']),
    ('AU', 'Australian Securities Exchange', 'XASX', 'Australia', 'AUD', 'Australia/Sydney', '10:00:00', '16:00:00', ARRAY['Mon','Tue','Wed','Thu','Fri']),
    ('IN', 'India Exchanges', 'XNSE,XBOM', 'India', 'INR', 'Asia/Kolkata', '09:15:00', '15:30:00', ARRAY['Mon','Tue','Wed','Thu','Fri']),
    ('CRYPTO', 'Cryptocurrency', NULL, NULL, NULL, 'UTC', '00:00:00', '23:59:59', ARRAY['Mon','Tue','Wed','Thu','Fri','Sat','Sun']),
    ('FX', 'Foreign Exchange', NULL, NULL, NULL, 'UTC', NULL, NULL, ARRAY['Mon','Tue','Wed','Thu','Fri','Sun'])
ON CONFLICT (exchange_code) DO NOTHING;

INSERT INTO market.symbol_exchange_map (symbol, exchange_code, provider_symbol, asset_type, name, currency)
VALUES
    ('SPX', 'US', 'SP:SPX', 'index', 'S&P 500', 'USD'),
    ('DXY', 'US', 'TVC:DXY', 'index', 'US Dollar Index', 'USD'),
    ('N225', 'JP', 'TVC:NI225', 'index', 'Nikkei 225', 'JPY'),
    ('HSI', 'HK', 'HSI:HSI', 'index', 'Hang Seng Index', 'HKD'),
    ('SSEC', 'CN', 'SSE:000001', 'index', 'Shanghai Composite', 'CNY'),
    ('KOSPI', 'KR', 'KRX:KOSPI', 'index', 'KOSPI Composite', 'KRW'),
    ('STI', 'SG', 'TVC:STI', 'index', 'Straits Times Index', 'SGD'),
    ('JCI', 'IDX', 'IDX:COMPOSITE', 'index', 'Jakarta Composite Index', 'IDR'),
    ('ASX200', 'AU', 'ASX:XJO', 'index', 'S&P/ASX 200', 'AUD'),
    ('NIFTY50', 'IN', 'NSE:NIFTY', 'index', 'NIFTY 50', 'INR'),
    ('SENSEX', 'IN', 'BSE:SENSEX', 'index', 'BSE Sensex', 'INR'),
    ('AAPL', 'US', 'NASDAQ:AAPL', 'stock', 'Apple Inc.', 'USD'),
    ('MSFT', 'US', 'NASDAQ:MSFT', 'stock', 'Microsoft Corporation', 'USD'),
    ('NVDA', 'US', 'NASDAQ:NVDA', 'stock', 'NVIDIA Corporation', 'USD'),
    ('GOOGL', 'US', 'NASDAQ:GOOGL', 'stock', 'Alphabet Inc.', 'USD'),
    ('META', 'US', 'NASDAQ:META', 'stock', 'Meta Platforms Inc.', 'USD'),
    ('AMZN', 'US', 'NASDAQ:AMZN', 'stock', 'Amazon.com Inc.', 'USD'),
    ('TSLA', 'US', 'NASDAQ:TSLA', 'stock', 'Tesla Inc.', 'USD'),
    ('AVGO', 'US', 'NASDAQ:AVGO', 'stock', 'Broadcom Inc.', 'USD'),
    ('BRKB', 'US', 'NYSE:BRK.B', 'stock', 'Berkshire Hathaway Inc. Class B', 'USD'),
    ('JPM', 'US', 'NYSE:JPM', 'stock', 'JPMorgan Chase & Co.', 'USD'),
    ('V', 'US', 'NYSE:V', 'stock', 'Visa Inc.', 'USD'),
    ('LLY', 'US', 'NYSE:LLY', 'stock', 'Eli Lilly and Company', 'USD'),
    ('WMT', 'US', 'NYSE:WMT', 'stock', 'Walmart Inc.', 'USD'),
    ('UNH', 'US', 'NYSE:UNH', 'stock', 'UnitedHealth Group Incorporated', 'USD'),
    ('COST', 'US', 'NASDAQ:COST', 'stock', 'Costco Wholesale Corporation', 'USD'),
    ('BBCA', 'IDX', 'IDX:BBCA', 'stock', 'Bank Central Asia', 'IDR'),
    ('BBRI', 'IDX', 'IDX:BBRI', 'stock', 'Bank Rakyat Indonesia', 'IDR'),
    ('BMRI', 'IDX', 'IDX:BMRI', 'stock', 'Bank Mandiri', 'IDR'),
    ('TLKM', 'IDX', 'IDX:TLKM', 'stock', 'Telkom Indonesia', 'IDR'),
    ('ASII', 'IDX', 'IDX:ASII', 'stock', 'Astra International', 'IDR'),
    ('UNVR', 'IDX', 'IDX:UNVR', 'stock', 'Unilever Indonesia', 'IDR'),
    ('ICBP', 'IDX', 'IDX:ICBP', 'stock', 'Indofood CBP', 'IDR'),
    ('BBNI', 'IDX', 'IDX:BBNI', 'stock', 'Bank Negara Indonesia', 'IDR'),
    ('ADRO', 'IDX', 'IDX:ADRO', 'stock', 'Adaro Energy', 'IDR'),
    ('MDKA', 'IDX', 'IDX:MDKA', 'stock', 'Merdeka Copper Gold', 'IDR')
ON CONFLICT (symbol) DO NOTHING;
