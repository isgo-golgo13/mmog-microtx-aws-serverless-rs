//! Response models with guaranteed structure
//! 
//! ADVANTAGE: Response structure is compile-time guaranteed
//! ADVANTAGE: No accidental missing fields or wrong types

use serde::Serialize;
use uuid::Uuid;

use super::{Transaction, TransactionStatus};

/// Successful purchase response
/// 
/// ADVANTAGE: All fields required - no partial responses
/// ADVANTAGE: Serialize derive generates optimal JSON output
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PurchaseResponse {
    pub transaction_id: Uuid,
    pub status: TransactionStatus,
    pub item: ItemInfo,
    pub payment: PaymentInfo,
    pub created_at: String,
}

/// Item information in response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemInfo {
    pub id: String,
    pub name: String,
    pub quantity: i32,
}

/// Payment information in response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentInfo {
    pub amount_cents: i64,
    pub currency: String,
    pub processor_id: Option<String>,
}

impl PurchaseResponse {
    /// Create response from transaction
    /// 
    /// ADVANTAGE: Type system ensures all required data is provided
    pub fn from_transaction(tx: &Transaction, processor_id: Option<String>) -> Self {
        Self {
            transaction_id: tx.transaction_id,
            status: tx.status,
            item: ItemInfo {
                id: tx.item_id.clone(),
                name: tx.item_name.clone(),
                quantity: tx.quantity,
            },
            payment: PaymentInfo {
                amount_cents: tx.price_cents,
                currency: tx.currency.clone(),
                processor_id,
            },
            created_at: tx.created_at.to_rfc3339(),
        }
    }
}

/// Transaction list response
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionListResponse {
    pub transactions: Vec<Transaction>,
    pub count: usize,
    pub next_cursor: Option<Uuid>,
}

impl TransactionListResponse {
    pub fn new(transactions: Vec<Transaction>) -> Self {
        let next_cursor = transactions.last().map(|t| t.transaction_id);
        let count = transactions.len();
        Self {
            transactions,
            count,
            next_cursor,
        }
    }
}

/// Error response
/// 
/// ADVANTAGE: Error structure is consistent and typed
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub details: Vec<String>,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            code: None,
            details: Vec::new(),
        }
    }
    
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
    
    pub fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = details;
        self
    }
}

/// Health check response
#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<ComponentHealth>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}
