use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct TokenBucket {
    capacity: u32,
    tokens: u32,
    fill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    pub fn new(requests_per_second: u32, burst_size: Option<u32>) -> Self {
        let capacity = burst_size.unwrap_or(requests_per_second);
        Self {
            capacity,
            tokens: capacity,
            fill_rate: requests_per_second as f64 / 1000.0,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_millis() as f64;

        let new_tokens = (elapsed * self.fill_rate) as u32;
        if new_tokens > 0 {
            self.tokens = std::cmp::min(self.tokens + new_tokens, self.capacity);
            self.last_refill = now;
        }
    }

    pub fn try_acquire(&mut self, tokens: u32) -> bool {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimiter {
    buckets: Arc<RwLock<HashMap<String, TokenBucket>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check_rate_limit(
        &self,
        key: &str,
        requests_per_second: u32,
        burst_size: Option<u32>
    ) -> bool {
        let mut buckets = self.buckets.write().await;

        let bucket = buckets.entry(key.to_string()).or_insert_with(|| {
            TokenBucket::new(requests_per_second, burst_size)
        });

        bucket.try_acquire(1)
    }

    pub async fn update_rate_limit(
        &self,
        key: &str,
        requests_per_second: u32,
        burst_size: Option<u32>,
    ) {
        let mut buckets = self.buckets.write().await;
        buckets.insert(key.to_string(), TokenBucket::new(requests_per_second, burst_size));
    }

    pub async fn remove_rate_limit(&self, key: &str) {
        let mut buckets = self.buckets.write().await;
        buckets.remove(key);
    }
}