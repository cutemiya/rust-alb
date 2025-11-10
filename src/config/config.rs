use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendConfig {
    pub url: String,
    pub weight: u32,
    pub rate_limit: Option<RateLimitConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalancerConfig {
    pub strategy: LoadBalancingStrategy,
    pub backends: HashMap<String, BackendConfig>,
    pub global_rate_limit: Option<RateLimitConfig>,
    pub is_debug: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    WeightedRoundRobin,
    LeastConnections,
}

#[derive(Debug, Clone)]
pub struct ConfigManager {
    config: Arc<RwLock<BalancerConfig>>,
    config_path: String,
}

impl ConfigManager {
    pub fn new(config_path: String) -> Self {
        Self {
            config: Arc::new(RwLock::new(Self::load_config(&config_path))),
            config_path,
        }
    }

    fn load_config(path: &str) -> BalancerConfig {
        let content = fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("Failed to read config file: {}", path));

        serde_yaml::from_str(&content)
            .unwrap_or_else(|_| panic!("Failed to parse config file: {}", path))
    }

    pub async fn get_config(&self) -> BalancerConfig {
        self.config.read().await.clone()
    }

    pub async fn reload_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let new_config = Self::load_config(&self.config_path);
        *self.config.write().await = new_config;
        Ok(())
    }

    pub async fn update_config<F>(&self, updater: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut BalancerConfig),
    {
        let mut config = self.config.write().await;
        updater(&mut config);
        Ok(())
    }
}