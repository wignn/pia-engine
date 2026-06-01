use crate::repository::PriceAlert;
use crate::services::market_ws::CachedPrice;
use poise::serenity_prelude::{CreateEmbed, CreateEmbedFooter, Timestamp};

use super::MAX_ALERTS_PER_USER;

pub fn symbol_not_found(symbol: &str, symbols: &[String]) -> CreateEmbed {
    let description = if symbols.is_empty() {
        "No market data available yet. Please wait for the market feed to initialize.".to_string()
    } else {
        format!(
            "Symbol `{}` not found.\n\n**Available symbols:**\n{}",
            symbol,
            symbols
                .iter()
                .map(|s| format!("`{}`", s))
                .collect::<Vec<_>>()
                .join(" - ")
        )
    };

    CreateEmbed::new()
        .title("Symbol Not Found")
        .description(description)
        .color(0xF39C12u32)
        .footer(CreateEmbedFooter::new("Fio"))
}

pub fn no_market_data() -> CreateEmbed {
    CreateEmbed::new()
        .title("Market Prices")
        .description("No market data available yet. Please wait for the feed to initialize.")
        .color(0xF39C12u32)
        .footer(CreateEmbedFooter::new("Fio"))
}

pub fn price(symbol: &str, cached: &CachedPrice) -> CreateEmbed {
    let (color, arrow) = match cached.direction.as_str() {
        "buy" => (0x34D399u32, "BUY"),
        "sell" => (0xF87171u32, "SELL"),
        _ => (0x60A5FAu32, "BUY"),
    };

    let asset_label = match cached.asset_type.as_str() {
        "crypto" => "Crypto",
        "forex" => "Forex",
        "stock" => "Stock",
        _ => "Market",
    };

    let elapsed = cached.updated_at.elapsed();
    let ago = if elapsed.as_secs() < 60 {
        format!("{}s ago", elapsed.as_secs())
    } else {
        format!("{}m ago", elapsed.as_secs() / 60)
    };

    CreateEmbed::new()
        .title(format!("{} {} Price", arrow, symbol))
        .description(format!("## {}", display_price(cached)))
        .field("Type", asset_label, true)
        .field("Direction", &cached.direction, true)
        .field("Updated", ago, true)
        .color(color)
        .footer(CreateEmbedFooter::new("Fio - Powered by MT5"))
        .timestamp(Timestamp::now())
}

pub fn prices(all: &[CachedPrice]) -> CreateEmbed {
    let mut forex_lines = Vec::new();
    let mut crypto_lines = Vec::new();
    let mut stock_lines = Vec::new();

    let mut sorted = all.to_vec();
    sorted.sort_by(|a, b| a.symbol.cmp(&b.symbol));

    for price in &sorted {
        let line = format_market_line(price);
        if price.asset_type == "crypto" {
            crypto_lines.push(line);
        } else if price.asset_type == "stock" {
            stock_lines.push(line);
        } else {
            forex_lines.push(line);
        }
    }

    let mut embed = CreateEmbed::new()
        .title("Live Market Prices")
        .color(0x8B5CF6u32)
        .footer(CreateEmbedFooter::new("Fio - Powered by Infoway"))
        .timestamp(Timestamp::now());

    if !forex_lines.is_empty() {
        embed = embed.field("Forex", forex_lines.join("\n"), false);
    }
    if !crypto_lines.is_empty() {
        embed = embed.field("Crypto", crypto_lines.join("\n"), false);
    }
    if !stock_lines.is_empty() {
        embed = embed.field("Stocks", stock_lines.join("\n"), false);
    }

    embed
}

pub fn alert_limit_reached(count: i64) -> CreateEmbed {
    CreateEmbed::new()
        .title("Limit Tercapai")
        .description(format!(
            "Kamu sudah punya {} alert aktif. Maksimal {} alert.\nHapus alert yang tidak diperlukan dengan `/alert_remove`.",
            count, MAX_ALERTS_PER_USER
        ))
        .color(0xF39C12u32)
        .footer(CreateEmbedFooter::new("Fio"))
}

pub fn invalid_target() -> CreateEmbed {
    CreateEmbed::new()
        .title("Invalid Target")
        .description("Target price tidak boleh sama dengan harga saat ini.")
        .color(0xF39C12u32)
        .footer(CreateEmbedFooter::new("Fio"))
}

pub fn alert_created(
    symbol: &str,
    current: &CachedPrice,
    target_price: f64,
    direction: &str,
    alert_id: i64,
) -> CreateEmbed {
    let current_display = display_price(current);
    let target_display = display_target(current, target_price);
    let direction_label = if direction == "above" {
        "naik di atas"
    } else {
        "turun di bawah"
    };

    CreateEmbed::new()
        .title("Price Alert Aktif")
        .description(format!(
            "Alert akan dikirim via DM ketika **{}** {} **{}**",
            symbol, direction_label, target_display
        ))
        .field("Symbol", symbol, true)
        .field("Harga Saat Ini", current_display, true)
        .field("Target", &target_display, true)
        .field("Direction", direction, true)
        .field("Alert ID", format!("#{}", alert_id), true)
        .color(0x8B5CF6u32)
        .footer(CreateEmbedFooter::new("Fio Price Alert"))
        .timestamp(Timestamp::now())
}

pub fn empty_alerts() -> CreateEmbed {
    CreateEmbed::new()
        .title("Price Alerts")
        .description("Kamu belum punya alert aktif.\nGunakan `/alert <symbol> <target>` untuk membuat alert.")
        .color(0x60A5FAu32)
        .footer(CreateEmbedFooter::new("Fio"))
}

pub fn alert_list(user_alerts: &[PriceAlert]) -> CreateEmbed {
    let lines = user_alerts
        .iter()
        .map(|alert| {
            let dir_icon = if alert.direction == "above" {
                "UP"
            } else {
                "DOWN"
            };
            let current = crate::services::market_ws::get_price(&alert.symbol);
            let current_str = current
                .as_ref()
                .map(display_price)
                .unwrap_or_else(|| "-".to_string());
            let is_crypto = current
                .as_ref()
                .map(|price| price.asset_type == "crypto")
                .unwrap_or(false);
            let target_str = if is_crypto {
                format!("${:.2}", alert.target_price)
            } else {
                format!("{:.5}", alert.target_price)
            };

            format!(
                "`#{}` {} **{}** {} {} (now: {})",
                alert.id, dir_icon, alert.symbol, alert.direction, target_str, current_str
            )
        })
        .collect::<Vec<_>>();

    CreateEmbed::new()
        .title(format!(
            "Price Alerts ({}/{})",
            user_alerts.len(),
            MAX_ALERTS_PER_USER
        ))
        .description(lines.join("\n"))
        .color(0x8B5CF6u32)
        .footer(CreateEmbedFooter::new(
            "Hapus alert dengan /market_alert_remove <id>",
        ))
        .timestamp(Timestamp::now())
}

pub fn alert_removed(alert_id: i64) -> CreateEmbed {
    CreateEmbed::new()
        .title("Alert Dihapus")
        .description(format!("Alert `#{}` berhasil dihapus.", alert_id))
        .color(0x34D399u32)
        .footer(CreateEmbedFooter::new("Fio"))
}

pub fn alert_not_found(alert_id: i64) -> CreateEmbed {
    CreateEmbed::new()
        .title("Alert Tidak Ditemukan")
        .description(format!(
            "Alert `#{}` tidak ditemukan atau bukan milikmu.",
            alert_id
        ))
        .color(0xF39C12u32)
        .footer(CreateEmbedFooter::new("Fio"))
}

fn format_market_line(price: &CachedPrice) -> String {
    let arrow = match price.direction.as_str() {
        "buy" => "UP",
        "sell" => "DOWN",
        _ => "FLAT",
    };

    if price.asset_type == "crypto" {
        format!("{} **{}** - ${}", arrow, price.symbol, price.price_str)
    } else {
        format!("{} **{}** - {}", arrow, price.symbol, price.price_str)
    }
}

fn display_price(price: &CachedPrice) -> String {
    if price.asset_type == "crypto" {
        format!("${}", price.price_str)
    } else {
        price.price_str.clone()
    }
}

fn display_target(price: &CachedPrice, target_price: f64) -> String {
    if price.asset_type == "crypto" {
        format!("${:.2}", target_price)
    } else {
        format!("{:.5}", target_price)
    }
}
