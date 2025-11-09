use crate::app::state::AppState;
use axum::extract::State;
use axum::Json;
use axum::response::IntoResponse;
use serde_json::json;

pub async fn health_check(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let config = state.config_manager.get_config().await;

    Json(json!({
        "status": "healthy",
        "backends_count": config.backends.len(),
        "strategy": format!("{:?}", config.strategy)
    }))
}