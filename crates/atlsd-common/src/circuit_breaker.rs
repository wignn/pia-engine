use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitStateName {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug)]
enum CircuitState {
    Closed { consecutive_failures: u32 },
    Open { opened_at: Instant },
    HalfOpen { successes: u32 },
}

#[derive(Debug)]
pub struct CircuitBreaker {
    state: RwLock<CircuitState>,
    failure_threshold: u32,
    recovery_timeout: Duration,
    half_open_max_successes: u32,
}

impl CircuitBreaker {
    pub fn new(
        failure_threshold: u32,
        recovery_timeout: Duration,
        half_open_max_successes: u32,
    ) -> Self {
        Self {
            state: RwLock::new(CircuitState::Closed {
                consecutive_failures: 0,
            }),
            failure_threshold,
            recovery_timeout,
            half_open_max_successes,
        }
    }

    pub async fn allow_request(&self) -> bool {
        let mut state = self.state.write().await;
        match *state {
            CircuitState::Closed { .. } | CircuitState::HalfOpen { .. } => true,
            CircuitState::Open { opened_at } => {
                if opened_at.elapsed() >= self.recovery_timeout {
                    *state = CircuitState::HalfOpen { successes: 0 };
                    info!("circuit breaker transitioned open -> half_open");
                    true
                } else {
                    false
                }
            }
        }
    }

    pub async fn record_success(&self) {
        let mut state = self.state.write().await;
        match *state {
            CircuitState::Closed {
                ref mut consecutive_failures,
            } => {
                *consecutive_failures = 0;
            }
            CircuitState::HalfOpen { ref mut successes } => {
                *successes += 1;
                if *successes >= self.half_open_max_successes {
                    *state = CircuitState::Closed {
                        consecutive_failures: 0,
                    };
                    info!("circuit breaker transitioned half_open -> closed");
                }
            }
            CircuitState::Open { .. } => {}
        }
    }

    pub async fn record_failure(&self) {
        let mut state = self.state.write().await;
        match *state {
            CircuitState::Closed {
                ref mut consecutive_failures,
            } => {
                *consecutive_failures += 1;
                if *consecutive_failures >= self.failure_threshold {
                    *state = CircuitState::Open {
                        opened_at: Instant::now(),
                    };
                    info!(
                        failure_threshold = self.failure_threshold,
                        "circuit breaker transitioned closed -> open"
                    );
                }
            }
            CircuitState::HalfOpen { .. } => {
                *state = CircuitState::Open {
                    opened_at: Instant::now(),
                };
                info!("circuit breaker transitioned half_open -> open");
            }
            CircuitState::Open { .. } => {}
        }
    }

    pub async fn state_name(&self) -> CircuitStateName {
        match *self.state.read().await {
            CircuitState::Closed { .. } => CircuitStateName::Closed,
            CircuitState::Open { .. } => CircuitStateName::Open,
            CircuitState::HalfOpen { .. } => CircuitStateName::HalfOpen,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn opens_after_threshold_and_recovers() {
        let breaker = CircuitBreaker::new(2, Duration::from_millis(1), 1);

        assert!(breaker.allow_request().await);
        breaker.record_failure().await;
        assert_eq!(breaker.state_name().await, CircuitStateName::Closed);

        breaker.record_failure().await;
        assert_eq!(breaker.state_name().await, CircuitStateName::Open);
        assert!(!breaker.allow_request().await);

        tokio::time::sleep(Duration::from_millis(2)).await;
        assert!(breaker.allow_request().await);
        assert_eq!(breaker.state_name().await, CircuitStateName::HalfOpen);

        breaker.record_success().await;
        assert_eq!(breaker.state_name().await, CircuitStateName::Closed);
    }

    #[tokio::test]
    async fn half_open_failure_reopens_circuit() {
        let breaker = CircuitBreaker::new(1, Duration::from_millis(1), 2);

        breaker.record_failure().await;
        assert_eq!(breaker.state_name().await, CircuitStateName::Open);

        tokio::time::sleep(Duration::from_millis(2)).await;
        assert!(breaker.allow_request().await);
        assert_eq!(breaker.state_name().await, CircuitStateName::HalfOpen);

        breaker.record_failure().await;
        assert_eq!(breaker.state_name().await, CircuitStateName::Open);
        assert!(!breaker.allow_request().await);
    }

    #[tokio::test]
    async fn half_open_requires_configured_success_count() {
        let breaker = CircuitBreaker::new(1, Duration::from_millis(1), 2);

        breaker.record_failure().await;
        tokio::time::sleep(Duration::from_millis(2)).await;
        assert!(breaker.allow_request().await);

        breaker.record_success().await;
        assert_eq!(breaker.state_name().await, CircuitStateName::HalfOpen);

        breaker.record_success().await;
        assert_eq!(breaker.state_name().await, CircuitStateName::Closed);
    }
}
