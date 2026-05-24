use serde_json::Value;

pub fn symbol_for(provider_symbol: &str, public_symbol: &str, asset_type: &str) -> String {
    let provider = provider_symbol.trim().to_uppercase();
    if provider.contains(':') {
        return provider;
    }

    let public = public_symbol.trim().to_uppercase();
    match public.as_str() {
        "XAUUSD" => "OANDA:XAUUSD".to_string(),
        "SPX" => "SP:SPX".to_string(),
        "DXY" => "TVC:DXY".to_string(),
        _ if asset_type.eq_ignore_ascii_case("forex") && public.len() == 6 => {
            format!("FX:{public}")
        }
        _ => provider,
    }
}

pub async fn fetch_quote(
    client: &reqwest::Client,
    template: &str,
    symbol: &str,
) -> anyhow::Result<f64> {
    let val = if template.trim().is_empty() {
        fetch_scanner_quote(client, symbol).await?
    } else {
        let url = template
            .trim()
            .replace("{symbol}", &url_encode_symbol(symbol));
        let res = client.get(&url).send().await?;
        if !res.status().is_success() {
            anyhow::bail!("HTTP status error: {}", res.status());
        }
        res.json().await?
    };

    parse_quote(&val).ok_or_else(|| anyhow::anyhow!("failed to parse quote price from response"))
}

async fn fetch_scanner_quote(client: &reqwest::Client, symbol: &str) -> anyhow::Result<Value> {
    let payload = serde_json::json!({
        "symbols": {
            "tickers": [symbol],
            "query": { "types": [] }
        },
        "columns": ["close", "currency", "description", "exchange"]
    });

    let res = client
        .post("https://scanner.tradingview.com/global/scan")
        .json(&payload)
        .send()
        .await?;
    if !res.status().is_success() {
        anyhow::bail!("TradingView scanner HTTP status error: {}", res.status());
    }

    Ok(res.json().await?)
}

pub fn parse_quote(value: &Value) -> Option<f64> {
    direct_number(
        value,
        &["price", "last", "close", "lp", "regularMarketPrice"],
    )
    .or_else(|| scanner_price(value))
    .or_else(|| value.get("d").and_then(parse_quote_array))
    .or_else(|| value.get("data").and_then(parse_quote_array))
    .or_else(|| value.get("quote").and_then(parse_quote))
    .or_else(|| value.get("result").and_then(parse_quote_array))
    .or_else(|| yahoo_chart_price(value))
}

fn parse_quote_array(value: &Value) -> Option<f64> {
    match value {
        Value::Array(items) => items.iter().find_map(parse_quote),
        Value::Object(_) => parse_quote(value),
        _ => None,
    }
}

fn scanner_price(value: &Value) -> Option<f64> {
    value
        .get("data")?
        .as_array()?
        .first()?
        .get("d")?
        .as_array()?
        .first()
        .and_then(as_price)
}

fn direct_number(value: &Value, keys: &[&str]) -> Option<f64> {
    keys.iter()
        .find_map(|key| value.get(*key).and_then(as_price))
}

fn as_price(value: &Value) -> Option<f64> {
    match value {
        Value::Number(n) => n.as_f64().filter(|p| *p > 0.0),
        Value::String(s) => s.parse::<f64>().ok().filter(|p| *p > 0.0),
        _ => None,
    }
}

fn yahoo_chart_price(value: &Value) -> Option<f64> {
    value
        .get("chart")?
        .get("result")?
        .as_array()?
        .first()?
        .get("meta")?
        .get("regularMarketPrice")
        .and_then(as_price)
}

fn url_encode_symbol(symbol: &str) -> String {
    symbol.replace(':', "%3A")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn maps_common_symbols() {
        assert_eq!(symbol_for("XAUUSD", "XAUUSD", "forex"), "OANDA:XAUUSD");
        assert_eq!(symbol_for("SPX", "SPX", "index"), "SP:SPX");
        assert_eq!(symbol_for("DXY", "DXY", "index"), "TVC:DXY");
        assert_eq!(symbol_for("EURUSD", "EURUSD", "forex"), "FX:EURUSD");
        assert_eq!(
            symbol_for("OANDA:XAUUSD", "XAUUSD", "forex"),
            "OANDA:XAUUSD"
        );
    }

    #[test]
    fn parses_quote_shapes() {
        assert_eq!(parse_quote(&json!({ "price": 2345.6 })), Some(2345.6));
        assert_eq!(
            parse_quote(&json!({ "d": [{ "lp": 5010.2 }] })),
            Some(5010.2)
        );
        assert_eq!(
            parse_quote(&json!({ "quote": { "last": "1.0867" } })),
            Some(1.0867)
        );
        assert_eq!(
            parse_quote(&json!({ "data": [{ "s": "ok", "d": [2345.6] }] })),
            Some(2345.6)
        );
        assert_eq!(
            parse_quote(
                &json!({ "chart": { "result": [{ "meta": { "regularMarketPrice": 100.0 } }] } })
            ),
            Some(100.0)
        );
    }
}
