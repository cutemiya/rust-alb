use std::sync::Arc;
use crate::api::Server;
use crate::config::ConfigManager;
use crate::app::state::{AppState, SharedState};

pub mod state;
pub struct App {
    state: AppState,
    server: Server,
}

impl App {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config_manager = ConfigManager::new("config.yaml".parse().unwrap());
        let state = Arc::new(SharedState::new(config_manager));

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