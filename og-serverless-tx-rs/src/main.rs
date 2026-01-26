//! # MMO Game Micro-transaction Lambda API - Rust Version
//!
//! ## ADVANTAGES OVER NODE.JS:
//! 
//! 1. **Compile-time type safety** - Errors caught at build time, not runtime
//! 2. **Zero-cost abstractions** - Strategy pattern with no runtime overhead
//! 3. **Memory safety without GC** - No garbage collection pauses
//! 4. **Blazing fast cold starts** - Native binary, no interpreter startup
//! 5. **Smaller memory footprint** - No V8 heap, no node_modules
//! 6. **True async/await** - Tokio's M:N green threads vs Node's single-threaded event loop
//! 7. **Pattern matching** - Exhaustive matching prevents forgotten edge cases
//! 8. **Result/Option types** - No null/undefined surprises
//! 9. **Ownership system** - Prevents data races at compile time
//! 10. **Fearless concurrency** - Safe parallel processing

use lambda_http::{run, service_fn, Body, Error, Request, Response};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod errors;
mod handlers;
mod models;
mod services;
mod strategies;

use handlers::router::Router;
use services::{database::PostgresDatabase, payment::PaymentService};
use strategies::payment::{StripePaymentStrategy, MockPaymentStrategy};

/// Application state - shared across Lambda invocations (warm starts)
/// 
/// ADVANTAGE: Arc<T> provides thread-safe reference counting
/// ADVANTAGE: State is explicitly typed and cannot be accidentally mutated
struct AppState {
    router: Router,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // ADVANTAGE: Structured logging with compile-time format strings
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .json()
        .with_target(false)
        .with_current_span(false)
        .init();

    info!("Initializing MMO Microtransaction Lambda (Rust)");

    // ADVANTAGE: Configuration validated at startup, not per-request
    let config = models::config::Config::from_env()?;
    
    // ADVANTAGE: Database pool created once, reused across warm invocations
    let db = Arc::new(PostgresDatabase::new(&config.database_url).await?);
    
    // ADVANTAGE: Strategy pattern with compile-time polymorphism
    // The concrete strategy is selected at startup, not per-request
    let payment_strategy: Arc<dyn strategies::payment::PaymentStrategy> = 
        if config.use_mock_payments {
            info!("Using mock payment strategy");
            Arc::new(MockPaymentStrategy::new())
        } else {
            info!("Using Stripe payment strategy");
            Arc::new(StripePaymentStrategy::new(&config.stripe_api_key))
        };
    
    let payment_service = Arc::new(PaymentService::new(payment_strategy));
    
    // ADVANTAGE: Router is statically typed - all routes validated at compile time
    let router = Router::new(db, payment_service);
    let state = Arc::new(AppState { router });

    // ADVANTAGE: Lambda runtime is a thin wrapper, not a full interpreter
    run(service_fn(|event: Request| {
        let state = Arc::clone(&state);
        async move { handle_request(event, state).await }
    }))
    .await
}

/// Handle incoming HTTP request
/// 
/// ADVANTAGE: Request and Response types are fully typed
/// ADVANTAGE: Pattern matching ensures all HTTP methods are handled
/// ADVANTAGE: Error propagation with ? operator - no try/catch nesting
async fn handle_request(
    request: Request,
    state: Arc<AppState>,
) -> Result<Response<Body>, Error> {
    // ADVANTAGE: Structured logging with typed fields
    info!(
        method = %request.method(),
        path = %request.uri().path(),
        "Processing request"
    );

    // ADVANTAGE: Router returns strongly-typed Response
    let response = state.router.route(request).await;
    
    Ok(response)
}
