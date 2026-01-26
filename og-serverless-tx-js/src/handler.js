/**
 * MMO Game Micro-transaction Lambda Handler - Node.js Version
 * 
 * WEAKNESS #1: No compile-time type safety - runtime errors only
 * WEAKNESS #2: Dynamic typing leads to subtle bugs
 * WEAKNESS #3: Callback/Promise complexity
 * WEAKNESS #4: Cold start overhead from V8 + node_modules
 * WEAKNESS #5: Memory inefficiency due to garbage collection
 * WEAKNESS #6: No zero-cost abstractions
 * WEAKNESS #7: Implicit type coercion causes bugs
 */

const { Client } = require('pg');
const { v4: uuidv4 } = require('uuid');
const Joi = require('joi');

// ============================================================================
// WEAKNESS: No compile-time guarantees on configuration
// A typo here won't be caught until runtime
// ============================================================================
const DB_CONFIG = {
  host: process.env.DB_HOST,
  port: parseInt(process.env.DB_PORT) || 5432,  // WEAKNESS: NaN if invalid
  database: process.env.DB_NAME,
  user: process.env.DB_USER,
  password: process.env.DB_PASSWORD,
  ssl: { rejectUnauthorized: false },
  connectionTimeoutMillis: 5000,
  query_timeout: 10000,
};

// ============================================================================
// WEAKNESS: Joi validation happens at RUNTIME, not compile time
// Schema errors won't be caught until the function is invoked
// ============================================================================
const purchaseSchema = Joi.object({
  player_id: Joi.string().uuid().required(),
  item_id: Joi.string().required(),
  item_name: Joi.string().max(255).required(),
  price_cents: Joi.number().integer().min(1).max(99999999).required(),
  currency: Joi.string().length(3).uppercase().required(),
  quantity: Joi.number().integer().min(1).max(100).default(1),
  metadata: Joi.object().optional(),
});

// ============================================================================
// Transaction status enum - WEAKNESS: Just strings, no compile-time checking
// ============================================================================
const TransactionStatus = {
  PENDING: 'pending',
  COMPLETED: 'completed',
  FAILED: 'failed',
  REFUNDED: 'refunded',
};

// ============================================================================
// WEAKNESS: Global mutable state - can cause issues in Lambda warm starts
// Connection pooling is tricky and error-prone in Lambda
// ============================================================================
let cachedClient = null;

/**
 * Get database client - WEAKNESS: Connection management is manual and error-prone
 */
async function getDbClient() {
  // WEAKNESS: Race condition possible here in concurrent invocations
  if (cachedClient && !cachedClient._ending) {
    try {
      await cachedClient.query('SELECT 1');
      return cachedClient;
    } catch (e) {
      cachedClient = null;
    }
  }
  
  // WEAKNESS: No connection pool - each instance manages its own connection
  cachedClient = new Client(DB_CONFIG);
  await cachedClient.connect();
  return cachedClient;
}

/**
 * Process micro-transaction purchase
 * 
 * WEAKNESS: No generic type parameters - everything is 'any'
 * WEAKNESS: Error handling is verbose and repetitive
 * WEAKNESS: No ownership/borrowing - data can be mutated anywhere
 */
async function processPurchase(purchaseData) {
  const client = await getDbClient();
  
  try {
    // WEAKNESS: Begin transaction - manual management required
    await client.query('BEGIN');
    
    const transactionId = uuidv4();
    const now = new Date().toISOString();
    
    // WEAKNESS: SQL injection possible if not careful (pg does parameterize, but...)
    // WEAKNESS: No compile-time SQL validation
    const insertQuery = `
      INSERT INTO microtransactions (
        transaction_id,
        player_id,
        item_id,
        item_name,
        price_cents,
        currency,
        quantity,
        status,
        metadata,
        created_at,
        updated_at
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
      RETURNING *
    `;
    
    // WEAKNESS: Parameter ordering is error-prone - no named parameters
    const values = [
      transactionId,
      purchaseData.player_id,
      purchaseData.item_id,
      purchaseData.item_name,
      purchaseData.price_cents,
      purchaseData.currency,
      purchaseData.quantity,
      TransactionStatus.PENDING,
      JSON.stringify(purchaseData.metadata || {}),
      now,
      now,
    ];
    
    const result = await client.query(insertQuery, values);
    
    // WEAKNESS: Simulated payment processing - no type safety
    const paymentResult = await processPayment({
      amount: purchaseData.price_cents,
      currency: purchaseData.currency,
      playerId: purchaseData.player_id,
    });
    
    // WEAKNESS: Type coercion - paymentResult.success could be truthy string
    if (paymentResult.success) {
      await client.query(
        'UPDATE microtransactions SET status = $1, updated_at = $2 WHERE transaction_id = $3',
        [TransactionStatus.COMPLETED, new Date().toISOString(), transactionId]
      );
    } else {
      await client.query(
        'UPDATE microtransactions SET status = $1, updated_at = $2 WHERE transaction_id = $3',
        [TransactionStatus.FAILED, new Date().toISOString(), transactionId]
      );
    }
    
    await client.query('COMMIT');
    
    // WEAKNESS: No structural guarantee on return type
    return {
      transactionId,
      status: paymentResult.success ? TransactionStatus.COMPLETED : TransactionStatus.FAILED,
      item: {
        id: purchaseData.item_id,
        name: purchaseData.item_name,
        quantity: purchaseData.quantity,
      },
      payment: {
        amount: purchaseData.price_cents,
        currency: purchaseData.currency,
      },
    };
  } catch (error) {
    // WEAKNESS: error is 'any' type - no guarantees on structure
    await client.query('ROLLBACK');
    throw error;
  }
}

/**
 * Simulated payment processor
 * WEAKNESS: No interface/trait enforcement - duck typing
 */
async function processPayment(paymentData) {
  // WEAKNESS: Simulated delay - blocking event loop potential
  await new Promise(resolve => setTimeout(resolve, 50));
  
  // WEAKNESS: Magic numbers, no validation
  const success = paymentData.amount < 100000; // Simulate: under $1000 always succeeds
  
  return {
    success,
    processorId: uuidv4(),
    timestamp: new Date().toISOString(),
    // WEAKNESS: Inconsistent structure - sometimes has error, sometimes doesn't
    ...(success ? {} : { error: 'Payment declined' }),
  };
}

/**
 * Get player's transaction history
 * WEAKNESS: No pagination by default - could return massive datasets
 */
async function getPlayerTransactions(playerId, limit = 100) {
  const client = await getDbClient();
  
  // WEAKNESS: limit parameter could be tampered with
  const result = await client.query(
    `SELECT * FROM microtransactions 
     WHERE player_id = $1 
     ORDER BY created_at DESC 
     LIMIT $2`,
    [playerId, limit]
  );
  
  return result.rows;
}

/**
 * Main Lambda handler
 * 
 * WEAKNESS: No compile-time event type validation
 * WEAKNESS: Error responses are ad-hoc
 * WEAKNESS: Cold starts are slow due to requiring all modules
 */
exports.handler = async (event, context) => {
  // WEAKNESS: Prevent Lambda from waiting for event loop
  context.callbackWaitsForEmptyEventLoop = false;
  
  console.log('Event received:', JSON.stringify(event));
  
  try {
    // WEAKNESS: event.body could be undefined, null, or non-string
    const body = typeof event.body === 'string' 
      ? JSON.parse(event.body) 
      : event.body;
    
    // WEAKNESS: event.httpMethod vs event.requestContext.http.method inconsistency
    const method = event.httpMethod || event.requestContext?.http?.method;
    const path = event.path || event.requestContext?.http?.path;
    
    // WEAKNESS: Manual routing - error prone, no type safety
    if (method === 'POST' && path === '/purchase') {
      // WEAKNESS: Validation at runtime only
      const { error, value } = purchaseSchema.validate(body);
      
      if (error) {
        return {
          statusCode: 400,
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({
            error: 'Validation failed',
            details: error.details.map(d => d.message),
          }),
        };
      }
      
      const result = await processPurchase(value);
      
      return {
        statusCode: 201,
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(result),
      };
    }
    
    if (method === 'GET' && path?.startsWith('/transactions/')) {
      // WEAKNESS: Manual path parsing - regex or split, both error prone
      const playerId = path.split('/')[2];
      
      // WEAKNESS: No UUID validation here - easy to forget
      if (!playerId) {
        return {
          statusCode: 400,
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ error: 'Player ID required' }),
        };
      }
      
      const transactions = await getPlayerTransactions(playerId);
      
      return {
        statusCode: 200,
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ transactions }),
      };
    }
    
    // WEAKNESS: Easy to forget a route
    return {
      statusCode: 404,
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ error: 'Not found' }),
    };
    
  } catch (error) {
    // WEAKNESS: error could be anything - string, Error, object
    console.error('Handler error:', error);
    
    return {
      statusCode: 500,
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        error: 'Internal server error',
        // WEAKNESS: Potentially leaking stack traces in production
        message: process.env.NODE_ENV === 'development' 
          ? error.message 
          : 'An unexpected error occurred',
      }),
    };
  }
};

// ============================================================================
// Health check endpoint
// WEAKNESS: No graceful degradation
// ============================================================================
exports.healthHandler = async (event, context) => {
  context.callbackWaitsForEmptyEventLoop = false;
  
  try {
    const client = await getDbClient();
    await client.query('SELECT 1');
    
    return {
      statusCode: 200,
      body: JSON.stringify({ status: 'healthy', timestamp: new Date().toISOString() }),
    };
  } catch (error) {
    return {
      statusCode: 503,
      body: JSON.stringify({ status: 'unhealthy', error: error.message }),
    };
  }
};
