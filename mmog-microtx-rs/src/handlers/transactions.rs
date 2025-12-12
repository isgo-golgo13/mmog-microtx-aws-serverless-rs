//! # Transactions Handler
//! 
//! ADVANTAGE: Query parameters are typed
//! ADVANTAGE: Pagination is safe by default

use lambda_http::{Body, Request, Response};
use tracing::{info, error, instrument};
use uuid::Uuid;

use crate::errors::AppError;
use crate::models::TransactionListResponse;
use crate::services::PostgresDatabase;
use super::router::json_response;

/// Handle get transactions request
#[instrument(skip(request, db))]
pub async fn handle_get_transactions(
    request: Request,
    db: &PostgresDatabase,
    player_id_str: &str,
) -> Response<Body> {
    match get_transactions(request, db, player_id_str).await {
        Ok(response) => json_response(200, &response),
        Err(e) => {
            error!(error = %e, "Get transactions failed");
            e.into_response()
        }
    }
}

async fn get_transactions(
    request: Request,
    db: &PostgresDatabase,
    player_id_str: &str,
) -> Result<TransactionListResponse, AppError> {
    // ADVANTAGE: UUID parsing is explicit - invalid UUIDs rejected
    let player_id: Uuid = player_id_str
        .parse()
        .map_err(|_| AppError::Validation(format!("Invalid player ID: {}", player_id_str)))?;
    
    // ADVANTAGE: Query params extraction with typed defaults
    let query_string = request.uri().query().unwrap_or("");
    let params = parse_query_params(query_string);
    
    // ADVANTAGE: Limit is bounded - can't request 1 million records
    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<i32>().ok())
        .unwrap_or(100)
        .clamp(1, 1000);
    
    let cursor = params
        .get("cursor")
        .and_then(|v| v.parse::<Uuid>().ok());
    
    info!(
        player_id = %player_id,
        limit = limit,
        cursor = ?cursor,
        "Fetching player transactions"
    );
    
    let transactions = db.get_player_transactions(player_id, limit, cursor).await?;
    
    info!(count = transactions.len(), "Retrieved transactions");
    
    Ok(TransactionListResponse::new(transactions))
}

/// Parse query string into key-value pairs
/// 
/// ADVANTAGE: Simple, safe parsing - no complex regex
fn parse_query_params(query: &str) -> std::collections::HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?;
            let value = parts.next()?;
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}
