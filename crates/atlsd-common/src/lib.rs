pub mod circuit_breaker;
pub mod config;
pub mod db;
pub mod error;
pub mod ipc;
pub mod net;
pub mod util;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
