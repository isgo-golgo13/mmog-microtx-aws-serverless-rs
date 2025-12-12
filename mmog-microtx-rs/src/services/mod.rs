//! # Services
//! 
//! ADVANTAGE: Clear separation of concerns
//! ADVANTAGE: Services are typed and injectable

pub mod database;
pub mod payment;

pub use database::PostgresDatabase;
pub use payment::PaymentService;
