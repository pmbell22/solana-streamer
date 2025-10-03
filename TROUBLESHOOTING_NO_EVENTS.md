# Troubleshooting: No Events Received

## Problem

When running `focused_arbitrage_example`, you see:
```
Events Processed │ 0
```

No swap events, no logs, nothing happening.

## Possible Causes

### 1. RPC Endpoint Not Streaming (Most Likely)

**Issue:** The public RPC endpoint might not support gRPC streaming or might be rate-limiting you.

**Solution:** Use a dedicated gRPC endpoint:

#### Free Options:
- **Helius** (free tier): https://mainnet.helius-rpc.com
- **Triton** (free tier): Contact for access
- **Your own RPC node**

#### How to Change Endpoint:

```rust
// In your example, change this line:
let grpc = YellowstoneGrpc::new_with_config(
    "https://solana-yellowstone-grpc.publicnode.com:443".to_string(),  // ← Old
    // TO:
    "https://YOUR-HELIUS-KEY.helius-rpc.com:443".to_string(),  // ← New
    None,
    config,
)?;
```

### 2. No Trades Happening for Your Tokens

**Issue:** The specific token pairs you're monitoring might not have active trading.

**Solution:**
- Add more common pairs (SOL/USDC is most active)
- Check if the tokens are actually trading on those DEXes
- Try the `simple_stream_test` example to see if ANY events come through

### 3. Filters Too Restrictive

**Issue:** The combination of transaction filter + account filter might be too narrow.

**Solution:** Start broader, then narrow down.

## Debugging Steps

### Step 1: Test Basic Connectivity

Run the simple test:
```bash
cargo run --example simple_stream_test
```

**Expected:** You should see events within 10-30 seconds
**If nothing:** Your RPC endpoint isn't streaming events → Try different endpoint

### Step 2: Test Without Token Filtering

Modify `focused_arbitrage_example.rs`:

```rust
// Comment out the transaction filter
let transaction_filter = TransactionFilter {
    account_include: vec![], // Empty = all transactions
    account_exclude: vec![],
    account_required: vec![],
};
```

**Expected:** Lots of events (all DEX swaps)
**If nothing:** RPC endpoint issue

### Step 3: Check Specific Program Events

Modify to only watch one program:

```rust
let program_ids = vec![
    JUPITER_AGG_V6_PROGRAM_ID.to_string(),
    // Comment out others for testing
];

let transaction_filter = TransactionFilter {
    account_include: vec![JUPITER_AGG_V6_PROGRAM_ID.to_string()],
    account_exclude: vec![],
    account_required: vec![],
};
```

**Expected:** Jupiter swaps only
**If nothing:** Either RPC issue or Jupiter not active

### Step 4: Verify Token Addresses

Make sure your token addresses are correct:

```bash
# Verify SOL address
echo "So11111111111111111111111111111111111111112"

# Verify USDC address
echo "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
```

These should match what's in your code.

## Quick Fixes

### Fix 1: Use Different RPC Endpoint

The `publicnode.com` endpoint might not support streaming. Try:

1. Sign up for Helius (free tier)
2. Get your API key
3. Update the endpoint:
   ```rust
   "https://mainnet.helius-rpc.com/?api-key=YOUR_KEY".to_string()
   ```

### Fix 2: Simplify Filters

Start with the broadest possible filters:

```rust
// Simplest possible - all Jupiter swaps
let transaction_filter = TransactionFilter {
    account_include: vec![JUPITER_AGG_V6_PROGRAM_ID.to_string()],
    account_exclude: vec![],
    account_required: vec![],
};

let account_filter = AccountFilter {
    account: vec![],
    owner: vec![JUPITER_AGG_V6_PROGRAM_ID.to_string()],
    filters: vec![],
};
```

### Fix 3: Add More Logging

Add debug prints to verify callback is being set up:

```rust
let callback = create_focused_arbitrage_callback(...);

// Add this wrapper for debugging:
let debug_callback = move |event: Box<dyn UnifiedEvent>| {
    println!("🔔 EVENT RECEIVED! Type: {:?}", event.event_type());
    callback(event);
};

// Use debug_callback instead of callback in subscribe
```

## Common RPC Endpoints

| Provider | Endpoint | Free Tier | Notes |
|----------|----------|-----------|-------|
| Helius | `mainnet.helius-rpc.com` | Yes | Best for development |
| Triton | `contact for access` | Limited | High performance |
| QuickNode | `quicknode endpoint` | Trial | Paid |
| Your own node | `localhost:10000` | Yes | Requires setup |

## Expected Behavior

Once working, you should see:

```
Starting subscription...

Monitored tokens (5):
  DezXAZ... (BONK)
  JUPyi... (JUP)
  So111... (SOL)
  EPjFW... (USDC)
  Es9vM... (USDT)

Press Ctrl+C to stop...

================================================

🔵 Jupiter Swap [MONITORED]: SOL -> USDC (1000000 -> 167500000)
📊 Adding quote: Jupiter - 167500000/1000000 = 167.500000 (existing quotes: 0)
🔍 Checking 1 quotes for token pair So111.../EPjFW...
📭 No quotes found for this token pair yet

🟣 Raydium CPMM Swap [MONITORED]: SOL -> USDC (1000000 -> 167400000)
📊 Adding quote: RaydiumCpmm - 167400000/1000000 = 167.400000 (existing quotes: 1)
🔍 Checking 2 quotes for token pair So111.../EPjFW...
💡 Found price difference: Jupiter @ 167.500000 vs RaydiumCpmm @ 167.400000 = 0.06% profit
❌ Below 0.30% threshold
```

## Still Not Working?

If after trying all the above you still see 0 events:

1. **Verify the RPC endpoint supports gRPC Yellowstone streaming**
   - Not all RPC providers support this
   - Most public endpoints don't support it
   - You likely need a paid/dedicated endpoint

2. **Check if the endpoint requires authentication**
   - Some endpoints need API keys in headers
   - Check the provider's documentation

3. **Try the original `arbitrage_detector_example.rs`**
   - If that also shows 0 events, it's definitely an RPC issue
   - If that works but focused doesn't, compare the filters

4. **Check network connectivity**
   ```bash
   # Test basic connectivity
   curl -I https://solana-yellowstone-grpc.publicnode.com
   ```

## Next Steps

1. **Get a working RPC endpoint** - This is #1 priority
2. **Run `simple_stream_test`** to verify connectivity
3. **Gradually add filters** once basic streaming works
4. **Monitor for actual trading activity** on your token pairs

---

**TL;DR:** Most likely you need a different RPC endpoint that supports gRPC streaming. Try Helius free tier.
