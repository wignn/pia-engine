use std::future::Future;
use std::panic::AssertUnwindSafe;
use std::time::Duration;

use futures_util::FutureExt;
use tokio::time;
use tracing::{debug, warn};

const MAX_BACKOFF_MULTIPLIER: u32 = 8;

pub async fn run_scheduled<F, Fut>(name: &str, interval: Duration, f: F)
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send,
{
    let mut consecutive_failures = 0u32;
    run_with_failure_tracking(name, &f, &mut consecutive_failures).await;

    let mut ticker = time::interval(interval);
    ticker.tick().await;

    loop {
        ticker.tick().await;

        if consecutive_failures > 0 {
            let multiplier = (1u32 << consecutive_failures.min(3)).min(MAX_BACKOFF_MULTIPLIER);
            let delay = interval.saturating_mul(multiplier);
            warn!(
                pipeline = name,
                consecutive_failures,
                backoff_multiplier = multiplier,
                ?delay,
                "pipeline backoff active"
            );
            time::sleep(delay).await;
        }

        run_with_failure_tracking(name, &f, &mut consecutive_failures).await;
    }
}

async fn run_with_failure_tracking<F, Fut>(name: &str, f: &F, consecutive_failures: &mut u32)
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send,
{
    match safe_run(name, f).await {
        Ok(()) => *consecutive_failures = 0,
        Err(()) => *consecutive_failures = consecutive_failures.saturating_add(1),
    }
}

async fn safe_run<F, Fut>(name: &str, f: &F) -> Result<(), ()>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send,
{
    let start = std::time::Instant::now();
    let result = AssertUnwindSafe(f()).catch_unwind().await;
    let elapsed = start.elapsed();

    match result {
        Ok(()) => {
            debug!(pipeline = name, ?elapsed, "pipeline tick completed");
            Ok(())
        }
        Err(_) => {
            warn!(pipeline = name, ?elapsed, "pipeline tick panicked");
            Err(())
        }
    }
}
