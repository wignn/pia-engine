use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

#[derive(Debug, Default)]
pub struct Metrics {
    active_ws_connections: AtomicUsize,
    ws_connections_total: AtomicU64,
    ws_connection_rejections_total: AtomicU64,
    ws_messages_out_total: AtomicU64,
    ws_messages_in_total: AtomicU64,
    ws_commands_total: AtomicU64,
    ws_pings_total: AtomicU64,
    ws_pongs_total: AtomicU64,
    ws_broadcasts_total: AtomicU64,
    ws_broadcast_recipients_total: AtomicU64,
    ws_send_failures_total: AtomicU64,
}

impl Metrics {
    pub fn connection_opened(&self) {
        self.active_ws_connections.fetch_add(1, Ordering::Relaxed);
        self.ws_connections_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn connection_closed(&self) {
        self.active_ws_connections
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_sub(1)
            })
            .ok();
    }

    pub fn connection_rejected(&self) {
        self.ws_connection_rejections_total
            .fetch_add(1, Ordering::Relaxed);
    }

    pub fn message_out(&self) {
        self.ws_messages_out_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn message_in(&self) {
        self.ws_messages_in_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn command(&self) {
        self.ws_commands_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn ping(&self) {
        self.ws_pings_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn pong(&self) {
        self.ws_pongs_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn broadcast(&self, recipients: usize) {
        self.ws_broadcasts_total.fetch_add(1, Ordering::Relaxed);
        self.ws_broadcast_recipients_total
            .fetch_add(recipients as u64, Ordering::Relaxed);
    }

    pub fn send_failure(&self) {
        self.ws_send_failures_total.fetch_add(1, Ordering::Relaxed);
    }

    pub fn render_prometheus(&self) -> String {
        let metrics = [
            (
                "atlsd_realtime_ws_active_connections",
                "gauge",
                "Active WebSocket connections",
                self.active_ws_connections.load(Ordering::Relaxed) as u64,
            ),
            (
                "atlsd_realtime_ws_connections_total",
                "counter",
                "Total accepted WebSocket connections",
                self.ws_connections_total.load(Ordering::Relaxed),
            ),
            (
                "atlsd_realtime_ws_connection_rejections_total",
                "counter",
                "Total rejected WebSocket connection attempts",
                self.ws_connection_rejections_total.load(Ordering::Relaxed),
            ),
            (
                "atlsd_realtime_ws_messages_out_total",
                "counter",
                "Total outbound WebSocket messages",
                self.ws_messages_out_total.load(Ordering::Relaxed),
            ),
            (
                "atlsd_realtime_ws_messages_in_total",
                "counter",
                "Total inbound WebSocket messages",
                self.ws_messages_in_total.load(Ordering::Relaxed),
            ),
            (
                "atlsd_realtime_ws_commands_total",
                "counter",
                "Total inbound WebSocket commands",
                self.ws_commands_total.load(Ordering::Relaxed),
            ),
            (
                "atlsd_realtime_ws_pings_total",
                "counter",
                "Total WebSocket pings sent",
                self.ws_pings_total.load(Ordering::Relaxed),
            ),
            (
                "atlsd_realtime_ws_pongs_total",
                "counter",
                "Total WebSocket pongs received",
                self.ws_pongs_total.load(Ordering::Relaxed),
            ),
            (
                "atlsd_realtime_ws_broadcasts_total",
                "counter",
                "Total broadcast events processed",
                self.ws_broadcasts_total.load(Ordering::Relaxed),
            ),
            (
                "atlsd_realtime_ws_broadcast_recipients_total",
                "counter",
                "Total WebSocket broadcast recipients",
                self.ws_broadcast_recipients_total.load(Ordering::Relaxed),
            ),
            (
                "atlsd_realtime_ws_send_failures_total",
                "counter",
                "Total WebSocket send failures",
                self.ws_send_failures_total.load(Ordering::Relaxed),
            ),
        ];

        let mut body = String::new();
        for (name, kind, help, value) in metrics {
            body.push_str("# HELP ");
            body.push_str(name);
            body.push(' ');
            body.push_str(help);
            body.push('\n');
            body.push_str("# TYPE ");
            body.push_str(name);
            body.push(' ');
            body.push_str(kind);
            body.push('\n');
            body.push_str(name);
            body.push(' ');
            body.push_str(&value.to_string());
            body.push('\n');
        }
        body
    }
}
