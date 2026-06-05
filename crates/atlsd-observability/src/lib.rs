use tracing_subscriber::{fmt, EnvFilter};

pub fn init_tracing(service_name: &str, log_level: &str) {
    let log_level = match log_level.to_uppercase().as_str() {
        "DEBUG" => "debug",
        "WARN" | "WARNING" => "warn",
        "ERROR" => "error",
        "TRACE" => "trace",
        _ => "info",
    };
    let env_filter = EnvFilter::new(format!("{}={},tower_http=debug", service_name, log_level));
    fmt()
        .json()
        .with_env_filter(env_filter)
        .with_target(true)
        .with_thread_ids(false)
        .init();
}

pub fn init() {
    init_tracing("atlsd", "info");
}
