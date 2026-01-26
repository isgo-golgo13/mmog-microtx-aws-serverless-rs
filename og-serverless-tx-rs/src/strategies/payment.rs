//! # Payment Strategy Implementations
//! 
//! ADVANTAGE: async-trait provides zero-cost async method dispatch
//! ADVANTAGE: Each strategy is a separate type with its own state
//! ADVANTAGE: Strategies can be swapped at compile time or runtime
//! ADVANTAGE: Mock strategy enables testing without external dependencies

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn, instrument};
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

/// Payment request data
/// 
/// ADVANTAGE: Immutable by default - prevents accidental mutation
/// ADVANTAGE: Clone is explicit, not implicit copying
#[derive(Debug, Clone)]
pub struct PaymentRequest {
    pub amount_cents: i64,
    pub currency: String,
    pub player_id: Uuid,
    pub transaction_id: Uuid,
    pub idempotency_key: String,
}

/// Payment result from processor
/// 
/// ADVANTAGE: Result type forces handling of both success and failure
/// ADVANTAGE: Fields are typed - no checking if processor_id exists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResult {
    pub success: bool,
    pub processor_id: String,
    pub processor_response: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
}

impl PaymentResult {
    /// Create successful payment result
    pub fn success(processor_id: impl Into<String>) -> Self {
        Self {
            success: true,
            processor_id: processor_id.into(),
            processor_response: None,
            error_code: None,
            error_message: None,
        }
    }
    
    /// Create failed payment result
    pub fn failure(
        processor_id: impl Into<String>,
        error_code: impl Into<String>,
        error_message: impl Into<String>,
    ) -> Self {
        Self {
            success: false,
            processor_id: processor_id.into(),
            processor_response: None,
            error_code: Some(error_code.into()),
            error_message: Some(error_message.into()),
        }
    }
}

// ============================================================================
// STRATEGY TRAIT
// ============================================================================

/// Payment processing strategy trait
/// 
/// ADVANTAGE: Trait defines the contract at compile time
/// ADVANTAGE: Send + Sync bounds ensure thread safety
/// ADVANTAGE: async_trait enables async methods in traits
/// 
/// In Node.js, you'd have:
/// ```javascript
/// // No compile-time contract enforcement
/// // Duck typing - hope the object has these methods
/// class PaymentStrategy {
///   async processPayment(request) { throw "Not implemented"; }
///   async refundPayment(processorId) { throw "Not implemented"; }
/// }
/// ```
#[async_trait]
pub trait PaymentStrategy: Send + Sync {
    /// Process a payment
    /// 
    /// ADVANTAGE: Return type is guaranteed - no undefined/null surprises
    async fn process_payment(&self, request: PaymentRequest) -> AppResult<PaymentResult>;
    
    /// Refund a payment
    async fn refund_payment(&self, processor_id: &str, amount_cents: i64) -> AppResult<PaymentResult>;
    
    /// Get strategy name for logging
    fn name(&self) -> &'static str;
}

// ============================================================================
// STRIPE PAYMENT STRATEGY
// ============================================================================

/// Stripe payment processor strategy
/// 
/// ADVANTAGE: API key is stored securely in struct, not global state
/// ADVANTAGE: Each instance can have different configuration
pub struct StripePaymentStrategy {
    api_key: String,
    // In production, you'd have a reqwest::Client here
}

impl StripePaymentStrategy {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
        }
    }
}

#[async_trait]
impl PaymentStrategy for StripePaymentStrategy {
    /// Process payment via Stripe
    /// 
    /// ADVANTAGE: #[instrument] provides automatic tracing
    /// ADVANTAGE: All error paths return typed errors
    #[instrument(skip(self, request), fields(strategy = "stripe"))]
    async fn process_payment(&self, request: PaymentRequest) -> AppResult<PaymentResult> {
        info!(
            amount = request.amount_cents,
            currency = %request.currency,
            player_id = %request.player_id,
            "Processing Stripe payment"
        );
        
        // Validate amount before processing
        if request.amount_cents <= 0 {
            return Err(AppError::Payment("Amount must be positive".into()));
        }
        
        if request.amount_cents > 99_999_999 {
            return Err(AppError::Payment("Amount exceeds maximum".into()));
        }
        
        // Simulate Stripe API call
        // In production: use reqwest to call Stripe API
        tokio::time::sleep(Duration::from_millis(50)).await;
        
        // Simulate success for amounts under $1000
        let processor_id = format!("pi_{}", Uuid::new_v4().to_string().replace("-", "")[..24].to_string());
        
        if request.amount_cents < 100_000 {
            info!(processor_id = %processor_id, "Payment successful");
            Ok(PaymentResult::success(processor_id))
        } else {
            warn!(processor_id = %processor_id, "Payment declined - amount too high");
            Ok(PaymentResult::failure(
                processor_id,
                "card_declined",
                "Your card was declined. Please try a different payment method.",
            ))
        }
    }
    
    #[instrument(skip(self), fields(strategy = "stripe"))]
    async fn refund_payment(&self, processor_id: &str, amount_cents: i64) -> AppResult<PaymentResult> {
        info!(
            processor_id = %processor_id,
            amount = amount_cents,
            "Processing Stripe refund"
        );
        
        // Simulate refund
        tokio::time::sleep(Duration::from_millis(30)).await;
        
        let refund_id = format!("re_{}", Uuid::new_v4().to_string().replace("-", "")[..24].to_string());
        Ok(PaymentResult::success(refund_id))
    }
    
    fn name(&self) -> &'static str {
        "stripe"
    }
}

// ============================================================================
// MOCK PAYMENT STRATEGY (for testing)
// ============================================================================

/// Mock payment processor for testing
/// 
/// ADVANTAGE: Same interface as real processor - tests are realistic
/// ADVANTAGE: No network calls - fast unit tests
/// ADVANTAGE: Deterministic behavior for reliable testing
pub struct MockPaymentStrategy {
    /// Simulate failure rate (0.0 - 1.0)
    failure_rate: f64,
    /// Simulated processing delay
    delay: Duration,
}

impl MockPaymentStrategy {
    pub fn new() -> Self {
        Self {
            failure_rate: 0.0,
            delay: Duration::from_millis(10),
        }
    }
    
    /// Create mock with specific failure rate
    pub fn with_failure_rate(failure_rate: f64) -> Self {
        Self {
            failure_rate: failure_rate.clamp(0.0, 1.0),
            delay: Duration::from_millis(10),
        }
    }
}

impl Default for MockPaymentStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PaymentStrategy for MockPaymentStrategy {
    #[instrument(skip(self, request), fields(strategy = "mock"))]
    async fn process_payment(&self, request: PaymentRequest) -> AppResult<PaymentResult> {
        info!(
            amount = request.amount_cents,
            "Processing mock payment"
        );
        
        // Simulate processing time
        tokio::time::sleep(self.delay).await;
        
        let processor_id = format!("mock_{}", Uuid::new_v4());
        
        // Deterministic "failure" based on player_id for testing
        // This allows predictable test scenarios
        let should_fail = request.player_id.as_bytes()[0] as f64 / 255.0 < self.failure_rate;
        
        if should_fail {
            Ok(PaymentResult::failure(
                processor_id,
                "mock_decline",
                "Mock payment declined for testing",
            ))
        } else {
            Ok(PaymentResult::success(processor_id))
        }
    }
    
    #[instrument(skip(self), fields(strategy = "mock"))]
    async fn refund_payment(&self, processor_id: &str, _amount_cents: i64) -> AppResult<PaymentResult> {
        tokio::time::sleep(self.delay).await;
        
        let refund_id = format!("mock_refund_{}", Uuid::new_v4());
        Ok(PaymentResult::success(refund_id))
    }
    
    fn name(&self) -> &'static str {
        "mock"
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_strategy_success() {
        let strategy = MockPaymentStrategy::new();
        
        let request = PaymentRequest {
            amount_cents: 1000,
            currency: "USD".to_string(),
            player_id: Uuid::new_v4(),
            transaction_id: Uuid::new_v4(),
            idempotency_key: Uuid::new_v4().to_string(),
        };
        
        let result = strategy.process_payment(request).await.unwrap();
        
        // ADVANTAGE: We know exactly what fields exist
        assert!(result.success);
        assert!(result.processor_id.starts_with("mock_"));
        assert!(result.error_code.is_none());
    }

    #[tokio::test]
    async fn test_strategy_polymorphism() {
        // ADVANTAGE: Different strategies, same interface
        let strategies: Vec<Box<dyn PaymentStrategy>> = vec![
            Box::new(MockPaymentStrategy::new()),
            Box::new(StripePaymentStrategy::new("sk_test_xxx")),
        ];
        
        for strategy in strategies {
            // ADVANTAGE: Compiler knows this method exists
            let name = strategy.name();
            assert!(!name.is_empty());
        }
    }
}
