//! # Payment Service
//! 
//! ADVANTAGE: Service uses Strategy pattern through trait object
//! ADVANTAGE: Strategy can be swapped without changing service code
//! ADVANTAGE: Testing is easy with mock strategy injection

use std::sync::Arc;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::strategies::payment::{PaymentStrategy, PaymentRequest, PaymentResult};

/// Payment service that delegates to a strategy
/// 
/// ADVANTAGE: Arc allows sharing across async tasks without copying
/// ADVANTAGE: Strategy is determined at construction, not per-call
pub struct PaymentService {
    strategy: Arc<dyn PaymentStrategy>,
}

impl PaymentService {
    /// Create new payment service with strategy
    /// 
    /// ADVANTAGE: Strategy must implement PaymentStrategy trait
    /// ADVANTAGE: dyn PaymentStrategy allows runtime polymorphism when needed
    pub fn new(strategy: Arc<dyn PaymentStrategy>) -> Self {
        info!(strategy = strategy.name(), "Payment service initialized");
        Self { strategy }
    }
    
    /// Process a purchase
    /// 
    /// ADVANTAGE: Input and output types are fully specified
    /// ADVANTAGE: Errors are typed and must be handled
    #[instrument(skip(self), fields(
        strategy = self.strategy.name(),
        transaction_id = %transaction_id,
        amount = amount_cents
    ))]
    pub async fn process_purchase(
        &self,
        transaction_id: Uuid,
        player_id: Uuid,
        amount_cents: i64,
        currency: &str,
    ) -> AppResult<PaymentResult> {
        // Validate inputs
        if amount_cents <= 0 {
            return Err(AppError::Validation("Amount must be positive".into()));
        }
        
        // Create idempotency key from transaction ID
        let idempotency_key = format!("purchase_{}", transaction_id);
        
        let request = PaymentRequest {
            amount_cents,
            currency: currency.to_string(),
            player_id,
            transaction_id,
            idempotency_key,
        };
        
        info!("Delegating to payment strategy");
        
        // ADVANTAGE: Strategy call is just a method call - no reflection
        let result = self.strategy.process_payment(request).await?;
        
        if result.success {
            info!(processor_id = %result.processor_id, "Payment processed successfully");
        } else {
            info!(
                error_code = result.error_code.as_deref().unwrap_or("unknown"),
                "Payment failed"
            );
        }
        
        Ok(result)
    }
    
    /// Process a refund
    #[instrument(skip(self), fields(strategy = self.strategy.name()))]
    pub async fn process_refund(
        &self,
        processor_id: &str,
        amount_cents: i64,
    ) -> AppResult<PaymentResult> {
        if amount_cents <= 0 {
            return Err(AppError::Validation("Refund amount must be positive".into()));
        }
        
        info!(processor_id = %processor_id, amount = amount_cents, "Processing refund");
        
        let result = self.strategy.refund_payment(processor_id, amount_cents).await?;
        
        Ok(result)
    }
    
    /// Get the name of the current strategy
    pub fn strategy_name(&self) -> &'static str {
        self.strategy.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::payment::MockPaymentStrategy;

    #[tokio::test]
    async fn test_payment_service_with_mock() {
        // ADVANTAGE: Mock strategy implements same trait as real strategy
        let mock_strategy = Arc::new(MockPaymentStrategy::new());
        let service = PaymentService::new(mock_strategy);
        
        let result = service.process_purchase(
            Uuid::new_v4(),
            Uuid::new_v4(),
            1000,
            "USD",
        ).await.unwrap();
        
        // ADVANTAGE: Result type is known - all fields accessible
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_validation_error() {
        let mock_strategy = Arc::new(MockPaymentStrategy::new());
        let service = PaymentService::new(mock_strategy);
        
        // ADVANTAGE: Error is typed - we know exactly what to expect
        let result = service.process_purchase(
            Uuid::new_v4(),
            Uuid::new_v4(),
            -100,  // Invalid amount
            "USD",
        ).await;
        
        assert!(matches!(result, Err(AppError::Validation(_))));
    }
}
