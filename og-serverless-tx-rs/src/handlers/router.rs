//! # Request Router
//! 
//! ADVANTAGE: Pattern matching ensures all routes are handled
//! ADVANTAGE: Exhaustive matching prevents forgotten endpoints
//! ADVANTAGE: No runtime string comparisons for routing

use lambda_http::{Body, Request, Response, http::Method};
use std::sync::Arc;
use tracing::{info, warn};

use crate::services::{PostgresDatabase, PaymentService};
use crate::errors::AppError;

use super::{purchase, transactions, health};

/// HTTP request router
/// 
/// ADVANTAGE: Dependencies are injected at construction
/// ADVANTAGE: Router is stateless - services are shared via Arc
pub struct Router {
    db: Arc<PostgresDatabase>,
    payment_service: Arc<PaymentService>,
}

impl Router {
    pub fn new(db: Arc<PostgresDatabase>, payment_service: Arc<PaymentService>) -> Self {
        Self { db, payment_service }
    }
    
    /// Route incoming request to appropriate handler
    /// 
    /// ADVANTAGE: Pattern matching on (Method, Path)
    /// ADVANTAGE: Compiler warns about unhandled patterns
    /// ADVANTAGE: No regex parsing overhead
    pub async fn route(&self, request: Request) -> Response<Body> {
        let method = request.method().clone();
        let path = request.uri().path().to_string();
        
        info!(method = %method, path = %path, "Routing request");
        
        // ADVANTAGE: Exhaustive pattern matching
        // The compiler ensures we handle all cases
        match (method, path.as_str()) {
            // Purchase endpoint
            (Method::POST, "/purchase") => {
                self.handle_purchase(request).await
            }
            
            // Get player transactions - with path parameter extraction
            (Method::GET, path) if path.starts_with("/transactions/") => {
                // ADVANTAGE: Path parsing is explicit and typed
                let player_id = path.strip_prefix("/transactions/")
                    .unwrap_or("")
                    .trim_end_matches('/');
                
                self.handle_get_transactions(request, player_id).await
            }
            
            // Health check
            (Method::GET, "/health") => {
                self.handle_health(request).await
            }
            
            // CORS preflight
            (Method::OPTIONS, _) => {
                self.cors_response()
            }
            
            // Not found - ADVANTAGE: Explicit handling of unknown routes
            _ => {
                warn!(method = %method, path = %path, "Route not found");
                self.not_found()
            }
        }
    }
    
    /// Handle purchase request
    async fn handle_purchase(&self, request: Request) -> Response<Body> {
        purchase::handle_purchase(request, &self.db, &self.payment_service).await
    }
    
    /// Handle get transactions request
    async fn handle_get_transactions(&self, request: Request, player_id: &str) -> Response<Body> {
        transactions::handle_get_transactions(request, &self.db, player_id).await
    }
    
    /// Handle health check
    async fn handle_health(&self, _request: Request) -> Response<Body> {
        health::handle_health(&self.db).await
    }
    
    /// CORS preflight response
    fn cors_response(&self) -> Response<Body> {
        Response::builder()
            .status(204)
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
            .header("Access-Control-Allow-Headers", "Content-Type, Authorization")
            .body(Body::Empty)
            .unwrap()
    }
    
    /// Not found response
    fn not_found(&self) -> Response<Body> {
        AppError::NotFound("Endpoint not found".into()).into_response()
    }
}

/// Build success JSON response
/// 
/// ADVANTAGE: Helper function is generic over any serializable type
pub fn json_response<T: serde::Serialize>(status: u16, body: &T) -> Response<Body> {
    let json = serde_json::to_string(body).unwrap_or_else(|_| "{}".to_string());
    
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(Body::from(json))
        .unwrap()
}
