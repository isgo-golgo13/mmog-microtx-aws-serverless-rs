//! # Error Types
//! 
//! ADVANTAGE: All error types are known at compile time
//! ADVANTAGE: thiserror generates From implementations automatically
//! ADVANTAGE: Pattern matching on errors is exhaustive
//! ADVANTAGE: Error messages are consistent and typed

use lambda_http::{Body, Response};
use thiserror::Error;

/// Application error type
/// 
/// ADVANTAGE: Each variant has specific, typed data
/// ADVANTAGE: No stringly-typed errors like JS throw "something went wrong"
/// ADVANTAGE: Compiler ensures all error cases are handled
#[derive(Error, Debug)]
pub enum AppError {
    /// Configuration error - missing or invalid environment variables
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    /// Validation error - invalid request data
    #[error("Validation error: {0}")]
    Validation(String),
    
    /// Database error - connection or query failure
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    /// Payment processing error
    #[error("Payment error: {0}")]
    Payment(String),
    
    /// Resource not found
    #[error("Not found: {0}")]
    NotFound(String),
    
    /// Conflict - duplicate transaction, etc.
    #[error("Conflict: {0}")]
    Conflict(String),
    
    /// Rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimited,
    
    /// Internal server error
    #[error("Internal error: {0}")]
    Internal(String),
    
    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl AppError {
    /// Get HTTP status code for error
    /// 
    /// ADVANTAGE: Status codes are deterministic based on error type
    /// ADVANTAGE: Pattern matching ensures all cases handled
    pub const fn status_code(&self) -> u16 {
        match self {
            Self::Validation(_) => 400,
            Self::Configuration(_) => 500,
            Self::Database(_) => 503,
            Self::Payment(_) => 402,
            Self::NotFound(_) => 404,
            Self::Conflict(_) => 409,
            Self::RateLimited => 429,
            Self::Internal(_) => 500,
            Self::Json(_) => 400,
        }
    }
    
    /// Get error code for API response
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::Configuration(_) => "CONFIGURATION_ERROR",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Payment(_) => "PAYMENT_ERROR",
            Self::NotFound(_) => "NOT_FOUND",
            Self::Conflict(_) => "CONFLICT",
            Self::RateLimited => "RATE_LIMITED",
            Self::Internal(_) => "INTERNAL_ERROR",
            Self::Json(_) => "INVALID_JSON",
        }
    }
    
    /// Convert error to HTTP response
    /// 
    /// ADVANTAGE: Error -> Response conversion is guaranteed to succeed
    pub fn into_response(self) -> Response<Body> {
        use crate::models::response::ErrorResponse;
        
        let status = self.status_code();
        let error_response = ErrorResponse::new(self.to_string())
            .with_code(self.error_code());
        
        let body = serde_json::to_string(&error_response)
            .unwrap_or_else(|_| r#"{"error":"Internal error"}"#.to_string());
        
        Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap()  // ADVANTAGE: Builder pattern can't fail with valid inputs
    }
}

/// Result type alias for convenience
/// 
/// ADVANTAGE: Consistent Result type throughout application
pub type AppResult<T> = Result<T, AppError>;

// ============================================================================
// From implementations for automatic error conversion
// ============================================================================

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        let messages: Vec<String> = err
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |e| {
                    format!("{}: {}", field, e.message.as_ref().map(|m| m.to_string()).unwrap_or_else(|| "invalid".to_string()))
                })
            })
            .collect();
        
        Self::Validation(messages.join(", "))
    }
}

impl From<uuid::Error> for AppError {
    fn from(err: uuid::Error) -> Self {
        Self::Validation(format!("Invalid UUID: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        // ADVANTAGE: All error types have deterministic status codes
        assert_eq!(AppError::Validation("test".into()).status_code(), 400);
        assert_eq!(AppError::NotFound("test".into()).status_code(), 404);
        assert_eq!(AppError::RateLimited.status_code(), 429);
    }

    #[test]
    fn test_error_conversion() {
        // ADVANTAGE: Error conversion is type-checked at compile time
        let json_error: AppError = serde_json::from_str::<String>("invalid").unwrap_err().into();
        assert_eq!(json_error.status_code(), 400);
    }
}
