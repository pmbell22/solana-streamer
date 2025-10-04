# DEX IDL Parser

A unified, IDL-based parsing system for Solana DEX protocols. This crate provides a consistent way to parse instructions from Jupiter, Raydium, Orca, and other DEXs using their Interface Definition Language (IDL) files.

## Features

- **Unified Parsing**: Single API to parse instructions from multiple DEX protocols
- **IDL-Based**: Uses official IDL files for accurate instruction parsing
- **Yellowstone gRPC Integration**: Seamlessly works with Yellowstone gRPC streaming
- **Extensible**: Easy to add new DEX protocols by providing their IDL files

## Supported Protocols

- **Jupiter Aggregator V6** (`JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`)
- **Raydium CLMM** (`CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK`)
- **Orca Whirlpool** (`whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc`)

## Architecture

The parsing system follows the approach used by [tkhq/solana-parser](https://github.com/tkhq/solana-parser), providing:

1. **IDL Types** (`idl/types.rs`): Complete representation of Anchor IDL structure
2. **IDL Loader** (`idl/loader.rs`): Load and process IDL files
3. **Instruction Parser** (`parser/instruction.rs`): Parse transaction instructions using IDL definitions
4. **Stream Parser** (`streaming.rs`): Integration with Yellowstone gRPC for real-time event streaming
5. **Unified Events** (`types/event.rs`): Common event types across all DEXs

## Usage

### Basic Instruction Parsing

```rust
use dex_idl_parser::prelude::*;

// Load an IDL and create a parser
let idl = load_idl_from_file("idls/jupiter_agg_v6.json")?;
let parser = InstructionParser::new(idl);

// Parse an instruction
let parsed = parser.parse_instruction(
    instruction_data,
    accounts,
)?;

println!("Instruction: {}", parsed.instruction);
println!("Accounts: {:?}", parsed.accounts);
```

### Unified Multi-DEX Parsing

```rust
use dex_idl_parser::prelude::*;

// Create a unified parser for all supported DEXs
let dex_parser = DexStreamParser::new_all_protocols()?;

// Parse a transaction instruction
let event = dex_parser.parse_instruction(
    &program_id,
    instruction_data,
    accounts,
    signature,
    slot,
    block_time,
    tx_index,
)?;

// Check event type
if event.is_swap() {
    println!("Swap detected: {}", event);
}
```

### Yellowstone gRPC Streaming

```rust
use dex_idl_parser::prelude::*;
use solana_streamer_sdk::streaming::YellowstoneGrpc;

// Initialize parser
let dex_parser = DexStreamParser::new_all_protocols()?;

// Setup gRPC streaming
let grpc = YellowstoneGrpc::new(...)?;

grpc.subscribe_geyser_raw(
    vec![transaction_filter],
    vec![account_filter],
    Box::new(move |update| {
        if let Some(tx_update) = extract_transaction_update(update) {
            let events = dex_parser.parse_from_grpc_transaction(
                &tx_update.transaction,
                tx_update.slot,
                block_time,
            );

            for event in events {
                println!("DEX Event: {}", event);
            }
        }
    }),
).await?;
```

## Adding New DEX Protocols

To add support for a new DEX:

1. **Obtain the IDL file** (usually from the protocol's GitHub repo)
2. **Add to `idls/` directory**: `idls/new_dex.json`
3. **Update `DexProtocol` enum** in `idl/loader.rs`:
   ```rust
   pub enum DexProtocol {
       // ... existing protocols
       NewDex,
   }
   ```
4. **Add protocol metadata**:
   ```rust
   impl DexProtocol {
       pub fn program_id(&self) -> &'static str {
           match self {
               // ... existing protocols
               DexProtocol::NewDex => "NewDexProgramID...",
           }
       }

       pub fn idl_path(&self) -> &'static str {
           match self {
               // ... existing protocols
               DexProtocol::NewDex => "idls/new_dex.json",
           }
       }
   }
   ```

That's it! The parser will automatically handle the new protocol.

## Examples

Run the unified DEX parser example:

```bash
cargo run --example unified_dex_parser_example
```

This will stream and parse events from Jupiter, Raydium, and Orca in real-time.

## IDL Files

The IDL files are sourced from official repositories:

- **Jupiter V6**: [jup-ag/jupiter-cpi](https://github.com/jup-ag/jupiter-cpi/blob/main/idl.json)
- **Raydium CLMM**: [raydium-io/raydium-idl](https://github.com/raydium-io/raydium-idl)
- **Orca Whirlpool**: [orca-so/whirlpools](https://github.com/orca-so/whirlpools)

## Event Types

All parsed events implement a unified `DexEvent` type with:

- `protocol`: The DEX protocol (Jupiter, Raydium, Orca)
- `instruction`: Parsed instruction with named accounts and data
- `signature`: Transaction signature
- `slot`: Slot number
- `block_time`: Block timestamp
- Helper methods: `is_swap()`, `is_liquidity_provision()`, `is_liquidity_removal()`

## Testing

Run tests:

```bash
cargo test --package dex-idl-parser
```

## License

MIT
