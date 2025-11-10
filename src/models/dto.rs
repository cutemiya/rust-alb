use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::models::enums::{HealthStatus, HttpError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    status: HealthStatus,
    backends_count: usize,
    strategy: String,
}

impl HealthCheckResponse {
    pub fn new(status: HealthStatus, backends_count: usize, strategy: String) -> HealthCheckResponse {
        Self { status, backends_count, strategy }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    error: HttpError,
    status: u16,
    message: String,
}

impl ErrorResponse {
    pub fn new(error: HttpError, status_code: StatusCode, msg: &str) -> Self {
        Self { error, status: status_code.as_u16(), message: msg.to_string() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessResponse {
    status: u16,
    data: Value,
    url: String
}

impl SuccessResponse {
    pub fn new(status: u16, data: Value, url: &str) -> SuccessResponse {
        Self { status, data, url: url.to_string() }
    }
}