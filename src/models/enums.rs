use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    #[serde(rename = "undefined")]
    Undefined,
    #[serde(rename = "healthy")]
    Healthy,
    #[serde(rename = "failed")]
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HttpError {
    #[serde(rename = "undefined")]
    Undefined,
    #[serde(rename = "global_rate_limit_exceeded")]
    GlobalRateLimitExceeded,
    #[serde(rename = "no_backend_available")]
    NoBackendAvailable,
    #[serde(rename = "backend_rate_limit_exceeded")]
    BackendRateLimitExceeded,
}