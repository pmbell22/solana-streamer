# Quick Start: Streaming Jupiter & Orca Events

This guide shows how to stream swap events from Jupiter Aggregator V6 and Orca Whirlpool using the config-based parser.

## What's Included

The `configs/protocols/` directory now contains:

- **`jupiter_v6.json`** - Jupiter Aggregator V6 configuration
  - Program ID: `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`
  - Instructions: `route`, `shared_accounts_route`, `exact_out_route`

- **`orca_whirlpool.json`** - Orca Whirlpool configuration
  - Program ID: `whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc`
  - Instructions: `swap`, `swap_v2`, `two_hop_swap`

## Running the Example

### 1. Set up your gRPC endpoint

```bash
export GRPC_ENDPOINT="your-yellowstone-grpc-endpoint"
export GRPC_X_TOKEN="your-auth-token"  # Optional
```

### 2. Run the streaming example

```bash
cargo run --example jupiter_orca_streaming
```

This will:
- Load all protocol configs from `configs/protocols/`
- Connect to Yellowstone gRPC
- Subscribe to Jupiter V6 and Orca Whirlpool transactions
- Print swap events in real-time

## Code Example

```rust
use solana_streamer_sdk::streaming::{
    event_parser::core::ConfigurableEventParser,
    yellowstone_grpc::YellowstoneGrpcClient,
};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configs from directory
    let parser = ConfigurableEventParser::from_config_directory(
        vec![],  // No static protocols needed
        "configs/protocols",
        None,
    )?;

    println!("Loaded: {:?}", parser.protocol_names());
    // Output: ["jupiter_aggregator_v6", "orca_whirlpool", ...]

    // Connect to gRPC
    let mut client = YellowstoneGrpcClient::new(
        "http://127.0.0.1:10000",
        None,
        None,
    ).await?;

    // Subscribe to events
    let callback = Arc::new(|event: Box<dyn UnifiedEvent>| {
        println!("Event: {:?}", event.event_type());
    });

    client.subscribe_with_callback(config, callback).await?;
    Ok(())
}
```

## Event Data Access

When you receive an event, you can access the parsed data:

```rust
use solana_streamer_sdk::streaming::event_parser::config::dynamic_parser::DynamicEvent;

let callback = Arc::new(|event: Box<dyn UnifiedEvent>| {
    if let Some(dynamic) = event.as_any().downcast_ref::<DynamicEvent>() {
        // Access accounts
        if let Some(user_authority) = dynamic.accounts.get("user_transfer_authority") {
            println!("User: {}", user_authority);
        }

        // Access data fields
        if let Some(DynamicFieldValue::U64(amount)) = dynamic.data_fields.get("in_amount") {
            println!("Swap amount: {}", amount);
        }
    }
});
```

## Jupiter V6 Event Fields

### Route Instruction

**Accounts:**
- `user_transfer_authority` - The user initiating the swap
- `user_source_token_account` - Source token account
- `user_destination_token_account` - Destination token account
- `destination_mint` - Destination token mint

**Data Fields:**
- `in_amount` (u64) - Input amount
- `quoted_out_amount` (u64) - Expected output amount
- `slippage_bps` (u16) - Slippage tolerance in basis points
- `platform_fee_bps` (u8) - Platform fee in basis points

## Orca Whirlpool Event Fields

### Swap V2 Instruction

**Accounts:**
- `token_authority` - User authority
- `whirlpool` - Whirlpool pool account
- `token_owner_account_a` - User's token A account
- `token_owner_account_b` - User's token B account
- `token_vault_a` - Pool's token A vault
- `token_vault_b` - Pool's token B vault

**Data Fields:**
- `amount` (u64) - Swap amount
- `other_amount_threshold` (u64) - Minimum out / maximum in
- `sqrt_price_limit` (u128) - Price limit
- `amount_specified_is_input` (bool) - Is exact input swap
- `a_to_b` (bool) - Swap direction (A to B or B to A)

## Filtering Events

You can filter to only specific event types:

```rust
use solana_streamer_sdk::streaming::event_parser::common::{
    filter::EventTypeFilter, EventType
};

let filter = EventTypeFilter {
    include: vec![
        EventType::Custom("JupiterV6Route".to_string()),
        EventType::Custom("OrcaWhirlpoolSwapV2".to_string()),
    ],
};

let parser = ConfigurableEventParser::from_config_directory(
    vec![],
    "configs/protocols",
    Some(filter),
)?;
```

## Adding More Protocols

To add another DEX or protocol:

1. Create a new JSON file in `configs/protocols/`
2. Define the program ID and instructions
3. Restart your application - it will automatically load the new config

No code changes needed!

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `GRPC_ENDPOINT` | Yellowstone gRPC endpoint URL | Yes |
| `GRPC_X_TOKEN` | Authentication token for gRPC | No |
| `RUST_LOG` | Logging level (e.g., `info`, `debug`) | No |

## Example Output

```
=== Jupiter V6 & Orca Whirlpool Streaming Example ===

1. Loading protocol configurations...
   ✓ Loaded protocols: ["jupiter_aggregator_v6", "orca_whirlpool", "raydium_cpmm"]
   ✓ Tracking 5 program IDs

2. Setting up Yellowstone gRPC connection...
   Endpoint: http://127.0.0.1:10000

3. Building subscription configuration...
   Adding program: JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4
   Adding program: whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc
   ✓ Subscription configured

4. Starting event stream...

┌─────────────────────────────────────────────────────
│ 🔥 JupiterV6Route Event
├─────────────────────────────────────────────────────
│ Signature: 2ZE7T...
│ Slot:      284532190
│ Instruction: route
├─────────────────────────────────────────────────────
│ Accounts:
│   • user_transfer_authority: 9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM
│   • user_source_token_account: HJk...
│   • user_destination_token_account: 8pQ...
├─────────────────────────────────────────────────────
│ Data Fields:
│   • in_amount: U64(1000000)
│   • quoted_out_amount: U64(125430)
│   • slippage_bps: U16(50)
│   • platform_fee_bps: U8(0)
└─────────────────────────────────────────────────────
```

## Troubleshooting

### Events not appearing

1. Verify your gRPC endpoint is working
2. Check that the program IDs are correct
3. Enable debug logging: `RUST_LOG=debug cargo run --example jupiter_orca_streaming`

### Config fails to load

1. Validate JSON syntax
2. Ensure discriminators are correct hex strings
3. Verify program IDs are valid base58

### Performance optimization

For high-throughput streams, consider:
- Using event filters to reduce processing
- Implementing batching in your callback
- Using async processing in the callback

## Next Steps

- Review `configs/README.md` for detailed config documentation
- Check `examples/config_based_parser_example.rs` for more examples
- Add your own protocol configs in `configs/protocols/`

## Support

For issues or questions:
- Check existing examples in `examples/`
- Review protocol configs in `configs/protocols/`
- See the main README for architecture details
