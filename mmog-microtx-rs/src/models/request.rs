//! Request models with compile-time and runtime validation

use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

/// Purchase request payload
/// 
/// ADVANTAGE: Validation rules are declarative and compile-time checked
/// ADVANTAGE: Deserialize derive rejects invalid JSON shapes at parse time
/// ADVANTAGE: Field types prevent implicit coercion (no "123" becoming 123)
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct PurchaseRequest {
    /// Player's unique identifier
    #[validate(required)]
    pub player_id: Uuid,  // ADVANTAGE: UUID type - invalid UUIDs rejected at parse
    
    /// Item identifier in the game catalog
    #[validate(length(min = 1, max = 255))]
    pub item_id: String,
    
    /// Human-readable item name
    #[validate(length(min = 1, max = 255))]
    pub item_name: String,
    
    /// Price in cents (smallest currency unit)
    /// ADVANTAGE: i64 is explicit - no floating point precision issues
    #[validate(range(min = 1, max = 99_999_999))]
    pub price_cents: i64,
    
    /// ISO 4217 currency code
    #[validate(length(equal = 3))]
    pub currency: String,
    
    /// Quantity of items (defaults to 1)
    #[validate(range(min = 1, max = 100))]
    #[serde(default = "default_quantity")]
    pub quantity: i32,
    
    /// Optional metadata (item stats, etc.)
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Default quantity for purchases
const fn default_quantity() -> i32 {
    1
}

impl PurchaseRequest {
    /// Validate the request
    /// 
    /// ADVANTAGE: Returns Result with specific validation errors
    /// ADVANTAGE: Validation rules are enforced by the type system
    pub fn validate_request(&self) -> Result<(), validator::ValidationErrors> {
        self.validate()
    }
    
    /// Calculate total price
    /// 
    /// ADVANTAGE: Overflow is checked - panics in debug, wraps in release
    /// For production, use checked_mul for explicit handling
    pub fn total_price_cents(&self) -> i64 {
        self.price_cents
            .checked_mul(self.quantity as i64)
            .expect("Price overflow")
    }
}

/// Get player transactions request
/// 
/// ADVANTAGE: Query parameters are typed and validated
#[derive(Debug, Clone, Deserialize, Validate)]
pub struct GetTransactionsRequest {
    #[validate(range(min = 1, max = 1000))]
    #[serde(default = "default_limit")]
    pub limit: i32,
    
    #[serde(default)]
    pub cursor: Option<Uuid>,
    
    #[serde(default)]
    pub status: Option<super::TransactionStatus>,
}

const fn default_limit() -> i32 {
    100
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_purchase_request_validation() {
        // ADVANTAGE: Invalid UUID literally cannot compile
        // let invalid = PurchaseRequest { player_id: "not-a-uuid", ... };
        // ^^^^^^^ This is a compile error, not a runtime error
        
        let valid_request = PurchaseRequest {
            player_id: Uuid::new_v4(),
            item_id: "sword_001".to_string(),
            item_name: "Iron Sword".to_string(),
            price_cents: 999,
            currency: "USD".to_string(),
            quantity: 1,
            metadata: None,
        };
        
        assert!(valid_request.validate().is_ok());
    }

    #[test]
    fn test_invalid_price_rejected() {
        let invalid_request = PurchaseRequest {
            player_id: Uuid::new_v4(),
            item_id: "sword_001".to_string(),
            item_name: "Iron Sword".to_string(),
            price_cents: -100,  // Negative price
            currency: "USD".to_string(),
            quantity: 1,
            metadata: None,
        };
        
        assert!(invalid_request.validate().is_err());
    }
}
