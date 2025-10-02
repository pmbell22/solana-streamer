use solana_streamer_sdk::streaming::event_parser::{
    config::ConfigLoader,
    core::{ConfigurableEventParser, event_parser::EventParser},
    Protocol, UnifiedEvent,
};
use std::path::Path;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    println!("=== Config-based Event Parser Example ===\n");

    // Example 1: Load a single config file
    println!("1. Loading single config file...");
    let config_path = Path::new("configs/protocols/raydium_amm_v4.json");

    if config_path.exists() {
        let protocol_config = ConfigLoader::load_from_file(config_path)?;
        println!("   Loaded protocol: {} v{}", protocol_config.name, protocol_config.version);
        println!("   Program ID: {}", protocol_config.program_id);
        println!("   Instructions: {}", protocol_config.instructions.len());
        for inst in &protocol_config.instructions {
            println!("     - {} (discriminator: {})", inst.name, inst.discriminator);
        }
    } else {
        println!("   Config file not found at {:?}", config_path);
    }

    println!("\n2. Creating parser with mixed static and dynamic protocols...");

    // Combine static protocols (hardcoded) with dynamic ones (from config)
    let config_paths = vec![
        Path::new("configs/protocols/raydium_amm_v4.json"),
        Path::new("configs/protocols/example_orca.json"),
    ];

    // Filter out non-existent paths
    let existing_paths: Vec<&Path> = config_paths
        .into_iter()
        .filter(|p| p.exists())
        .collect();

    if !existing_paths.is_empty() {
        let parser = ConfigurableEventParser::new(
            vec![Protocol::RaydiumCpmm, Protocol::RaydiumClmm], // Static protocols
            existing_paths,
            None,
        )?;

        println!("   Parser created successfully!");
        println!("   Loaded protocols: {:?}", parser.protocol_names());
        println!("   Total program IDs tracked: {}", parser.program_ids().len());
    } else {
        println!("   No config files found. Creating parser with static protocols only...");
        let parser = EventParser::new(
            vec![Protocol::RaydiumCpmm, Protocol::RaydiumClmm, Protocol::RaydiumAmmV4],
            None,
        );
        println!("   Parser created with {} program IDs", parser.program_ids.len());
    }

    // Example 3: Load from directory
    println!("\n3. Loading all configs from directory...");
    let config_dir = Path::new("configs/protocols");

    if config_dir.exists() && config_dir.is_dir() {
        let parser = ConfigurableEventParser::from_config_directory(
            vec![Protocol::RaydiumCpmm],
            config_dir,
            None,
        )?;

        println!("   Loaded {} protocols from directory", parser.configs.len());
        for config in &parser.configs {
            println!("     - {} ({})", config.name, config.program_id);
        }
    } else {
        println!("   Config directory not found at {:?}", config_dir);
    }

    // Example 4: Demonstrate event parsing with callback
    println!("\n4. Event parsing example:");
    println!("   When you receive gRPC events, you can parse them like this:");
    println!("
   let callback = Arc::new(|event: Box<dyn UnifiedEvent>| {{
       println!(\"Event: {{:?}}\", event.event_type());
       // Process your event here
   }});

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
   ");

    println!("\n=== Example complete ===");
    println!("\nTo add a new protocol:");
    println!("1. Create a JSON or TOML config file in configs/protocols/");
    println!("2. Define the program_id, instructions, and data fields");
    println!("3. Load it using ConfigurableEventParser");
    println!("\nNo code changes needed!");

    Ok(())
}
