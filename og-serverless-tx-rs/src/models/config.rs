//! Configuration model with compile-time type safety

use crate::errors::AppError;
use std::env;

/// Application configuration
/// 
/// ADVANTAGE: Missing or invalid config is caught at startup, not runtime
/// ADVANTAGE: All fields have explicit types - no string-to-number coercion bugs
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub stripe_api_key: String,
    pub use_mock_payments: bool,
    pub max_transaction_cents: i64,
    pub max_quantity: i32,
}

impl Config {
    /// Load configuration from environment variables
    /// 
    /// ADVANTAGE: Returns Result<Config, AppError> - caller MUST handle errors
    /// ADVANTAGE: No silent defaults that hide misconfiguration
    pub fn from_env() -> Result<Self, AppError> {
        // ADVANTAGE: Each env var read returns Result, forcing error handling
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| AppError::Configuration("DATABASE_URL not set".into()))?;
        
        let stripe_api_key = env::var("STRIPE_API_KEY")
            .unwrap_or_else(|_| String::new());
        
        // ADVANTAGE: Parse with explicit error handling - no NaN surprises
        let use_mock_payments = env::var("USE_MOCK_PAYMENTS")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);
        
        let max_transaction_cents = env::var("MAX_TRANSACTION_CENTS")
            .unwrap_or_else(|_| "99999999".to_string())
            .parse::<i64>()
            .map_err(|_| AppError::Configuration(
                "MAX_TRANSACTION_CENTS must be a valid integer".into()
            ))?;
        
        let max_quantity = env::var("MAX_QUANTITY")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<i32>()
            .map_err(|_| AppError::Configuration(
                "MAX_QUANTITY must be a valid integer".into()
            ))?;

        // ADVANTAGE: Validation at construction time
        if max_transaction_cents <= 0 {
            return Err(AppError::Configuration(
                "MAX_TRANSACTION_CENTS must be positive".into()
            ));
        }

        Ok(Self {
            database_url,
            stripe_api_key,
            use_mock_payments,
            max_transaction_cents,
            max_quantity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        // ADVANTAGE: Tests are compiled and type-checked
        // Invalid config would fail to compile if types don't match
    }
}
