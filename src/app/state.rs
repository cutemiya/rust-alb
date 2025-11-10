use std::sync::Arc;
use crate::balancer::balancer::LoadBalancer;
use crate::config::ConfigManager;
use crate::limiter::limiter::RateLimiter;

pub type AppState = Arc<SharedState>;

#[derive(Clone)]
pub struct SharedState {
    pub config_manager: ConfigManager,
    pub load_balancer: LoadBalancer,
    pub rate_limiter: RateLimiter,
}

impl SharedState {
    pub fn new(config_manager: ConfigManager, load_balancer: LoadBalancer, rate_limiter: RateLimiter) -> Self {
        Self { config_manager, load_balancer, rate_limiter }
    }
}