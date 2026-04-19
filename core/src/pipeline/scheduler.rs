use std::future::Future;
use std::time::Duration;
use tokio::time;
use tracing::{error, debug, info};

pub async fn run_scheduled<F, Fut>(name: &str, interval: Duration, f: F)

where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send,
{
    safe_run(name, &f).await;

    let mut ticker = time::interval(interval);
    ticker.tick().await; 

    loop {
        ticker.tick().await;
        safe_run(name, &f).await;
    }
}

async fn safe_run<F, Fut>(name: &str, f: &F)
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send,
{
    let start = std::time::Instant::now();
    f().await;
    let elapsed = start.elapsed();
    debug!(pipeline = name, ?elapsed, "pipeline tick completed");
}
