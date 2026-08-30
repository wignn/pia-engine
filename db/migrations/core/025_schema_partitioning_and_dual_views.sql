-- 025_schema_partitioning_and_dual_views.sql
-- Strategic Database Decoupling: Logical Schema Partitioning with Backward-Compatible Dual Views

-- 1. Create Core Domain Schemas
CREATE SCHEMA IF NOT EXISTS auth;
CREATE SCHEMA IF NOT EXISTS market;
CREATE SCHEMA IF NOT EXISTS news;
CREATE SCHEMA IF NOT EXISTS macro;
CREATE SCHEMA IF NOT EXISTS platform;

-- 2. Helper Procedure to Safely Migrate Table to Schema and Create Public View
DO $$
DECLARE
    t_name text;
    has_table boolean;
    has_view boolean;
BEGIN
    -- =========================================================================
    -- DOMAIN 1: AUTH & CONTROL-PLANE (auth.*)
    -- =========================================================================
    FOR t_name IN SELECT unnest(ARRAY['users', 'api_keys', 'plans', 'oauth_accounts', 'tenant_configs', 'usage_logs'])
    LOOP
        SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = t_name AND table_type = 'BASE TABLE'
        ) INTO has_table;

        IF has_table THEN
            EXECUTE format('ALTER TABLE public.%I SET SCHEMA auth', t_name);
        END IF;

        -- Create Updatable Backward-Compatible View in public
        SELECT EXISTS (
            SELECT FROM information_schema.views
            WHERE table_schema = 'public' AND table_name = t_name
        ) INTO has_view;

        IF NOT has_view THEN
            EXECUTE format('CREATE OR REPLACE VIEW public.%I AS SELECT * FROM auth.%I', t_name, t_name);
        END IF;
    END LOOP;

    -- =========================================================================
    -- DOMAIN 2: MARKET DATA & VOLATILITY (market.*)
    -- =========================================================================
    FOR t_name IN SELECT unnest(ARRAY[
        'ohlcv_candles', 'market_latest_prices', 'exchanges', 'exchange_holidays',
        'symbol_exchange_map', 'trading_halts', 'corporate_actions',
        'realized_volatility', 'options_snapshots', 'options_contracts',
        'why_move_explanations'
    ])
    LOOP
        SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = t_name AND table_type = 'BASE TABLE'
        ) INTO has_table;

        IF has_table THEN
            EXECUTE format('ALTER TABLE public.%I SET SCHEMA market', t_name);
        END IF;

        SELECT EXISTS (
            SELECT FROM information_schema.views
            WHERE table_schema = 'public' AND table_name = t_name
        ) INTO has_view;

        IF NOT has_view THEN
            EXECUTE format('CREATE OR REPLACE VIEW public.%I AS SELECT * FROM market.%I', t_name, t_name);
        END IF;
    END LOOP;

    -- =========================================================================
    -- DOMAIN 3: NEWS & INSTITUTIONAL INTELLIGENCE (news.*)
    -- =========================================================================
    FOR t_name IN SELECT unnest(ARRAY[
        'forex_news_sources', 'forex_news_articles', 'forex_news_analyses',
        'stock_news', 'url_analysis_cache', 'sec_companies', 'sec_filings',
        'central_bank_sources', 'central_bank_documents'
    ])
    LOOP
        SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = t_name AND table_type = 'BASE TABLE'
        ) INTO has_table;

        IF has_table THEN
            EXECUTE format('ALTER TABLE public.%I SET SCHEMA news', t_name);
        END IF;

        SELECT EXISTS (
            SELECT FROM information_schema.views
            WHERE table_schema = 'public' AND table_name = t_name
        ) INTO has_view;

        IF NOT has_view THEN
            EXECUTE format('CREATE OR REPLACE VIEW public.%I AS SELECT * FROM news.%I', t_name, t_name);
        END IF;
    END LOOP;

    -- =========================================================================
    -- DOMAIN 4: MACRO & ECONOMIC DATA (macro.*)
    -- =========================================================================
    FOR t_name IN SELECT unnest(ARRAY[
        'macro_series', 'macro_observations', 'macro_signals', 'macro_rates',
        'macro_rate_spreads', 'macro_bonds', 'energy_series', 'energy_observations',
        'cot_reports', 'cot_market_map', 'fear_greed_index', 'economic_calendar_events'
    ])
    LOOP
        SELECT EXISTS (
            SELECT FROM information_schema.tables
            WHERE table_schema = 'public' AND table_name = t_name AND table_type = 'BASE TABLE'
        ) INTO has_table;

        IF has_table THEN
            EXECUTE format('ALTER TABLE public.%I SET SCHEMA macro', t_name);
        END IF;

        SELECT EXISTS (
            SELECT FROM information_schema.views
            WHERE table_schema = 'public' AND table_name = t_name
        ) INTO has_view;

        IF NOT has_view THEN
            EXECUTE format('CREATE OR REPLACE VIEW public.%I AS SELECT * FROM macro.%I', t_name, t_name);
        END IF;
    END LOOP;

END $$;
