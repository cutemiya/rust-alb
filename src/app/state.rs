use std::sync::Arc;
use crate::config::ConfigManager;

pub type AppState = Arc<SharedState>;

#[derive(Clone)]
pub struct SharedState {
    pub config_manager: ConfigManager,
}

impl SharedState {
    pub fn new(config_manager: ConfigManager) -> Self {
        Self { config_manager }
    }
}