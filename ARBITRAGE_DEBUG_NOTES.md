# Arbitrage Detector Debug Notes

## Issues Found & Fixed

### 1. ⚠️ **CRITICAL: Raydium CLMM Discriminators Were Wrong (FIXED)**

**The Problem:** Raydium CLMM SwapV2 and other instruction discriminators were incorrect, causing the parser to NEVER match swap instructions!

**Impact:**
- CLMM SwapV2 events were not being parsed at all
- 0 amount values in events
- No arbitrage detection for CLMM pools

**Before (WRONG):**
```rust
pub const SWAP_V2: &[u8] = &[43, 4, 237, 11, 26, 201, 30, 98];
pub const CREATE_POOL: &[u8] = &[233, 146, 209, 142, 207, 104, 64, 188];
pub const INCREASE_LIQUIDITY_V2: &[u8] = &[133, 29, 89, 223, 69, 238, 176, 10];
pub const DECREASE_LIQUIDITY_V2: &[u8] = &[58, 127, 188, 62, 79, 82, 196, 96];
pub const OPEN_POSITION_V2: &[u8] = &[77, 184, 74, 214, 112, 86, 241, 199];
```

**After (FIXED):**
```rust
pub const SWAP_V2: &[u8] = &[114, 113, 45, 226, 179, 239, 106, 225];           // swapV2
pub const CREATE_POOL: &[u8] = &[244, 236, 117, 4, 18, 0, 62, 88];             // createPool
pub const INCREASE_LIQUIDITY_V2: &[u8] = &[67, 78, 196, 105, 211, 25, 62, 252]; // increaseLiquidityV2
pub const DECREASE_LIQUIDITY_V2: &[u8] = &[82, 1, 46, 234, 207, 210, 241, 169]; // decreaseLiquidityV2
pub const OPEN_POSITION_V2: &[u8] = &[218, 45, 162, 175, 86, 17, 83, 121];     // openPositionV2
```

**Root Cause:**
The discriminators were hardcoded with incorrect values instead of being calculated using Anchor's SHA256("global:instruction_name") formula.

**Files Fixed:**
- `src/streaming/event_parser/protocols/raydium_clmm/events.rs:262-278`

---

### 2. ✅ **Opportunities ARE Being Added to the Vector**

The `opportunities` vector in the callback is working correctly. Each event handler can mutate it because:
- The `match_event!` macro expands to sequential if-let-else chains
- Each handler closure captures `opportunities` mutably from the outer scope
- Only one handler executes per event, so no conflicts

### 2. ⚠️ **Token Pair Mismatch (CRITICAL BUG - FIXED)**

**The Real Problem:** Raydium protocols were using token **ACCOUNT** addresses instead of **MINT** addresses!

**Before (BROKEN):**
```rust
// Raydium AMM V4
TokenPair::new(
    event.pool_coin_token_account,  // ❌ Account address
    event.pool_pc_token_account      // ❌ Account address
)

// Raydium CLMM v1 & v2
TokenPair::new(event.input_vault, event.output_vault)  // ❌ Vault addresses
```

**After (FIXED):**
```rust
// Raydium CLMM V2
TokenPair::new(event.input_vault_mint, event.output_vault_mint)  // ✅ Mint addresses!

// Jupiter (always correct)
TokenPair::new(event.source_mint, event.destination_mint)  // ✅ Mint addresses!

// Raydium CPMM (always correct)
TokenPair::new(event.input_token_mint, event.output_token_mint)  // ✅ Mint addresses!
```

**Why This Broke Everything:**
- Jupiter creates: `TokenPair(SOL_MINT, USDC_MINT)`
- Raydium (old) creates: `TokenPair(POOL_SOL_ACCOUNT, POOL_USDC_ACCOUNT)` ← Different addresses!
- They NEVER match → No arbitrage detected!

### 3. ⚠️ **Protocols Temporarily Disabled**

Due to missing mint information in event structures:

**DISABLED (returns empty Vec):**
- ❌ Raydium AMM V4 - No mint fields in event (only has token accounts)
- ❌ Raydium CLMM v1 - No mint fields in event (only has vault addresses)

**ENABLED:**
- ✅ Jupiter Agg V6 - Has source_mint/destination_mint
- ✅ Raydium CPMM - Has input_token_mint/output_token_mint
- ✅ Raydium CLMM V2 - Has input_vault_mint/output_vault_mint

## What the Debug Output Will Show

When you run the example now, you'll see:

### When a Quote is Added:
```
📊 Adding quote: Jupiter - 1100000/1000000 = 1.100000 (existing quotes: 0)
```

### When Checking for Arbitrage:
```
🔍 Checking 2 quotes for token pair So11111.../EPjFWdd...
```

### When Prices Don't Meet Threshold:
```
💡 Found price difference: Jupiter @ 1.100000 vs RaydiumCpmm @ 1.098000 = 0.18% profit
❌ Below 0.50% threshold
```

### When Opportunities ARE Found:
```
💡 Found price difference: Jupiter @ 1.100000 vs RaydiumCpmm @ 1.108000 = 0.73% profit
✅ Meets threshold! Adding opportunity

🚀 ARBITRAGE OPPORTUNITY DETECTED! 🚀
================================================
Token Pair: So11111... <-> EPjFWdd...
Buy on:  Jupiter at price 1.100000
Sell on: RaydiumCpmm at price 1.108000
...
```

## Why You Might Still Not See Opportunities

Even with the fixes, you might not see arbitrage because:

1. **No Matching Token Pairs** - Different DEXes trading different pairs
2. **Price Differences Too Small** - Most efficient markets have <0.5% spreads
3. **Timing** - Quotes expire after 30 seconds, markets move fast
4. **Limited Protocol Support** - Only Jupiter, CPMM, and CLMM V2 are enabled now

## To Enable Full Coverage

To enable Raydium AMM V4 and CLMM v1, you need to:

1. **Listen to Pool Initialize Events** and cache `pool_address → (mint0, mint1)` mappings
2. **Subscribe to Pool Account Updates** and parse mint info from account data
3. **Look up mints on-demand** using RPC calls (slower but simpler)

## Testing Tips

To verify it's working:
1. Watch for the `📊 Adding quote` messages
2. If you see `existing quotes: 1+`, it's finding multiple quotes for the same pair
3. Look for `💡 Found price difference` messages to see if prices are being compared
4. Check if profit percentages are close to your 0.5% threshold

## Quick Fix to See More Opportunities

To see more opportunities (for testing), temporarily lower the threshold:

```rust
let detector = Arc::new(Mutex::new(ArbitrageDetector::new(0.1, 30)));  // 0.1% instead of 0.5%
```

This will show smaller price differences that might not be worth executing.
