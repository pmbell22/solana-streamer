# Solana Streamer
[中文](https://github.com/0xfnzero/solana-streamer/blob/main/README_CN.md) | [English](https://github.com/0xfnzero/solana-streamer/blob/main/README.md) | [Telegram](https://t.me/fnzero_group)

A lightweight Rust library for real-time event streaming from Solana DEX trading programs. This library provides efficient event parsing and subscription capabilities for PumpFun, PumpSwap, Bonk, and Raydium CPMM protocols.

## Project Features

1. **Real-time Event Streaming**: Subscribe to live trading events from multiple Solana DEX protocols
2. **Yellowstone gRPC Support**: High-performance event subscription using Yellowstone gRPC
3. **ShredStream Support**: Alternative event streaming using ShredStream protocol
4. **Multi-Protocol Support**: 
   - **PumpFun**: Meme coin trading platform events
   - **PumpSwap**: PumpFun's swap protocol events
   - **Bonk**: Token launch platform events (letsbonk.fun)
   - **Raydium CPMM**: Raydium's Concentrated Pool Market Maker events
   - **Raydium CLMM**: Raydium's Concentrated Liquidity Market Maker events
   - **Raydium AMM V4**: Raydium's Automated Market Maker V4 events
5. **Unified Event Interface**: Consistent event handling across all supported protocols
6. **Event Parsing System**: Automatic parsing and categorization of protocol-specific events
7. **Account State Monitoring**: Real-time monitoring of protocol account states and configuration changes
8. **Transaction & Account Event Filtering**: Separate filtering for transaction events and account state changes
9. **High Performance**: Optimized for low-latency event processing
10. **Batch Processing Optimization**: Batch processing events to reduce callback overhead
11. **Performance Monitoring**: Built-in performance metrics monitoring, including event processing speed, etc.
12. **Memory Optimization**: Object pooling and caching mechanisms to reduce memory allocations
13. **Flexible Configuration System**: Support for custom batch sizes, backpressure strategies, channel sizes, and other parameters
14. **Preset Configurations**: Provides high-throughput and low-latency preset configurations optimized for different use cases
15. **Backpressure Handling**: Supports blocking and dropping backpressure strategies
16. **Runtime Configuration Updates**: Supports dynamic configuration parameter updates at runtime
17. **Full Function Performance Monitoring**: All subscribe_events functions support performance monitoring, automatically collecting and reporting performance metrics
18. **Graceful Shutdown**: Support for programmatic stop() method for clean shutdown
19. **Dynamic Subscription Management**: Runtime filter updates without reconnection, enabling adaptive monitoring strategies

## Installation

### Direct Clone

Clone this project to your project directory:

```bash
cd your_project_root_directory
git clone https://github.com/0xfnzero/solana-streamer
```

Add the dependency to your `Cargo.toml`:

```toml
# Add to your Cargo.toml
solana-streamer-sdk = { path = "./solana-streamer", version = "0.4.1" }
```

### Use crates.io

```toml
# Add to your Cargo.toml
solana-streamer-sdk = "0.4.1"
```

## Configuration System

### Preset Configurations

The library provides three preset configurations optimized for different use cases:

#### 1. High Throughput Configuration (`high_throughput()`)

Optimized for high-concurrency scenarios, prioritizing throughput over latency:

```rust
let config = StreamClientConfig::high_throughput();
// Or use convenience methods
let grpc = YellowstoneGrpc::new_high_throughput(endpoint, token)?;
let shred = ShredStreamGrpc::new_high_throughput(endpoint).await?;
```

**Features:**
- **Backpressure Strategy**: Drop - drops messages during high load to avoid blocking
- **Buffer Size**: 5,000 permits to handle burst traffic
- **Use Case**: Scenarios where you need to process large volumes of data and can tolerate occasional message drops during peak loads

#### 2. Low Latency Configuration (`low_latency()`)

Optimized for real-time scenarios, prioritizing latency over throughput:

```rust
let config = StreamClientConfig::low_latency();
// Or use convenience methods
let grpc = YellowstoneGrpc::new_low_latency(endpoint, token)?;
let shred = ShredStreamGrpc::new_low_latency(endpoint).await?;
```

**Features:**
- **Backpressure Strategy**: Block - ensures no data loss
- **Buffer Size**: 4000 permits for balanced throughput and latency
- **Immediate Processing**: No buffering, processes events immediately
- **Use Case**: Scenarios where every millisecond counts and you cannot afford to lose any events, such as trading applications or real-time monitoring


### Custom Configuration

You can also create custom configurations:

```rust
let config = StreamClientConfig {
    connection: ConnectionConfig {
        connect_timeout: 30,
        request_timeout: 120,
        max_decoding_message_size: 20 * 1024 * 1024, // 20MB
    },
    backpressure: BackpressureConfig {
        permits: 2000,
        strategy: BackpressureStrategy::Block,
    },
    enable_metrics: true,
};
```

## Usage Examples

### Usage Examples Summary Table

| Feature Type | Example File | Description | Run Command | Source Path |
|---------|---------|------|---------|----------|
| Yellowstone gRPC Stream | `grpc_example.rs` | Monitor transaction events using Yellowstone gRPC | `cargo run --example grpc_example` | [examples/grpc_example.rs](examples/grpc_example.rs) |
| ShredStream Stream | `shred_example.rs` | Monitor transaction events using ShredStream | `cargo run --example shred_example` | [examples/shred_example.rs](examples/shred_example.rs) |
| Parse Transaction Events | `parse_tx_events` | Parse Solana mainnet transaction data | `cargo run --example parse_tx_events` | [examples/parse_tx_events.rs](examples/parse_tx_events.rs) |
| Dynamic Subscription Management | `dynamic_subscription` | Update filters at runtime | `cargo run --example dynamic_subscription` | [examples/dynamic_subscription.rs](examples/dynamic_subscription.rs) |

### Event Filtering

The library supports flexible event filtering to reduce processing overhead and improve performance:

#### Basic Filtering

```rust
use solana_streamer_sdk::streaming::event_parser::common::{filter::EventTypeFilter, EventType};

// No filtering - receive all events
let event_type_filter = None;

// Filter specific event types - only receive PumpSwap buy/sell events
let event_type_filter = Some(EventTypeFilter { 
    include: vec![EventType::PumpSwapBuy, EventType::PumpSwapSell] 
});
```

#### Performance Impact

Event filtering can provide significant performance improvements:
- **60-80% reduction** in unnecessary event processing
- **Lower memory usage** by filtering out irrelevant events
- **Reduced network bandwidth** in distributed setups
- **Better focus** on events that matter to your application

#### Filtering Examples by Use Case

**Trading Bot (Focus on Trade Events)**
```rust
let event_type_filter = Some(EventTypeFilter { 
    include: vec![
        EventType::PumpSwapBuy,
        EventType::PumpSwapSell,
        EventType::PumpFunTrade,
        EventType::RaydiumCpmmSwap,
        EventType::RaydiumClmmSwap,
        EventType::RaydiumAmmV4Swap,
        ......
    ] 
});
```

**Pool Monitoring (Focus on Liquidity Events)**
```rust
let event_type_filter = Some(EventTypeFilter { 
    include: vec![
        EventType::PumpSwapCreatePool,
        EventType::PumpSwapDeposit,
        EventType::PumpSwapWithdraw,
        EventType::RaydiumCpmmInitialize,
        EventType::RaydiumCpmmDeposit,
        EventType::RaydiumCpmmWithdraw,
        EventType::RaydiumClmmCreatePool,
        ......
    ] 
});
```

## Dynamic Subscription Management

Update subscription filters at runtime without reconnecting to the stream.

```rust
// Update filters on existing subscription
grpc.update_subscription(
    TransactionFilter {
        account_include: vec!["new_program_id".to_string()],
        account_exclude: vec![],
        account_required: vec![],
    },
    AccountFilter {
        account: vec![],
        owner: vec![],
    },
).await?;
```

- **No Reconnection**: Filter changes apply immediately without closing the stream
- **Atomic Updates**: Both transaction and account filters updated together
- **Single Subscription**: One active subscription per client instance
- **Compatible**: Works with both immediate and advanced subscription methods

Note: Multiple subscription attempts on the same client return an error.

## Supported Protocols

- **PumpFun**: Primary meme coin trading platform
- **PumpSwap**: PumpFun's swap protocol
- **Bonk**: Token launch platform (letsbonk.fun)
- **Raydium CPMM**: Raydium's Concentrated Pool Market Maker protocol
- **Raydium CLMM**: Raydium's Concentrated Liquidity Market Maker protocol
- **Raydium AMM V4**: Raydium's Automated Market Maker V4 protocol

## Event Streaming Services

- **Yellowstone gRPC**: High-performance Solana event streaming
- **ShredStream**: Alternative event streaming protocol

## Architecture Features

### Unified Event Interface

- **UnifiedEvent Trait**: All protocol events implement a common interface
- **Protocol Enum**: Easy identification of event sources
- **Event Factory**: Automatic event parsing and categorization

### Event Parsing System

- **Protocol-specific Parsers**: Dedicated parsers for each supported protocol
- **Event Factory**: Centralized event creation and parsing
- **Extensible Design**: Easy to add new protocols and event types

### Streaming Infrastructure

- **Yellowstone gRPC Client**: Optimized for Solana event streaming
- **ShredStream Client**: Alternative streaming implementation
- **Async Processing**: Non-blocking event handling

## Project Structure

```
src/
├── common/           # Common functionality and types
├── protos/           # Protocol buffer definitions
├── streaming/        # Event streaming system
│   ├── event_parser/ # Event parsing system
│   │   ├── common/   # Common event parsing tools
│   │   ├── core/     # Core parsing traits and interfaces
│   │   ├── protocols/# Protocol-specific parsers
│   │   │   ├── bonk/ # Bonk event parsing
│   │   │   ├── pumpfun/ # PumpFun event parsing
│   │   │   ├── pumpswap/ # PumpSwap event parsing
│   │   │   ├── raydium_amm_v4/ # Raydium AMM V4 event parsing
│   │   │   ├── raydium_cpmm/ # Raydium CPMM event parsing
│   │   │   └── raydium_clmm/ # Raydium CLMM event parsing
│   │   └── factory.rs # Parser factory
│   ├── shred_stream.rs # ShredStream client
│   ├── yellowstone_grpc.rs # Yellowstone gRPC client
│   └── yellowstone_sub_system.rs # Yellowstone subsystem
├── lib.rs            # Main library file
└── main.rs           # Example program
```

## License

MIT License

## Contact

- Project Repository: https://github.com/0xfnzero/solana-streamer
- Telegram Group: https://t.me/fnzero_group

## Performance Considerations

1. **Connection Management**: Properly handle connection lifecycle and reconnection
2. **Event Filtering**: Use protocol filtering to reduce unnecessary event processing
3. **Memory Management**: Implement appropriate cleanup for long-running streams
4. **Error Handling**: Robust error handling for network issues and service interruptions
5. **Batch Processing Optimization**: Use batch processing to reduce callback overhead and improve throughput
6. **Performance Monitoring**: Enable performance monitoring to identify bottlenecks and optimization opportunities
7. **Graceful Shutdown**: Use the stop() method for clean shutdown and implement signal handlers for proper resource cleanup

## Important Notes

1. **Network Stability**: Ensure stable network connection for continuous event streaming
2. **Rate Limiting**: Be aware of rate limits on public gRPC endpoints
3. **Error Recovery**: Implement proper error handling and reconnection logic
5. **Compliance**: Ensure compliance with relevant laws and regulations

## Language Versions

- [English](README.md)
- [中文](README_CN.md)
