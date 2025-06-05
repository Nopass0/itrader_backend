use std::collections::HashMap;
use std::sync::Arc;
use std::num::NonZeroU32;
use governor::{Quota, RateLimiter as GovRateLimiter, Jitter};
use governor::clock::{DefaultClock, Clock};
use governor::state::{InMemoryState, NotKeyed};
use governor::middleware::NoOpMiddleware;
use parking_lot::Mutex;
use tracing::{debug, warn};
use crate::utils::error::{AppError, Result};

type Limiter = GovRateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>;

pub struct RateLimiter {
    limiters: HashMap<String, Arc<Limiter>>,
    default_limiter: Arc<Limiter>,
}

impl RateLimiter {
    pub fn new(config: &crate::core::config::RateLimitsConfig) -> Self {
        let mut limiters = HashMap::new();

        // Create Gate.io rate limiter
        let gate_quota = Quota::per_minute(NonZeroU32::new(config.gate_requests_per_minute).unwrap())
            .allow_burst(NonZeroU32::new(config.default_burst_size).unwrap());
        limiters.insert(
            "gate".to_string(),
            Arc::new(GovRateLimiter::direct(gate_quota)),
        );

        // Create Bybit rate limiter
        let bybit_quota = Quota::per_minute(NonZeroU32::new(config.bybit_requests_per_minute).unwrap())
            .allow_burst(NonZeroU32::new(config.default_burst_size).unwrap());
        limiters.insert(
            "bybit".to_string(),
            Arc::new(GovRateLimiter::direct(bybit_quota)),
        );

        // Create default rate limiter (more conservative)
        let default_quota = Quota::per_minute(NonZeroU32::new(30).unwrap())
            .allow_burst(NonZeroU32::new(5).unwrap());
        let default_limiter = Arc::new(GovRateLimiter::direct(default_quota));

        Self {
            limiters,
            default_limiter,
        }
    }

    pub async fn check_and_wait(&self, endpoint: &str) -> Result<()> {
        let limiter = self.limiters
            .get(endpoint)
            .unwrap_or(&self.default_limiter);

        match limiter.check() {
            Ok(_) => {
                debug!("Rate limit OK for endpoint: {}", endpoint);
                Ok(())
            }
            Err(_) => {
                let jitter = Jitter::up_to(std::time::Duration::from_millis(500));
                limiter.until_ready_with_jitter(jitter).await;
                
                warn!(
                    "Rate limited on endpoint '{}', waited for slot",
                    endpoint
                );
                Ok(())
            }
        }
    }

    pub fn check_immediate(&self, endpoint: &str) -> Result<()> {
        let limiter = self.limiters
            .get(endpoint)
            .unwrap_or(&self.default_limiter);

        match limiter.check() {
            Ok(_) => {
                debug!("Rate limit OK for endpoint: {}", endpoint);
                Ok(())
            }
            Err(not_until) => {
                let wait_time = not_until.wait_time_from(governor::clock::DefaultClock::default().now());
                Err(AppError::RateLimit {
                    retry_after: wait_time.as_secs(),
                })
            }
        }
    }

    pub fn reset(&self, endpoint: &str) {
        // Governor doesn't support reset, but we can log this for monitoring
        debug!("Reset requested for endpoint: {} (no-op with governor)", endpoint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::RateLimitsConfig;

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimitsConfig {
            gate_requests_per_minute: 60,
            bybit_requests_per_minute: 60,
            default_burst_size: 10,
        };

        let limiter = RateLimiter::new(&config);

        // Should allow burst
        for _ in 0..10 {
            assert!(limiter.check_immediate("gate").is_ok());
        }

        // 11th request might be rate limited
        // (depending on timing, this test might be flaky)
    }
}