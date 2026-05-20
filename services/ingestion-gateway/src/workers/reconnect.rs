use std::time::Duration;

pub struct ReconnectPolicy {
    base_sec: u64,
    max_sec: u64,
    current_sec: u64,
}

impl ReconnectPolicy {
    pub fn new(base_sec: u64, max_sec: u64) -> Self {
        Self {
            base_sec: base_sec.max(1),
            max_sec: max_sec.max(1),
            current_sec: base_sec.max(1),
        }
    }

    pub fn next_delay(&mut self) -> Duration {
        let delay = Duration::from_secs(self.current_sec);
        self.current_sec = (self.current_sec * 2).min(self.max_sec);
        delay
    }
    pub fn reset(&mut self) {
        self.current_sec = self.base_sec;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exponential_growth() {
        let mut p = ReconnectPolicy::new(2, 30);
        assert_eq!(p.next_delay(), Duration::from_secs(2));
        assert_eq!(p.next_delay(), Duration::from_secs(4));
        assert_eq!(p.next_delay(), Duration::from_secs(8));
        assert_eq!(p.next_delay(), Duration::from_secs(16));
        assert_eq!(p.next_delay(), Duration::from_secs(30));
        assert_eq!(p.next_delay(), Duration::from_secs(30));
    }

    #[test]
    fn reset_works() {
        let mut p = ReconnectPolicy::new(5, 300);
        let _ = p.next_delay();
        let _ = p.next_delay();
        p.reset();
        assert_eq!(p.next_delay(), Duration::from_secs(5));
    }
}
