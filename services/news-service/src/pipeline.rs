mod analysis;
mod finnhub;
mod fred;
pub mod r#macro;
mod persistence;
mod rss;
mod scheduler;
pub mod scrape;
mod sources;
mod text;

pub use scheduler::run;
