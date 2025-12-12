//! # HTTP Handlers
//! 
//! ADVANTAGE: Handlers are strongly typed
//! ADVANTAGE: Request/Response types are known at compile time

pub mod router;
pub mod purchase;
pub mod transactions;
pub mod health;

pub use router::Router;
