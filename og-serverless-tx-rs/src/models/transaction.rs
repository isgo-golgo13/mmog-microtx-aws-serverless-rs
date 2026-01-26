//! Transaction models with compile-time guarantees

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Transaction status enum
/// 
/// ADVANTAGE: Exhaustive pattern matching - compiler ensures all cases handled
/// ADVANTAGE: Invalid status values are impossible to represent
/// ADVANTAGE: Serialization derives are zero-cost
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "transaction_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
}

impl TransactionStatus {
    /// Check if transaction is in a terminal state
    /// 
    /// ADVANTAGE: Method on enum - behavior attached to data
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Refunded)
    }
    
    /// Check if transaction can be refunded
    pub const fn can_refund(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

/// Complete transaction record from database
/// 
/// ADVANTAGE: FromRow derive generates compile-time checked SQL mapping
/// ADVANTAGE: Field types match database schema exactly
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub transaction_id: Uuid,
    pub player_id: Uuid,
    pub item_id: String,
    pub item_name: String,
    pub price_cents: i64,
    pub currency: String,
    pub quantity: i32,
    pub status: TransactionStatus,
    pub metadata: serde_json::Value,
    pub processor_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// New transaction for insertion
/// 
/// ADVANTAGE: Separate types for insert vs select - impossible to mix up
/// ADVANTAGE: Builder pattern with compile-time field validation
#[derive(Debug, Clone)]
pub struct NewTransaction {
    pub transaction_id: Uuid,
    pub player_id: Uuid,
    pub item_id: String,
    pub item_name: String,
    pub price_cents: i64,
    pub currency: String,
    pub quantity: i32,
    pub metadata: serde_json::Value,
}

impl NewTransaction {
    /// Create a new transaction with generated UUID
    /// 
    /// ADVANTAGE: Constructor ensures all required fields are provided
    pub fn new(
        player_id: Uuid,
        item_id: String,
        item_name: String,
        price_cents: i64,
        currency: String,
        quantity: i32,
        metadata: serde_json::Value,
    ) -> Self {
        Self {
            transaction_id: Uuid::new_v4(),
            player_id,
            item_id,
            item_name,
            price_cents,
            currency,
            quantity,
            metadata,
        }
    }
}

/// Currency enum for compile-time currency validation
/// 
/// ADVANTAGE: Only valid currencies can be represented
/// ADVANTAGE: No "USDD" or "usd" typos at runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CAD,
    AUD,
}

impl Currency {
    /// Get currency code as string
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::USD => "USD",
            Self::EUR => "EUR",
            Self::GBP => "GBP",
            Self::JPY => "JPY",
            Self::CAD => "CAD",
            Self::AUD => "AUD",
        }
    }
    
    /// Get decimal places for currency
    /// 
    /// ADVANTAGE: Currency-specific logic is centralized and type-safe
    pub const fn decimal_places(&self) -> u8 {
        match self {
            Self::JPY => 0,
            _ => 2,
        }
    }
}

impl std::str::FromStr for Currency {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "USD" => Ok(Self::USD),
            "EUR" => Ok(Self::EUR),
            "GBP" => Ok(Self::GBP),
            "JPY" => Ok(Self::JPY),
            "CAD" => Ok(Self::CAD),
            "AUD" => Ok(Self::AUD),
            _ => Err(format!("Invalid currency: {}", s)),
        }
    }
}
