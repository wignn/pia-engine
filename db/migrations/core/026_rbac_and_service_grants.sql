-- 026_rbac_and_service_grants.sql
-- Role-Based Access Control (RBAC) and Least Privilege Service Grants

-- 1. Create Dedicated Service Roles If Not Exists
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'auth_user') THEN
        CREATE ROLE auth_user WITH LOGIN;
    END IF;
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'market_user') THEN
        CREATE ROLE market_user WITH LOGIN;
    END IF;
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'news_user') THEN
        CREATE ROLE news_user WITH LOGIN;
    END IF;
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'macro_sink_user') THEN
        CREATE ROLE macro_sink_user WITH LOGIN;
    END IF;
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'intelligence_user') THEN
        CREATE ROLE intelligence_user WITH LOGIN;
    END IF;
    IF NOT EXISTS (SELECT FROM pg_roles WHERE rolname = 'readonly_slave_user') THEN
        CREATE ROLE readonly_slave_user WITH LOGIN;
    END IF;
END $$;

-- 2. Configure Role Search Paths
ALTER ROLE auth_user SET search_path TO auth, platform, public;
ALTER ROLE market_user SET search_path TO market, platform, public;
ALTER ROLE news_user SET search_path TO news, platform, public;
ALTER ROLE macro_sink_user SET search_path TO macro, news, platform, public;
ALTER ROLE intelligence_user SET search_path TO market, news, macro, platform, public;
ALTER ROLE readonly_slave_user SET search_path TO market, news, macro, auth, public;

-- 3. Grant Schema Usages
GRANT USAGE ON SCHEMA auth, platform, public TO auth_user;
GRANT USAGE ON SCHEMA market, platform, public TO market_user;
GRANT USAGE ON SCHEMA news, platform, public TO news_user;
GRANT USAGE ON SCHEMA macro, news, platform, public TO macro_sink_user;
GRANT USAGE ON SCHEMA market, news, macro, platform, public TO intelligence_user;
GRANT USAGE ON SCHEMA auth, market, news, macro, public TO readonly_slave_user;

-- 4. Domain 1: Auth Grants
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA auth TO auth_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA auth TO auth_user;
GRANT INSERT, SELECT ON TABLE platform.outbox_events, platform.deadletter_batches TO auth_user;

-- 5. Domain 2: Market Grants
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA market TO market_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA market TO market_user;
GRANT INSERT, SELECT ON TABLE platform.outbox_events, platform.deadletter_batches TO market_user;

-- 6. Domain 3: News Grants
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA news TO news_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA news TO news_user;
GRANT INSERT, SELECT ON TABLE platform.outbox_events, platform.deadletter_batches TO news_user;

-- 7. Domain 4: Macro & Ingestion Sink Grants
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA macro TO macro_sink_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA macro TO macro_sink_user;
-- Allow macro_sink_user to write scraped news article enrichments and deadletter logs
GRANT SELECT, UPDATE ON TABLE news.forex_news_articles TO macro_sink_user;
GRANT INSERT, SELECT ON TABLE platform.outbox_events, platform.deadletter_batches TO macro_sink_user;

-- 8. Intelligence Service Grants
GRANT SELECT, INSERT, UPDATE ON TABLE market.why_move_explanations TO intelligence_user;
GRANT SELECT ON ALL TABLES IN SCHEMA news TO intelligence_user;
GRANT SELECT ON ALL TABLES IN SCHEMA macro TO intelligence_user;
GRANT INSERT, SELECT ON TABLE platform.deadletter_batches TO intelligence_user;

-- 9. Read-Only Slave Grants
GRANT SELECT ON ALL TABLES IN SCHEMA market, news, macro, auth TO readonly_slave_user;
