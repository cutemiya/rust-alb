use crate::limiter::limiter::RateLimiter;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::config::{BackendConfig, LoadBalancingStrategy};

#[derive(Debug, Clone)]
pub struct Backend {
    pub id: String,
    pub url: String,
    pub weight: u32,
    pub active_connections: Arc<RwLock<u32>>,
    pub rate_limiter: Option<RateLimiter>,
}

#[derive(Debug, Clone)]
pub struct LoadBalancer {
    backends: Arc<RwLock<Vec<Backend>>>,
    strategy: Arc<RwLock<LoadBalancingStrategy>>,
    current_index: Arc<RwLock<usize>>,
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            backends: Arc::new(RwLock::new(Vec::new())),
            strategy: Arc::new(RwLock::new(LoadBalancingStrategy::RoundRobin)),
            current_index: Arc::new(RwLock::new(0)),
        }
    }

    pub async fn update_backends(&self, backends_config: HashMap<String, BackendConfig>) {
        let mut new_backends = Vec::new();

        for (id, config) in backends_config {
            let rate_limiter = config.rate_limit.as_ref().map(|_| {
                RateLimiter::new()
            });

            let backend = Backend {
                id: id.clone(),
                url: config.url,
                weight: config.weight,
                active_connections: Arc::new(RwLock::new(0)),
                rate_limiter,
            };
            new_backends.push(backend);
        }

        *self.backends.write().await = new_backends;
        *self.current_index.write().await = 0;
    }

    pub async fn set_strategy(&self, strategy: LoadBalancingStrategy) {
        *self.strategy.write().await = strategy;
    }

    pub async fn select_backend(&self) -> Option<Backend> {
        let backends = self.backends.read().await;
        if backends.is_empty() {
            return None;
        }

        let strategy = self.strategy.read().await;

        match &*strategy {
            LoadBalancingStrategy::RoundRobin => self.round_robin(&backends).await,
            LoadBalancingStrategy::WeightedRoundRobin => self.weighted_round_robin(&backends).await,
            LoadBalancingStrategy::LeastConnections => self.least_connections(&backends).await,
        }
    }

    async fn round_robin(&self, backends: &[Backend]) -> Option<Backend> {
        let mut index = self.current_index.write().await;
        let backend = backends.get(*index).cloned();

        *index = (*index + 1) % backends.len();
        backend
    }

    async fn weighted_round_robin(&self, backends: &[Backend]) -> Option<Backend> {
        let total_weight: u32 = backends.iter().map(|b| b.weight).sum();

        let mut index = self.current_index.write().await;
        let mut current_weight = 0u32;
        let mut selected = None;

        for _ in 0..total_weight {
            let backend = &backends[*index];
            current_weight += 1;

            if current_weight <= backend.weight {
                selected = Some(backend.clone());
                break;
            }

            current_weight = 0;
            *index = (*index + 1) % backends.len();
        }

        *index = (*index + 1) % backends.len();
        selected
    }

    async fn least_connections(&self, backends: &[Backend]) -> Option<Backend> {
        let mut min_connections = u32::MAX;
        let mut selected_backend = None;

        for backend in backends {
            let connections = *backend.active_connections.read().await;
            if connections < min_connections {
                min_connections = connections;
                selected_backend = Some(backend.clone());
            }
        }

        selected_backend
    }

    pub async fn increment_connections(&self, backend_id: &str) {
        let backends = self.backends.read().await;
        if let Some(backend) = backends.iter().find(|b| b.id == backend_id) {
            *backend.active_connections.write().await += 1;
        }
    }

    pub async fn decrement_connections(&self, backend_id: &str) {
        let backends = self.backends.read().await;
        if let Some(backend) = backends.iter().find(|b| b.id == backend_id) {
            let mut connections = backend.active_connections.write().await;
            *connections = connections.saturating_sub(1);
        }
    }
}