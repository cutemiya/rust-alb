use crate::app::state::AppState;
use crate::models::dto::{ErrorResponse, HealthCheckResponse, SuccessResponse};
use crate::models::enums::HealthStatus::Healthy;
use crate::models::enums::HttpError::{
    BackendRateLimitExceeded, GlobalRateLimitExceeded, NoBackendAvailable,
};
use axum::Json;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::IntoResponse;
use serde_json::{Value, json};

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config_manager.get_config().await;
    Json(json!(HealthCheckResponse::new(
        Healthy,
        config.backends.len(),
        format!("{:?}", config.strategy)
    )))
}

pub async fn handle_request(State(state): State<AppState>, request: Request) -> impl IntoResponse {
    let (parts, body) = request.into_parts();

    let path = parts.uri.path();
    let path_after_proxy = path.strip_prefix("/proxy").unwrap_or("");
    let method = parts.method.clone();
    let headers = parts.headers.clone();

    let client_ip = headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let config = state.config_manager.get_config().await;

    if let Some(global_rl) = &config.global_rate_limit {
        if !state
            .rate_limiter
            .check_rate_limit(
                &format!("global_{}", client_ip),
                global_rl.requests_per_second,
                global_rl.burst_size,
            )
            .await
        {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(json!(ErrorResponse::new(
                    GlobalRateLimitExceeded,
                    StatusCode::TOO_MANY_REQUESTS,
                    ""
                ))),
            );
        }
    }

    let backend = match state.load_balancer.select_backend().await {
        Some(backend) => backend,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!(ErrorResponse::new(
                    NoBackendAvailable,
                    StatusCode::SERVICE_UNAVAILABLE,
                    ""
                ))),
            );
        }
    };

    if let Some(limiter) = &backend.rate_limiter {
        if let Some(backend_config) = config.backends.get(&backend.id) {
            if let Some(rate_limit) = &backend_config.rate_limit {
                if !limiter
                    .check_rate_limit(
                        &backend.id,
                        rate_limit.requests_per_second,
                        rate_limit.burst_size,
                    )
                    .await
                {
                    return (
                        StatusCode::TOO_MANY_REQUESTS,
                        Json(json!(ErrorResponse::new(
                            BackendRateLimitExceeded,
                            StatusCode::TOO_MANY_REQUESTS,
                            ""
                        ))),
                    );
                }
            }
        }
    }

    state.load_balancer.increment_connections(&backend.id).await;
    let result = forward_request(
        &backend.url,
        path_after_proxy,
        method,
        headers,
        body,
        state.config_manager.get_config().await.is_debug,
    )
    .await;
    state.load_balancer.decrement_connections(&backend.id).await;

    result
}

async fn forward_request(
    backend_url: &str,
    path: &str,
    method: Method,
    headers: HeaderMap,
    body: Body,
    is_debug: bool,
) -> (StatusCode, Json<Value>) {
    match do_forward(backend_url, path, method, headers, body, is_debug).await {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(e) => {
            tracing::error!("Failed to forward request: {}", e);
            (
                StatusCode::BAD_GATEWAY,
                Json(json!(ErrorResponse::new(
                    BackendRateLimitExceeded,
                    StatusCode::BAD_GATEWAY,
                    format!("Failed to forward request: {}", e).as_str()
                ))),
            )
        }
    }
}

async fn do_forward(
    backend_url: &str,
    path: &str,
    method: Method,
    headers: HeaderMap,
    body: Body,
    is_debug: bool,
) -> Result<Value, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let mut request_builder = client
        .request(method, &format!("{}{}", backend_url, path))
        .body(axum::body::to_bytes(body, 1024 * 1024).await?);

    for (key, value) in headers.iter() {
        if !key.as_str().eq_ignore_ascii_case("host") {
            request_builder = request_builder.header(key, value);
        }
    }

    let response = request_builder.send().await?;
    let status = response.status();
    let response_raw = response.text().await?;

    let response_data = match serde_json::from_str::<Value>(&response_raw) {
        Ok(json) => json!({"content": json}),
        Err(_) => json!({"content": response_raw}),
    };

    Ok(json!(SuccessResponse::new(
        status.as_u16(),
        response_data,
        if is_debug { backend_url } else { "" }
    )))
}
