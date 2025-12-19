//! Rate-Limited HTTP Client for POD Provider APIs
//!
//! This module provides a rate-limited HTTP client wrapper that respects
//! provider API rate limits and handles retries with exponential backoff.

use governor::{Quota, RateLimiter, state::NotKeyed, clock::DefaultClock, middleware::NoOpMiddleware};
use reqwest::{Client, RequestBuilder, Response};
use std::num::NonZeroU32;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use tracing::{debug, warn};

use crate::providers::traits::ProviderError;

/// Rate-limited HTTP client for API requests
pub struct RateLimitedClient {
    /// Inner HTTP client
    client: Client,

    /// Rate limiter (requests per minute)
    limiter: RateLimiter<NotKeyed, governor::state::InMemoryState, DefaultClock, NoOpMiddleware>,

    /// Configured rate limit
    rate_limit_per_minute: u32,

    /// Remaining requests (from API response headers)
    remaining_requests: AtomicU32,

    /// Default timeout for requests
    timeout: Duration,
}

impl RateLimitedClient {
    /// Create a new rate-limited client
    ///
    /// # Arguments
    /// * `rate_limit_per_minute` - Maximum requests allowed per minute
    pub fn new(rate_limit_per_minute: u32) -> Self {
        // Ensure at least 1 request per minute
        let rate = NonZeroU32::new(rate_limit_per_minute.max(1)).unwrap();

        // Create quota: rate requests per 60 seconds
        let quota = Quota::per_minute(rate);
        let limiter = RateLimiter::direct(quota);

        // Create HTTP client with reasonable defaults
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(10)
            .user_agent("r-image-magic/1.0")
            .build()
            .expect("Failed to create HTTP client");

        RateLimitedClient {
            client,
            limiter,
            rate_limit_per_minute,
            remaining_requests: AtomicU32::new(rate_limit_per_minute),
            timeout: Duration::from_secs(30),
        }
    }

    /// Create a new rate-limited client with custom timeout
    pub fn with_timeout(rate_limit_per_minute: u32, timeout: Duration) -> Self {
        let mut client = Self::new(rate_limit_per_minute);
        client.timeout = timeout;
        client
    }

    /// Get remaining requests in current rate limit window
    pub fn remaining_requests(&self) -> Option<u32> {
        let remaining = self.remaining_requests.load(Ordering::Relaxed);
        if remaining > 0 {
            Some(remaining)
        } else {
            None
        }
    }

    /// Build a GET request
    pub fn get(&self, url: &str) -> RateLimitedRequestBuilder {
        RateLimitedRequestBuilder {
            client: self,
            builder: self.client.get(url),
        }
    }

    /// Build a POST request
    pub fn post(&self, url: &str) -> RateLimitedRequestBuilder {
        RateLimitedRequestBuilder {
            client: self,
            builder: self.client.post(url),
        }
    }

    /// Wait for rate limit and execute request
    async fn execute(&self, builder: RequestBuilder) -> Result<Response, ProviderError> {
        // Wait for rate limit permit
        self.limiter.until_ready().await;

        debug!("Executing rate-limited request");

        // Execute request
        let response = builder.send().await?;

        // Update remaining requests from response headers
        if let Some(remaining) = response
            .headers()
            .get("X-RateLimit-Remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok())
        {
            self.remaining_requests.store(remaining, Ordering::Relaxed);
        }

        // Check for rate limit response
        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok())
                .unwrap_or(60);

            warn!(retry_after_secs = retry_after, "Rate limited by provider");

            return Err(ProviderError::RateLimited {
                retry_after_secs: retry_after,
            });
        }

        Ok(response)
    }

    /// Execute request with retries and exponential backoff
    pub async fn execute_with_retry(
        &self,
        builder: RequestBuilder,
        max_retries: u32,
    ) -> Result<Response, ProviderError> {
        let mut last_error = None;
        let mut backoff = Duration::from_millis(500);

        for attempt in 0..=max_retries {
            if attempt > 0 {
                debug!(attempt, backoff_ms = backoff.as_millis(), "Retrying request");
                tokio::time::sleep(backoff).await;
                backoff = (backoff * 2).min(Duration::from_secs(30));
            }

            // Clone the builder for retry (we need to rebuild each time)
            let result = self.execute(builder.try_clone().unwrap()).await;

            match result {
                Ok(response) => return Ok(response),
                Err(ProviderError::RateLimited { retry_after_secs }) => {
                    // For rate limits, wait the specified time instead of backoff
                    if attempt < max_retries {
                        tokio::time::sleep(Duration::from_secs(retry_after_secs)).await;
                    }
                    last_error = Some(ProviderError::RateLimited { retry_after_secs });
                }
                Err(e) => {
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| ProviderError::Internal("Unknown error".to_string())))
    }
}

impl Clone for RateLimitedClient {
    fn clone(&self) -> Self {
        // Create a new client with the same rate limit
        // Each clone shares the same underlying rate limiter would require Arc,
        // but for simplicity we create independent clients
        Self::new(self.rate_limit_per_minute)
    }
}

/// Request builder wrapper that enforces rate limiting
pub struct RateLimitedRequestBuilder<'a> {
    client: &'a RateLimitedClient,
    builder: RequestBuilder,
}

impl<'a> RateLimitedRequestBuilder<'a> {
    /// Add a header to the request
    pub fn header(mut self, key: &str, value: &str) -> Self {
        self.builder = self.builder.header(key, value);
        self
    }

    /// Add JSON body to the request
    pub fn json<T: serde::Serialize + ?Sized>(mut self, json: &T) -> Self {
        self.builder = self.builder.json(json);
        self
    }

    /// Add a bearer token header
    pub fn bearer_auth(mut self, token: &str) -> Self {
        self.builder = self.builder.bearer_auth(token);
        self
    }

    /// Send the request (waits for rate limit)
    pub async fn send(self) -> Result<Response, ProviderError> {
        self.client.execute(self.builder).await
    }

    /// Send with retries
    pub async fn send_with_retry(self, max_retries: u32) -> Result<Response, ProviderError> {
        self.client.execute_with_retry(self.builder, max_retries).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limited_client_creation() {
        let client = RateLimitedClient::new(120);
        assert_eq!(client.rate_limit_per_minute, 120);
    }

    #[test]
    fn test_remaining_requests() {
        let client = RateLimitedClient::new(100);
        assert_eq!(client.remaining_requests(), Some(100));
    }
}
