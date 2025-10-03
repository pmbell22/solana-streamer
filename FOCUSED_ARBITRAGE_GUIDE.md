# Focused Arbitrage Monitoring Guide

This guide explains how to monitor specific token pairs for arbitrage instead of all pairs across all DEXes.

## Why Focus on Specific Pairs?

**Benefits:**
- ✅ **Less Data** - Dramatically reduces bandwidth usage
- ✅ **Faster Processing** - Only process relevant events
- ✅ **Better Focus** - Monitor liquid, profitable pairs you care about
- ✅ **Lower Costs** - Some RPC providers charge by data volume
- ✅ **More Reliable** - Less likely to miss opportunities in your target pairs

**Trade-offs:**
- ❌ Miss opportunities in pairs you're not monitoring
- ❌ Need to know which pairs/pools to monitor upfront

## Three Approaches

### 1. **Filter by Token Mints (Simplest)**

Subscribe to all transactions involving specific token mints.

**Pros:** Easy to configure, catches all pools
**Cons:** Still gets some irrelevant data

```rust
let token_mints = vec![
    "So11111111111111111111111111111111111111112".to_string(), // SOL
    "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(), // USDC
];

let transaction_filter = TransactionFilter {
    account_include: token_mints,
    account_exclude: vec![],
    account_required: vec![],
};
```

### 2. **Filter by Pool Addresses (Most Efficient)**

Subscribe directly to specific pool accounts.

**Pros:** Minimal data, very focused
**Cons:** Need to find pool addresses first

```rust
let pool_addresses = vec![
    "POOL_ADDRESS_1".to_string(),
    "POOL_ADDRESS_2".to_string(),
];

let account_filter = AccountFilter {
    account: pool_addresses,
    owner: vec![],
    filters: vec![],
};
```

### 3. **Post-Filter in Code (Flexible)**

Subscribe to everything but only process specific pairs.

**Pros:** Can change pairs without resubscribing
**Cons:** Still receives all data

```rust
// In your callback
if monitored_pairs.contains(&(e.source_mint, e.destination_mint)) {
    // Process this event
}
```

## Complete Example

See `examples/focused_arbitrage_example.rs` for a full implementation that:

1. Defines specific token pairs to monitor
2. Filters by token mints at subscription level
3. Post-filters in the callback for extra precision
4. Shows `[MONITORED]` tags on relevant events

## How to Find Pool Addresses

### Option 1: Use DEX APIs

**Raydium:**
```bash
# Get all pools for a token pair
curl 'https://api.raydium.io/v2/main/pairs'
```

**Jupiter:**
```bash
# Jupiter aggregates from multiple DEXes
curl 'https://quote-api.jup.ag/v6/quote?inputMint=SOL&outputMint=USDC&amount=1000000'
```

### Option 2: On-Chain Discovery

Listen to pool creation events and cache them:

```rust
EventType::RaydiumCpmmInitialize => |e| {
    // Cache: pool_address -> (token0_mint, token1_mint)
    pool_cache.insert(e.pool_state, (e.token0_mint, e.token1_mint));
}
```

### Option 3: Use Block Explorers

Search on Solscan/Solana FM for pools trading your desired pairs.

## Example Configuration

```rust
fn get_monitored_token_pairs() -> Vec<TokenPairConfig> {
    let sol = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let usdc = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();

    vec![
        TokenPairConfig {
            name: "SOL/USDC".to_string(),
            token_a: sol,
            token_b: usdc,
            pools: vec![
                PoolInfo {
                    dex: "Raydium CPMM".to_string(),
                    pool_address: Pubkey::from_str("ACTUAL_POOL_ADDRESS").unwrap(),
                },
                // Add more pools...
            ],
        },
    ]
}
```

## Running the Example

```bash
cargo run --example focused_arbitrage_example
```

You should see output like:

```
Monitoring 4 token pairs:
  - SOL/USDC (2 pools)
  - SOL/USDT (1 pools)
  - BONK/SOL (3 pools)
  - JUP/USDC (1 pools)

Subscribing to 7 specific pool accounts
Starting subscription...

Monitored tokens:
  So11111111111111111111111111111111111111112
  EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v
  ...

🔵 Jupiter Swap [MONITORED]: SOL -> USDC (1000000 -> 167500000)
📊 Adding quote: Jupiter - 167500000/1000000 = 167.500000 (existing quotes: 0)
🔍 Checking 1 quotes for token pair So11111.../EPjFWdd...

🟣 Raydium CPMM Swap [MONITORED]: SOL -> USDC (1000000 -> 167300000)
📊 Adding quote: RaydiumCpmm - 167300000/1000000 = 167.300000 (existing quotes: 1)
🔍 Checking 2 quotes for token pair So11111.../EPjFWdd...
💡 Found price difference: Jupiter @ 167.500000 vs RaydiumCpmm @ 167.300000 = 0.12% profit
❌ Below 0.30% threshold
```

## Best Practices

### 1. Start with High-Volume Pairs

Focus on the most liquid pairs with tightest spreads:
- SOL/USDC
- SOL/USDT
- mSOL/SOL
- JUP/USDC
- BONK/SOL

### 2. Monitor Multiple Pools per Pair

Each pair might have multiple pools:
- Raydium CPMM pool
- Raydium CLMM pool (multiple fee tiers)
- Orca Whirlpool
- Phoenix

### 3. Adjust Thresholds by Pair

```rust
// High volume pairs = lower threshold
let sol_usdc_threshold = 0.1; // 0.1%

// Lower volume pairs = higher threshold (more risk)
let obscure_pair_threshold = 1.0; // 1.0%
```

### 4. Consider Transaction Costs

Include gas costs in your profitability calculation:
```rust
// For 1 SOL input:
// - Jupiter swap: ~0.001 SOL
// - Raydium swap: ~0.001 SOL
// Total: ~0.002 SOL = ~0.2% at $200/SOL
```

### 5. Handle Token Decimals

Different tokens have different decimals:
```rust
// SOL: 9 decimals (1 SOL = 1_000_000_000 lamports)
// USDC: 6 decimals (1 USDC = 1_000_000 micro-USDC)
```

Make sure to normalize prices correctly!

## Advanced: Dynamic Pool Discovery

For production systems, you'll want to dynamically discover pools:

```rust
// 1. Listen to initialization events
EventType::RaydiumCpmmInitialize => |e| {
    if is_monitored_pair(e.token0_mint, e.token1_mint) {
        // Add this pool to subscription
        add_pool_to_subscription(e.pool_state);
    }
}

// 2. Periodically query DEX APIs for new pools
tokio::spawn(async move {
    loop {
        discover_new_pools().await;
        tokio::time::sleep(Duration::from_secs(300)).await; // Every 5 min
    }
});
```

## Troubleshooting

### Not seeing any events?

1. Check your token mint addresses are correct
2. Verify pool addresses exist and are active
3. Make sure you're subscribed to the right event types
4. Check if the pools have recent activity (not all pools trade frequently)

### Too many events?

1. Remove low-volume token pairs
2. Use pool-specific filtering instead of mint filtering
3. Increase your profit threshold

### Missing opportunities?

1. Lower your profit threshold (temporarily for testing)
2. Check if you're monitoring all relevant pools for a pair
3. Verify token decimals are handled correctly
4. Check timestamp/quote age settings

## Performance Tips

1. **Use specific pool addresses** instead of mint filters when possible
2. **Cache pool information** to avoid repeated lookups
3. **Process events in parallel** if monitoring many pairs
4. **Set appropriate quote expiry** (30s is usually good)
5. **Monitor metrics** to track event processing latency

## Next Steps

1. Start with 2-3 high-volume pairs
2. Add pool addresses as you discover them
3. Adjust thresholds based on observed spreads
4. Add more pairs once stable
5. Implement dynamic pool discovery for production
