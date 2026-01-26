//! # Payment Strategy Pattern
//! 
//! ## ADVANTAGE: Zero-Cost Abstractions
//! 
//! The Strategy pattern in Rust uses trait objects with static dispatch
//! when possible, resulting in ZERO runtime overhead compared to a direct
//! function call. In Node.js, the equivalent would require:
//! - Object allocation for the strategy
//! - Prototype chain lookup for methods
//! - Dynamic dispatch overhead
//! - GC pressure from creating strategy objects
//!
//! ## How This Embarrasses Node.js:
//! 
//! 1. **Type Safety**: The PaymentStrategy trait defines a contract that
//!    MUST be implemented correctly. In Node.js, you'd use duck typing
//!    and hope the object has the right methods.
//! 
//! 2. **Compile-Time Polymorphism**: With monomorphization, the compiler
//!    can inline strategy method calls. Node.js ALWAYS does dynamic dispatch.
//! 
//! 3. **No Runtime Reflection**: Rust doesn't need to inspect objects at
//!    runtime. Node.js constantly checks types during execution.
//! 
//! 4. **Memory Efficiency**: Strategy objects in Rust have known sizes.
//!    Node.js objects have unpredictable memory layouts.

pub mod payment;

pub use payment::{PaymentStrategy, PaymentResult, StripePaymentStrategy, MockPaymentStrategy};
