//! # Data Models
//! 
//! ADVANTAGE: All types are known at compile time
//! ADVANTAGE: Serde derives generate zero-overhead serialization
//! ADVANTAGE: Validation is declarative and compile-time checked

pub mod config;
pub mod transaction;
pub mod request;
pub mod response;

pub use config::Config;
pub use transaction::{Transaction, TransactionStatus, NewTransaction};
pub use request::PurchaseRequest;
pub use response::{PurchaseResponse, TransactionListResponse, ErrorResponse};
