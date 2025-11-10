use std::sync::Arc;
use crate::api::Server;
use crate::config::ConfigManager;
use crate::app::state::{AppState, SharedState};
use crate::balancer::balancer::LoadBalancer;
use crate::limiter::limiter::RateLimiter;

pub mod state;
pub struct App {
    state: AppState,
    server: Server,
}

impl App {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_manager = ConfigManager::new("config.yaml".parse().unwrap());
        let load_balancer = LoadBalancer::new();
        load_balancer.update_backends(config_manager.get_config().await.backends).await;
        load_balancer.set_strategy(config_manager.get_config().await.strategy).await;
        let rate_limiter = RateLimiter::new();
        let state = Arc::new(SharedState::new(config_manager, load_balancer, rate_limiter));

        let server = Server::new(state.clone());

        Ok(Self { state, server })
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        self.server.run().await
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }
}