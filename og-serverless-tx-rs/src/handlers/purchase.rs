//! # Purchase Handler
//! 
//! ADVANTAGE: Request processing is typed end-to-end
//! ADVANTAGE: Error handling with ? operator - no try/catch nesting

use lambda_http::{Body, Request, Response};
use tracing::{info, error, instrument};
use validator::Validate;

use crate::errors::AppError;
use crate::models::{PurchaseRequest, PurchaseResponse, NewTransaction, TransactionStatus};
use crate::services::{PostgresDatabase, PaymentService};
use super::router::json_response;

/// Handle purchase request
/// 
/// ADVANTAGE: Full request pipeline with type safety
/// ADVANTAGE: Each step returns Result - errors bubble up automatically
#[instrument(skip(request, db, payment_service))]
pub async fn handle_purchase(
    request: Request,
    db: &PostgresDatabase,
    payment_service: &PaymentService,
) -> Response<Body> {
    match process_purchase(request, db, payment_service).await {
        Ok(response) => json_response(201, &response),
        Err(e) => {
            error!(error = %e, "Purchase failed");
            e.into_response()
        }
    }
}

/// Process purchase - separated for cleaner error handling
async fn process_purchase(
    request: Request,
    db: &PostgresDatabase,
    payment_service: &PaymentService,
) -> Result<PurchaseResponse, AppError> {
    // STEP 1: Parse request body
    // ADVANTAGE: JSON parsing errors are typed
    let body = request.body();
    let body_str = match body {
        Body::Text(s) => s.clone(),
        Body::Binary(b) => String::from_utf8(b.to_vec())
            .map_err(|_| AppError::Validation("Invalid UTF-8 in body".into()))?,
        Body::Empty => return Err(AppError::Validation("Request body required".into())),
    };
    
    // STEP 2: Deserialize to typed struct
    // ADVANTAGE: Invalid JSON shape fails here, not later
    let purchase_req: PurchaseRequest = serde_json::from_str(&body_str)?;
    
    // STEP 3: Validate request
    // ADVANTAGE: Validation rules are enforced by the type system
    purchase_req.validate()
        .map_err(AppError::from)?;
    
    info!(
        player_id = %purchase_req.player_id,
        item_id = %purchase_req.item_id,
        amount = purchase_req.price_cents,
        "Processing purchase"
    );
    
    // STEP 4: Create transaction record
    let new_tx = NewTransaction::new(
        purchase_req.player_id,
        purchase_req.item_id.clone(),
        purchase_req.item_name.clone(),
        purchase_req.price_cents,
        purchase_req.currency.clone(),
        purchase_req.quantity,
        purchase_req.metadata.clone().unwrap_or(serde_json::Value::Null),
    );
    
    // ADVANTAGE: Transaction ID is generated and typed
    let tx = db.insert_transaction(&new_tx).await?;
    
    // STEP 5: Process payment via strategy
    // ADVANTAGE: Payment service handles strategy selection
    let payment_result = payment_service
        .process_purchase(
            tx.transaction_id,
            tx.player_id,
            tx.price_cents,
            &tx.currency,
        )
        .await?;
    
    // STEP 6: Update transaction status
    let final_status = if payment_result.success {
        TransactionStatus::Completed
    } else {
        TransactionStatus::Failed
    };
    
    let updated_tx = db
        .update_transaction_status(
            tx.transaction_id,
            final_status,
            Some(&payment_result.processor_id),
        )
        .await?;
    
    // STEP 7: Build response
    // ADVANTAGE: Response structure is compile-time guaranteed
    let response = PurchaseResponse::from_transaction(
        &updated_tx,
        Some(payment_result.processor_id),
    );
    
    info!(
        transaction_id = %updated_tx.transaction_id,
        status = ?final_status,
        "Purchase completed"
    );
    
    Ok(response)
}
