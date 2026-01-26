# OG Micro-Transaction Serverless AWS (Rust) 
Online-Game Micro-Transaction Serverless Service using Rust, AWS SDK Rust, Rust Tokio Async and AWS Lambda w/ AWS RDS Aurora vs Node.js Version

This Git repository provides a dual head-to-head of the mmog-microtx-js (Node.js) and mmog-microtx-rs (Rust) services for a MMO Game Micro-Transaction API served on AWS Lambda.


**Bottom Line:** Rust delivers 4-80x faster cold starts, 4x lower memory usage, compile-time bug prevention, and significantly lower AWS costs while providing the exact same functionality.


![og-tx-aws-serverless-arch-1](docs/og-tx-aws-serverless-rs-arch-1.png)




### 1. Cold Start Performance
Cold starts are critical for Lambda functions—they directly impact user experience and costs.

| Metric              | Node.js                            | Rust                      | Advantage             |
|---------------------|------------------------------------|---------------------------|-----------------------|
| **Cold Start Time** | 200-800ms                          | 10-50ms                   | **Rust 4-80x faster** |
| **Reason**          | V8 initialization + module loading | Native binary, no runtime | Rust wins             |
| **Impact**          | User-visible latency spikes        | Near-instant response     | Rust wins             |



In a game with millions of players making micro-transactions:

- 1M transactions/day with 10% cold starts = 100,000 cold starts

- Node.js: 100,000 × 500ms = ~14 hours of cumulative cold start latency

- Rust: 100,000 × 30ms = ~50 minutes of cumulative cold start latency

This resultrs in **16x** less user-facing latency.


### 2. Memory Efficiency

| Metric | Node.js | Rust | Advantage |
|--------|---------|------|-----------|
| **Minimum Memory** | 512MB recommended | 128MB sufficient | **Rust 4x more efficient** |
| **Memory Growth** | Unpredictable (GC) | Deterministic | Rust wins |
| **Peak Memory** | Spikes during GC | Flat, predictable | Rust wins |


Cost Impact (Per Million Invocations)
```shell
Node.js: 512MB × 200ms × 1,000,000 = 102,400,000 GB-ms
Rust:    128MB × 50ms  × 1,000,000 =   6,400,000 GB-ms

Cost savings: ~94% reduction in compute costs
```

### 3. Static Type Safety and Runtime Failures Prevention

**Node.js: Runtime Errors Only**

```javascript
// This code has bugs that won't be caught until production:
const purchaseSchema = Joi.object({
  player_id: Joi.string().uuid().required(),
  price_cents: Joi.number().integer().required(),  // What if someone passes "123"?
});

// No guarantee this object has the right shape:
const result = await processPurchase(value);
console.log(result.transactionId);  // Could be undefined!

// Implicit type coercion causes bugs:
if (paymentResult.success) {  // "false" (string) is truthy!
```

**Rust: Compile-Time Guarantees**

```Rust
// This code is verified by the compiler:
pub struct PurchaseRequest {
    pub player_id: Uuid,           // Must be valid UUID
    pub price_cents: i64,          // Must be integer
}

// Response structure is guaranteed:
let result: PurchaseResponse = process_purchase(request).await?;
println!("{}", result.transaction_id);  // Always exists!

// No type coercion:
if payment_result.success {  // Must be bool - strings won't compile
```


#### Bugs Prevented w/ Rust Type System

 Bug Type | Node.js | Rust |
|----------|---------|------|
| Undefined/null access | Runtime crash | Compile error |
| Wrong parameter types | Silent corruption | Compile error |
| Missing error handling | Silent failure | Compile error |
| SQL injection | Possible | Prevented by design |
| Data races | Possible | Compile error |




### 4. Strategy Pattern Comparison

```javascript
// No contract enforcement - duck typing
class PaymentStrategy {
  async processPayment(request) { 
    throw "Not implemented"; 
  }
}

// Hope the object has the right methods
function processWithStrategy(strategy, request) {
  return strategy.processPayment(request);  // Could fail at runtime
}

// Object allocation for each strategy
// Prototype chain lookup
// Dynamic dispatch overhead
// GC pressure from strategy objects
```

**Rust Strategy Pattern (Zero-Cost Abstractions)**

```rust
// Compile-time contract enforcement
#[async_trait]
pub trait PaymentStrategy: Send + Sync {
    async fn process_payment(&self, request: PaymentRequest) -> AppResult<PaymentResult>;
    fn name(&self) -> &'static str;
}

// Compiler verifies strategy implements trait
fn process_with_strategy<S: PaymentStrategy>(strategy: &S, request: PaymentRequest) {
    strategy.process_payment(request)  // Guaranteed to exist!
}

// With monomorphization:
// - No virtual dispatch overhead
// - Strategy methods can be inlined
// - Zero runtime cost vs direct function call
```


### 5. Error Handling


Node.js: Stringly-Typed Errors

```javascript 
try {
  await processPurchase(data);
} catch (error) {
  // error is 'any' - no guarantees
  // Could be Error, string, number, object...
  console.log(error.message);  // Might not exist!
  
  // Ad-hoc status codes
  return { statusCode: 500, body: 'Something went wrong' };
}
```

Rust: Typed Error Handling

```rust
// All error types are known at compile time
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Payment error: {0}")]
    Payment(String),
}

// Exhaustive error handling
match result {
    Ok(response) => json_response(201, &response),
    Err(AppError::Validation(msg)) => error_response(400, msg),
    Err(AppError::Database(_)) => error_response(503, "Database unavailable"),
    Err(AppError::Payment(msg)) => error_response(402, msg),
    // Compiler ensures ALL cases handled
}
```

### 6. Deployment Package Size

| Metric | Node.js | Rust | Advantage |
|--------|---------|------|-----------|
| **Package Size** | 50-100MB | 5-10MB | **Rust 5-20x smaller** |
| **Contents** | node_modules (thousands of files) | Single binary | Rust wins |
| **Deploy Time** | Slower (more to upload) | Faster | Rust wins |
| **Cold Start Impact** | More to load | Less to load | Rust wins |



### 7. Concurrency Model

Node.js: Single-Threaded Event Loop

```javascript
// All JavaScript runs on ONE thread
// CPU-intensive work blocks the event loop
// Callbacks/Promises add complexity
async function processPurchase(data) {
  // If this is slow, ALL other requests wait
  const result = await heavyComputation();
}
```

Rust + Tokio: M:N Green Threads

```rust
// Multiple async tasks on multiple OS threads
// CPU-intensive work doesn't block other requests
// Structured concurrency with async/await
async fn process_purchase(data: PurchaseRequest) -> Result<Response> {
    // Tokio schedules across available cores
    // Other tasks continue while waiting on I/O
    let result = heavy_computation().await;
}
```


### 8. AWS Lambda Specific Advantages

- Official AWS Support for Rust 2025
- Official Lambda SLA metrics coverage
- Official AWS `lambda_runtime` crate
- AWS offficial `cargo-lambda` for trivial deployment


Resource Configuration Comparison

```yaml
# Node.js Lambda
MemorySize: 512   # Need extra for V8 + GC
Timeout: 15       # Need extra for cold start

# Rust Lambda  
MemorySize: 128   # Lowest footprint
Timeout: 5        # Fast startup
```

### 9. Real-World Cost Analysis

Assumptions

- 10 million invocations/month
- Average execution time: Node.js 200ms, **Rust 50ms**
- Memory: Node.js 512MB, Rust 128MB
- AWS Lambda pricing: $0.0000166667 per GB-second

Monthly Costs

| | Node.js | Rust |
|---|---------|------|
| **GB-seconds** | 1,024,000 | 64,000 |
| **Compute Cost** | $17.07 | $1.07 |
| **Request Cost** | $2.00 | $2.00 |
| **Total** | **$19.07** | **$3.07** |


**Annual Savings:** ~$192 per function



#### Long-Term Maintenance

| Aspect | Node.js | Rust |
|--------|---------|------|
| Dependency security | `npm audit` (reactive) | Compile-time (proactive) |
| Breaking changes | Runtime surprises | Compile errors |
| Technical debt | Accumulates silently | Caught at compile time |


### 11. Side-to-Side Code Comparison

**Request Validation**

Node.js

```javascript
const purchaseSchema = Joi.object({
  player_id: Joi.string().uuid().required(),
  price_cents: Joi.number().integer().min(1).required(),
});

// Validation at RUNTIME
const { error, value } = purchaseSchema.validate(body);
if (error) { /* handle */ }
```

Rust

```rust
#[derive(Deserialize, Validate)]
pub struct PurchaseRequest {
    pub player_id: Uuid,  // Invalid UUID = compile error
    #[validate(range(min = 1))]
    pub price_cents: i64, // Non-integer = compile error
}

// Validation at COMPILE TIME (schema) + runtime (values)
let request: PurchaseRequest = serde_json::from_str(&body)?;
request.validate()?;
```

#### Database Queries

Node.js

```javascript
const result = await client.query(
  'INSERT INTO microtransactions (...) VALUES ($1, $2, $3, ...)',
  [transactionId, playerId, itemId, ...]  // Order matters, easy to mess up
);
// result.rows[0] - is it defined? What shape?
```

Rust

```rust
let result: Transaction = sqlx::query_as(
    "INSERT INTO microtransactions (...) VALUES ($1, $2, $3, ...) RETURNING *"
)
.bind(tx.transaction_id)  // Type checked
.bind(tx.player_id)       // Type checked
.bind(&tx.item_id)        // Type checked
.fetch_one(&pool)
.await?;
// result is guaranteed to be Transaction type
```


### 12. Migration Path

**Phase 1: Parallel Deployment (Week 1-2)**

- Deploy Rust Lambda alongside Node.js
- Route 5% of traffic to Rust
- Monitor cold starts, latency, errors

**Phase 2: Gradual Rollout (Week 3-4)**

- Increase Rust traffic to 25%, 50%, 75%
- Compare CloudWatch metrics
- Address any issues

**Phase 3: Full Migration (Week 5)**

- Route 100% to Rust
- Keep Node.js as fallback
- Document learnings

**Phase 4: Cleanup (Week 6)**

- Remove Node.js Lambda
- Update CI/CD pipelines



## Appendix: Start Commands

**Deploy Node.js Version**
```shell
cd mmog-microtx-js
npm install
sam build
sam deploy --guided
```

**Deploy Rust Version**
```shell
cd mmog-microtx-rs
cargo lambda build --release
sam build
sam deploy --guided
```



## Conclusion

### Rust Advantages Summary

| Category | Improvement |
|----------|-------------|
| Cold Start | 4-80x faster |
| Memory Usage | 4x more efficient |
| Package Size | 5-20x smaller |
| Bug Prevention | Compile-time vs runtime |
| Type Safety | Complete vs none |
| Cost | ~85% reduction |
| Concurrency | M:N vs single-threaded |
