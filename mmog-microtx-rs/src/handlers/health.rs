//! # Health Handler

use lambda_http::{Body, Response};
use tracing::{info, warn};

use crate::models::response::{HealthResponse, HealthStatus, ComponentHealth};
use crate::services::PostgresDatabase;
use super::router::json_response;

/// Handle health check
pub async fn handle_health(db: &PostgresDatabase) -> Response<Body> {
    let timestamp = chrono::Utc::now().to_rfc3339();
    
    // Check database health
    let db_health = match db.health_check().await {
        Ok(latency) => {
            info!(latency_ms = latency.as_millis(), "Database healthy");
            ComponentHealth {
                status: HealthStatus::Healthy,
                latency_ms: Some(latency.as_millis() as u64),
            }
        }
        Err(e) => {
            warn!(error = %e, "Database unhealthy");
            ComponentHealth {
                status: HealthStatus::Unhealthy,
                latency_ms: None,
            }
        }
    };
    
    let overall_status = match db_health.status {
        HealthStatus::Healthy => HealthStatus::Healthy,
        HealthStatus::Degraded => HealthStatus::Degraded,
        HealthStatus::Unhealthy => HealthStatus::Unhealthy,
    };
    
    let response = HealthResponse {
        status: overall_status,
        timestamp,
        database: Some(db_health),
    };
    
    let status_code = match response.status {
        HealthStatus::Healthy => 200,
        HealthStatus::Degraded => 200,
        HealthStatus::Unhealthy => 503,
    };
    
    json_response(status_code, &response)
}
