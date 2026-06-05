use atlsd_eventbus::subjects;
use chrono::{DateTime, Utc};
use serde_json::json;
use std::{collections::HashMap, time::Duration};
use tracing::{info, warn};

use crate::{alerts, state::AppState};

pub async fn run(state: AppState) {
    if !state.config.alert_notifications_enabled {
        info!("alert notifier disabled; ALERT_NOTIFICATIONS_ENABLED=false");
        return;
    }
    if state.clickhouse.is_none() {
        warn!("alert notifier disabled; ClickHouse is not configured");
        return;
    }

    let mut notifier = match AlertNotifier::connect(&state.config.nats_url).await {
        Ok(notifier) => notifier,
        Err(err) => {
            warn!(error = %err, "alert notifier failed to connect to NATS");
            return;
        }
    };

    let scan_interval = Duration::from_secs(state.config.alert_scan_sec);
    loop {
        notifier.scan_and_publish(&state).await;
        tokio::time::sleep(scan_interval).await;
    }
}

struct AlertNotifier {
    client: async_nats::Client,
    last_sent: HashMap<String, DateTime<Utc>>,
}

impl AlertNotifier {
    async fn connect(nats_url: &str) -> anyhow::Result<Self> {
        Ok(Self {
            client: async_nats::connect(nats_url).await?,
            last_sent: HashMap::new(),
        })
    }

    async fn scan_and_publish(&mut self, state: &AppState) {
        let window_minutes = 5;
        let threshold = crate::spikes::spike_threshold(window_minutes);
        let alerts = alerts::collect_alerts(state, window_minutes, threshold, 25).await;
        for alert in alerts {
            if !alerts::is_notifiable_severity(alert.severity) {
                continue;
            }
            if self.is_in_cooldown(&alert.symbol, state.config.alert_cooldown_sec) {
                continue;
            }
            if let Err(err) = self.publish_alert(&alert).await {
                warn!(error = %err, symbol = %alert.symbol, "failed to publish alert");
                continue;
            }
            self.last_sent.insert(alert.symbol.clone(), Utc::now());
        }
    }

    fn is_in_cooldown(&self, symbol: &str, cooldown_sec: u64) -> bool {
        let Some(last_sent) = self.last_sent.get(symbol) else {
            return false;
        };
        let elapsed = Utc::now().signed_duration_since(*last_sent);
        elapsed.num_seconds() < cooldown_sec as i64
    }

    async fn publish_alert(&self, alert: &alerts::Alert) -> anyhow::Result<()> {
        let payload = json!({
            "alert": alert,
            "discord_embed": {
                "title": format!("ALERT - {}", alert.symbol),
                "description": alert.reason,
                "color": if alert.severity == "critical" { 15548997 } else { 16753920 },
                "fields": [
                    { "name": "Severity", "value": alert.severity, "inline": true },
                    { "name": "Move", "value": format!("{:+.2}%", alert.move_pct), "inline": true },
                    { "name": "Ticks 5m", "value": alert.ticks_5m.to_string(), "inline": true },
                    { "name": "Action", "value": alert.recommended_action, "inline": false }
                ],
                "footer": { "text": "Fio Alert" }
            },
            "generated_at": Utc::now().to_rfc3339(),
        });
        let payload = serde_json::to_vec(&payload)?;
        self.client
            .publish(subjects::MARKET_ALERTS_V1.to_string(), payload.into())
            .await?;
        info!(symbol = %alert.symbol, severity = %alert.severity, "published alert");
        Ok(())
    }
}
