# Protocol Configuration System

This directory contains protocol configuration files for the solana-streamer event parser. The configuration system allows you to add new protocols and markets without modifying the Rust code.

## Overview

The config system is inspired by Anchor IDLs and provides a standardized, modular structure for defining Solana program instructions and their event parsing logic.

## Directory Structure

```
configs/
├── README.md
└── protocols/
    ├── raydium_amm_v4.json
    └── example_orca.json
```

## Configuration File Format

Configs can be written in either JSON or TOML format. Here's the schema:

### JSON Example

```json
{
  "name": "protocol_name",
  "version": "1.0.0",
  "program_id": "YourProgramID...",
  "description": "Protocol description",
  "instructions": [
    {
      "name": "instruction_name",
      "discriminator": "09",
      "event_type": "EventTypeName",
      "accounts": [
        {
          "name": "account_name",
          "is_mut": true,
          "is_signer": false,
          "description": "Account description"
        }
      ],
      "data_fields": [
        {
          "name": "field_name",
          "field_type": "u64",
          "offset": 0,
          "description": "Field description"
        }
      ],
      "requires_inner_instruction": false
    }
  ]
}
```

## Field Definitions

### Protocol Config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Unique protocol identifier (e.g., "raydium_amm_v4") |
| `version` | string | Yes | Protocol version (e.g., "1.0.0") |
| `program_id` | string | Yes | Solana program ID (base58 encoded) |
| `description` | string | No | Human-readable protocol description |
| `instructions` | array | Yes | List of instruction configurations |
| `types` | object | No | Custom type definitions for complex structures |

### Instruction Config

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Instruction name (e.g., "swap_base_in") |
| `discriminator` | string | Yes | Instruction discriminator in hex (e.g., "09") |
| `event_type` | string | Yes | Event type identifier |
| `accounts` | array | Yes | Ordered list of account fields |
| `data_fields` | array | No | Instruction data fields (after discriminator) |
| `requires_inner_instruction` | boolean | No | Whether this instruction requires inner instruction data |
| `inner_discriminator` | string | No | Inner instruction discriminator if needed |

### Account Field

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Account field name |
| `is_mut` | boolean | No | Whether account is mutable (default: false) |
| `is_signer` | boolean | No | Whether account is a signer (default: false) |
| `description` | string | No | Account description |

### Data Field

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | string | Yes | Data field name |
| `field_type` | string | Yes | Data type (see supported types below) |
| `offset` | number | Yes | Byte offset in instruction data |
| `description` | string | No | Field description |

### Supported Data Types

- **Integers**: `u8`, `u16`, `u32`, `u64`, `u128`, `i8`, `i16`, `i32`, `i64`, `i128`
- **Boolean**: `bool`
- **Solana Types**: `pubkey`
- **String**: `string`
- **Custom**: Custom type references (defined in `types` field)

## Usage Examples

### 1. Loading a Single Config

```rust
use solana_streamer_sdk::streaming::event_parser::config::ConfigLoader;

let config = ConfigLoader::load_from_file("configs/protocols/raydium_amm_v4.json")?;
println!("Loaded protocol: {}", config.name);
```

### 2. Creating Parser with Configs

```rust
use solana_streamer_sdk::streaming::event_parser::core::ConfigurableEventParser;
use std::path::Path;

let parser = ConfigurableEventParser::new(
    vec![Protocol::RaydiumCpmm],  // Static protocols
    vec![Path::new("configs/protocols/example_orca.json")],  // Dynamic configs
    None,  // No event filter
)?;
```

### 3. Loading All Configs from Directory

```rust
let parser = ConfigurableEventParser::from_config_directory(
    vec![],  // No static protocols
    "configs/protocols",
    None,
)?;

println!("Loaded protocols: {:?}", parser.protocol_names());
```

### 4. Parsing Events

```rust
use std::sync::Arc;

let callback = Arc::new(|event: Box<dyn UnifiedEvent>| {
    println!("Event type: {:?}", event.event_type());

    // Access dynamic event data
    if let Some(dynamic_event) = event.as_any().downcast_ref::<DynamicEvent>() {
        println!("Instruction: {}", dynamic_event.instruction_name);
        println!("Accounts: {:?}", dynamic_event.accounts);
        println!("Data: {:?}", dynamic_event.data_fields);
    }
});

parser.parse_grpc_transaction_owned(
    grpc_tx,
    signature,
    Some(slot),
    block_time,
    recv_us,
    None,
    transaction_index,
    callback,
).await?;
```

## Adding a New Protocol

To add a new protocol/market:

1. **Create a config file** in `configs/protocols/` directory:
   ```bash
   touch configs/protocols/my_new_dex.json
   ```

2. **Define the protocol**:
   ```json
   {
     "name": "my_new_dex",
     "version": "1.0.0",
     "program_id": "YourProgramIDHere...",
     "description": "My new DEX protocol",
     "instructions": [
       {
         "name": "swap",
         "discriminator": "f8c69e91e17587c8",
         "event_type": "MyDexSwap",
         "accounts": [
           {"name": "user", "is_signer": true},
           {"name": "pool", "is_mut": true}
         ],
         "data_fields": [
           {"name": "amount_in", "field_type": "u64", "offset": 0},
           {"name": "amount_out", "field_type": "u64", "offset": 8}
         ]
       }
     ]
   }
   ```

3. **Load it in your code**:
   ```rust
   let parser = ConfigurableEventParser::from_config_directory(
       vec![],
       "configs/protocols",
       None,
   )?;
   ```

That's it! No Rust code changes needed.

## Finding Instruction Discriminators

To find instruction discriminators for a program:

1. **From IDL files**: Check the program's Anchor IDL if available
2. **From transactions**: Inspect transaction data on explorers like Solscan
3. **From source code**: Look at the program's instruction enum

The discriminator is typically the first 1-8 bytes of the instruction data.

## Best Practices

1. **Use descriptive names**: Make instruction and field names clear and consistent
2. **Document fields**: Add descriptions to help others understand the protocol
3. **Version your configs**: Update the version field when making changes
4. **Test your configs**: Run the example to validate your configuration
5. **Share configs**: Consider contributing configs for popular protocols

## Validation

The config system automatically validates:

- ✅ Non-empty protocol names
- ✅ Valid program IDs
- ✅ At least one instruction defined
- ✅ Valid hex discriminators
- ✅ Required fields are present

## Troubleshooting

### Config fails to load
- Check JSON/TOML syntax
- Verify program_id is valid base58
- Ensure discriminator is valid hex

### Events not parsing
- Verify discriminator matches actual instruction data
- Check account order matches on-chain instruction
- Confirm data field offsets are correct

### Type mismatches
- Use correct field_type for each data field
- Verify offset calculations (account for discriminator length)

## Examples

See `examples/config_based_parser_example.rs` for a complete working example.

## Contributing

To contribute a new protocol config:

1. Create a config file for the protocol
2. Test it with the example program
3. Document any special considerations
4. Submit a PR!

## License

MIT
