use std::time::Duration;
use tokio::time::sleep;
use tracing::{warn, debug};

use crate::utils::error::{AppError, Result};

pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub exponential_base: f32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            exponential_base: 2.0,
        }
    }
}

pub async fn retry_with_backoff<F, T, Fut>(
    config: RetryConfig,
    operation_name: &str,
    mut operation: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut delay = config.initial_delay;

    loop {
        attempt += 1;
        debug!("Attempting {} (attempt {}/{})", operation_name, attempt, config.max_attempts);

        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!("{} succeeded after {} attempts", operation_name, attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                if attempt >= config.max_attempts {
                    warn!("{} failed after {} attempts: {}", operation_name, attempt, e);
                    return Err(e);
                }

                // Check if error is retryable
                if !is_retryable_error(&e) {
                    warn!("{} failed with non-retryable error: {}", operation_name, e);
                    return Err(e);
                }

                warn!(
                    "{} failed (attempt {}/{}): {}. Retrying in {:?}",
                    operation_name, attempt, config.max_attempts, e, delay
                );

                sleep(delay).await;

                // Calculate next delay with exponential backoff
                delay = Duration::from_secs_f32(
                    (delay.as_secs_f32() * config.exponential_base).min(config.max_delay.as_secs_f32())
                );
            }
        }
    }
}

fn is_retryable_error(error: &AppError) -> bool {
    matches!(
        error,
        AppError::Network(_) | 
        AppError::RateLimit { .. } | 
        AppError::SessionExpired |
        AppError::CloudflareBlock
    )
}

pub async fn retry_with_fixed_delay<F, T, Fut>(
    max_attempts: u32,
    delay: Duration,
    operation_name: &str,
    mut operation: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let config = RetryConfig {
        max_attempts,
        initial_delay: delay,
        max_delay: delay,
        exponential_base: 1.0,
    };

    retry_with_backoff(config, operation_name, operation).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let result = retry_with_fixed_delay(3, Duration::from_millis(10), "test", || async {
            Ok::<_, AppError>(42)
        })
        .await
        .unwrap();

        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result = retry_with_fixed_delay(3, Duration::from_millis(10), "test", || {
            let attempts = attempts_clone.clone();
            async move {
                let attempt = attempts.fetch_add(1, Ordering::SeqCst);
                if attempt < 2 {
                    Err(AppError::Internal(
                        anyhow::anyhow!("timeout")
                    ))
                } else {
                    Ok(42)
                }
            }
        })
        .await
        .unwrap();

        assert_eq!(result, 42);
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_max_attempts_exceeded() {
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result = retry_with_fixed_delay(2, Duration::from_millis(10), "test", || {
            let attempts = attempts_clone.clone();
            async move {
                attempts.fetch_add(1, Ordering::SeqCst);
                Err::<i32, _>(AppError::Internal(
                    anyhow::anyhow!("timeout")
                ))
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }
}