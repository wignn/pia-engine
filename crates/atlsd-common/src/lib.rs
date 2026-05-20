// Shared utilities, configuration helpers, types, and errors for ATLSD.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
