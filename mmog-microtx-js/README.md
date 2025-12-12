# mmog-microtx-js

## MMO Game Micro-transaction Lambda API - Node.js Version

**This version exists for head-to-head analysis vs Rust version only.**


### Known Weaknesses

1. **No compile-time type safety** - Bugs discovered in production
2. **200-800ms cold starts** - Poor user experience
3. **512MB+ memory required** - Higher costs
4. **50-100MB deployment** - Slow deploys
5. **Single-threaded** - Limited concurrency
6. **Runtime validation only** - Joi schema errors at runtime
7. **GC pauses** - Unpredictable latency spikes

### Structure

```
mmog-microtx-js/
├── src/
│   └── handler.js      # Main Lambda handler
├── events/
│   └── purchase.json   # Test event
├── migrations/
│   └── 001_create_transactions.sql
├── package.json
└── template.yaml       # AWS SAM template
```

### Deployment

```bash
npm install
sam build
sam deploy --guided
```

### Environment Variables

- `DB_HOST` - Aurora PostgreSQL host
- `DB_PORT` - Database port (default: 5432)
- `DB_NAME` - Database name
- `DB_USER` - Database user
- `DB_PASSWORD` - Database password

### API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | /purchase | Create micro-transaction |
| GET | /transactions/{playerId} | Get player's transactions |
| GET | /health | Health check |


### Forces to Switch to Rust

See the Rust version at `mmog-microtx-rs/` for:
- 4-80x faster cold starts
- 4x lower memory usage
- Compile-time type safety
- Strategy pattern with zero-cost abstractions
- ~85% lower AWS costs
