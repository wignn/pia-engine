use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

/// Minimal Prometheus-compatible metrics registry: named counters with help
/// text, rendered in the text exposition format. Follows the per-service
/// pattern realtime-gateway already uses, so every Rust service can expose
/// `/metrics` without pulling in a metrics dependency tree.
#[derive(Default)]
pub struct MetricsRegistry {
    counters: Mutex<HashMap<String, MetricEntry>>,
}

struct MetricEntry {
    help: String,
    kind: &'static str,
    value: AtomicU64,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers the counter if new; increments it either way. Labels are not
    /// supported on purpose: cardinality belongs in the metric name here.
    pub fn inc(&self, name: &str, help: &str) {
        self.add(name, help, 1);
    }

    pub fn add(&self, name: &str, help: &str, value: u64) {
        let mut counters = self.lock();
        counters
            .entry(name.to_string())
            .or_insert_with(|| MetricEntry::new(help, "counter"))
            .value
            .fetch_add(value, Ordering::Relaxed);
    }

    /// Sets a gauge to an absolute value (e.g. queue backlog).
    pub fn set_gauge(&self, name: &str, help: &str, value: u64) {
        let mut counters = self.lock();
        counters
            .entry(name.to_string())
            .or_insert_with(|| MetricEntry::new(help, "gauge"))
            .value
            .store(value, Ordering::Relaxed);
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, HashMap<String, MetricEntry>> {
        match self.counters.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    /// Current value of a metric (0 when unregistered).
    pub fn get(&self, name: &str) -> u64 {
        self.lock()
            .get(name)
            .map(|entry| entry.value.load(Ordering::Relaxed))
            .unwrap_or(0)
    }

    pub fn render_prometheus(&self) -> String {
        let counters = self.lock();
        let mut names: Vec<&String> = counters.keys().collect();
        names.sort();

        let mut output = String::new();
        for name in names {
            let entry = &counters[name];
            output.push_str(&format!("# HELP {name} {}\n", entry.help));
            output.push_str(&format!("# TYPE {name} {}\n", entry.kind));
            output.push_str(&format!("{name} {}\n", entry.value.load(Ordering::Relaxed)));
        }
        output
    }
}

impl MetricEntry {
    fn new(help: &str, kind: &'static str) -> Self {
        Self {
            help: help.to_string(),
            kind,
            value: AtomicU64::new(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_accumulate_and_register_once() {
        let registry = MetricsRegistry::new();
        registry.inc("atlsd_test_events_total", "Test events.");
        registry.add("atlsd_test_events_total", "Test events.", 2);

        assert_eq!(registry.get("atlsd_test_events_total"), 3);
    }

    #[test]
    fn unregistered_counters_read_zero() {
        let registry = MetricsRegistry::new();
        assert_eq!(registry.get("atlsd_missing_total"), 0);
    }

    #[test]
    fn render_uses_prometheus_text_format() {
        let registry = MetricsRegistry::new();
        registry.inc("atlsd_b_total", "B events.");
        registry.set_gauge("atlsd_a_backlog", "A backlog.", 7);

        let rendered = registry.render_prometheus();
        let a = rendered.find("atlsd_a_backlog").unwrap();
        let b = rendered.find("atlsd_b_total").unwrap();
        assert!(a < b, "metrics render in sorted order");
        assert!(rendered.contains("# TYPE atlsd_b_total counter"));
        assert!(rendered.contains("# TYPE atlsd_a_backlog gauge"));
        assert!(rendered.contains("atlsd_a_backlog 7"));
    }
}
