from __future__ import annotations

from datetime import datetime, timedelta, timezone

import pandas as pd

from .config import ResearchConfig

MARKET_SYMBOLS = (
    "XAUUSD", "XAGUSD", "DXY", "USDX", "EURUSD", "USDJPY", "SPY", "GLD", "VIX", "GC=F"
)
MARKET_RESOLUTIONS = ("5m", "15m", "1h")


def _utc(value: str, end_of_day: bool = False) -> datetime:
    parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=timezone.utc)
    parsed = parsed.astimezone(timezone.utc)
    if end_of_day and len(value) == 10:
        parsed += timedelta(days=1)
    return parsed


def _sql_time(value: datetime) -> str:
    return value.strftime("%Y-%m-%d %H:%M:%S") + ".000"


def _clickhouse_client(url_str: str):
    import urllib.parse
    import clickhouse_connect

    parsed = urllib.parse.urlparse(url_str)
    return clickhouse_connect.get_client(
        host=parsed.hostname or "localhost",
        port=parsed.port or 8123,
        username=parsed.username or "default",
        password=parsed.password or "",
        database=parsed.path.lstrip("/") or "default",
    )


def load_market_panel(config: ResearchConfig, start: str, end: str) -> pd.DataFrame:
    if not config.clickhouse_url:
        raise ValueError("Set TRADING_MODEL_CLICKHOUSE_URL to read candle history")

    from_time = _utc(start) - timedelta(days=90)
    to_time = _utc(end, end_of_day=True) + timedelta(hours=1)
    symbols = ",".join(f"'{symbol}'" for symbol in MARKET_SYMBOLS)
    client = _clickhouse_client(config.clickhouse_url)
    rows: list[pd.DataFrame] = []
    try:
        for resolution in MARKET_RESOLUTIONS:
            table = f"market.ohlcv_candles_{resolution}"
            query = f"""
                SELECT symbol, '{resolution}' AS resolution, time,
                       argMinMerge(open_state) AS open,
                       maxMerge(high_state) AS high,
                       minMerge(low_state) AS low,
                       argMaxMerge(close_state) AS close,
                       sumMerge(volume_state) AS volume
                FROM {table}
                WHERE symbol IN ({symbols})
                  AND time >= toDateTime64('{_sql_time(from_time)}', 3, 'UTC')
                  AND time < toDateTime64('{_sql_time(to_time)}', 3, 'UTC')
                GROUP BY symbol, time
                ORDER BY symbol, time
            """
            rows.append(client.query_df(query))
    finally:
        client.close()

    panel = pd.concat(rows, ignore_index=True) if rows else pd.DataFrame()
    if not panel.empty:
        panel["time"] = pd.to_datetime(panel["time"], utc=True)
    return panel


def load_side_channels(config: ResearchConfig, start: str, end: str) -> dict[str, pd.DataFrame]:
    if not config.postgres_url:
        raise ValueError("Set TRADING_MODEL_POSTGRES_URL to read side-channel data")
    import psycopg
    from psycopg.rows import dict_row

    from_time = _utc(start) - timedelta(days=365)
    to_time = _utc(end, end_of_day=True) + timedelta(hours=1)
    result: dict[str, pd.DataFrame] = {}

    queries = {
        "news": """
            SELECT a.id::text AS id, a.original_title AS title, a.summary,
                   a.original_content AS content, a.published_at, a.processed_at,
                   COALESCE(an.sentiment, '') AS sentiment,
                   COALESCE(an.impact_level, '') AS impact_level,
                   COALESCE(an.currency_pairs, '') AS currency_pairs,
                   GREATEST(COALESCE(a.published_at, a.created_at),
                            COALESCE(a.processed_at, a.created_at),
                            COALESCE(an.created_at, a.created_at)) AS available_at
            FROM news.forex_news_articles a
            LEFT JOIN LATERAL (
                SELECT sentiment, impact_level, currency_pairs, created_at
                FROM news.forex_news_analyses WHERE article_id = a.id
                ORDER BY created_at DESC LIMIT 1
            ) an ON TRUE
            WHERE GREATEST(COALESCE(a.published_at, a.created_at),
                           COALESCE(a.processed_at, a.created_at),
                           COALESCE(an.created_at, a.created_at)) BETWEEN %s AND %s
        """,
        "social": """
            SELECT event_id AS id, text, platform, created_at, fetched_at, inserted_at,
                   reply_count, retweet_count, like_count, quote_count,
                   GREATEST(fetched_at, inserted_at) AS available_at
            FROM news.social_posts
            WHERE GREATEST(fetched_at, inserted_at) BETWEEN %s AND %s
        """,
        "geosignals": """
            SELECT event_id AS id, timestamp AS event_at, source, title, summary, category,
                   severity_score, sentiment_score, confidence_score, affected_assets,
                   asset_impact, GREATEST(timestamp, created_at) AS available_at
            FROM news.geosignals
            WHERE GREATEST(timestamp, created_at) BETWEEN %s AND %s
        """,
        "macro": """
            SELECT history_id AS id, source, observation_type, series_key, observation_date,
                   value, raw_value, available_at, source_payload
            FROM macro.macro_observation_history
            WHERE available_at BETWEEN %s AND %s
        """,
        "options": """
            SELECT snapshot_id AS id, symbol, observed_at AS available_at, underlying_price,
                   put_call_ratio, max_pain_strike, total_open_interest, total_volume,
                   total_gex, iv_atm, source_payload
            FROM market.options_snapshot_history
            WHERE snapshot_kind = 'summary' AND observed_at BETWEEN %s AND %s
        """,
        "options_contracts": """
            SELECT c.snapshot_id, s.symbol, s.observed_at AS available_at, c.contract_symbol,
                   c.option_type, c.strike, c.expiration_date, c.implied_volatility,
                   c.delta, c.gamma, c.theta, c.vega, c.gex, c.open_interest, c.volume
            FROM market.options_contract_history c
            JOIN market.options_snapshot_history s USING (snapshot_id)
            WHERE s.snapshot_kind = 'chain' AND s.observed_at BETWEEN %s AND %s
        """,
        "option_chains": """
            SELECT snapshot_id AS id, symbol, observed_at AS available_at, underlying_price
            FROM market.options_snapshot_history
            WHERE snapshot_kind = 'chain' AND observed_at BETWEEN %s AND %s
        """,
        "calendar": """
            SELECT id::text AS id, source, country, event_name, impact, actual, forecast,
                   previous, event_time, created_at, updated_at, updated_at AS available_at
            FROM macro.economic_calendar_events
            WHERE updated_at BETWEEN %s AND %s
        """,
        "cot": """
            SELECT market_code, market_name, report_date, report_type, commercial_long,
                   commercial_short, noncommercial_long, noncommercial_short,
                   nonreportable_long, nonreportable_short, open_interest, created_at, updated_at,
                   updated_at AS available_at
            FROM macro.cot_reports
            WHERE updated_at BETWEEN %s AND %s
        """,
        "fear_greed": """
            SELECT id, scope, date AS event_at, score, label, components, created_at,
                   GREATEST(date, created_at) AS available_at
            FROM macro.fear_greed_index
            WHERE GREATEST(date, created_at) BETWEEN %s AND %s
        """,
        "central_bank": """
            SELECT id, bank, document_type, title, published_at, summary, stance, confidence,
                   raw_text, created_at,
                   GREATEST(COALESCE(published_at, created_at), created_at) AS available_at
            FROM news.central_bank_documents
            WHERE GREATEST(COALESCE(published_at, created_at), created_at) BETWEEN %s AND %s
        """,
    }
    optional_tables = {
        "macro": "macro.macro_observation_history",
        "options": "market.options_snapshot_history",
        "options_contracts": "market.options_contract_history",
        "option_chains": "market.options_snapshot_history",
        "calendar": "macro.economic_calendar_events",
        "cot": "macro.cot_reports",
        "fear_greed": "macro.fear_greed_index",
        "central_bank": "news.central_bank_documents",
    }
    with psycopg.connect(config.postgres_url, row_factory=dict_row) as connection:
        connection.execute("SET TRANSACTION READ ONLY")
        for name, query in queries.items():
            table = optional_tables.get(name)
            if table:
                present = connection.execute("SELECT to_regclass(%s) IS NOT NULL AS present", (table,)).fetchone()["present"]
                if not present:
                    result[name] = pd.DataFrame()
                    continue
            rows = connection.execute(query, (from_time, to_time)).fetchall()
            result[name] = pd.DataFrame(rows)
    return result
