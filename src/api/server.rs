use std::net::SocketAddr;
use axum::Router;
use axum::routing::get;
use crate::api::handlers::health_check;
use crate::app::state::AppState;

pub struct Server {
    router: Router,
    state: AppState,
}

impl Server {
    pub fn new(state: AppState) -> Self {
        let router = Self::build_router(state.clone());

        Self { router, state }
    }

    fn build_router(state: AppState) -> Router {
        Router::new()
            .route("/health", get(health_check))
            .route("/config", get(Self::get_config))
            .with_state(state)
    }

    pub fn with_additional_routes<F>(mut self, router_builder: F) -> Self
    where
        F: FnOnce(Router) -> Router,
    {
        self.router = router_builder(self.router);
        self
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

        tracing::debug!("Server running on {}", addr);

        let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
            .await
            .unwrap();
        tracing::debug!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, self.router).await.unwrap();

        Ok(())
    }

    async fn get_config(
        axum::extract::State(state): axum::extract::State<AppState>,
    ) -> impl axum::response::IntoResponse {
        let config = state.config_manager.get_config().await;
        axum::Json(serde_json::json!({
            "strategy": format!("{:?}", config.strategy),
            "backends": config.backends
        }))
    }
}