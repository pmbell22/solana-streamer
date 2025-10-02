use solana_streamer_sdk::streaming::{
    common::{SubscriptionConfig, SubscriptionConfigBuilder},
    event_parser::{
        config::dynamic_parser::DynamicEvent,
        core::ConfigurableEventParser,
        Protocol, UnifiedEvent,
    },
    yellowstone_grpc::YellowstoneGrpcClient,
};
use std::path::Path;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    println!("=== Jupiter V6 & Orca Whirlpool Streaming Example ===\n");

    // Step 1: Load protocol configs from directory
    println!("1. Loading protocol configurations...");
    let config_dir = Path::new("configs/protocols");

    let parser = ConfigurableEventParser::from_config_directory(
        vec![
            // You can also include existing static protocols
            Protocol::RaydiumCpmm,
            Protocol::RaydiumClmm,
            Protocol::RaydiumAmmV4,
        ],
        config_dir,
        None, // No event filter - process all events
    )?;

    println!("   ✓ Loaded protocols: {:?}", parser.protocol_names());
    println!("   ✓ Tracking {} program IDs\n", parser.program_ids().len());

    // Step 2: Set up gRPC connection
    println!("2. Setting up Yellowstone gRPC connection...");

    let grpc_endpoint = std::env::var("GRPC_ENDPOINT")
        .unwrap_or_else(|_| "http://127.0.0.1:10000".to_string());
    let grpc_x_token = std::env::var("GRPC_X_TOKEN").ok();

    println!("   Endpoint: {}", grpc_endpoint);

    let mut client = YellowstoneGrpcClient::new(
        grpc_endpoint.clone(),
        grpc_x_token.clone(),
        None,
    )
    .await?;

    // Step 3: Build subscription config for Jupiter and Orca
    println!("\n3. Building subscription configuration...");

    let mut builder = SubscriptionConfigBuilder::new("jupiter_orca_stream");

    // Get program IDs from configs
    for program_id in parser.program_ids() {
        println!("   Adding program: {}", program_id);
        builder = builder.add_program_id(program_id);
    }

    let subscription_config = builder
        .commitment(yellowstone_grpc_proto::prelude::CommitmentLevel::Confirmed)
        .build();

    println!("   ✓ Subscription configured\n");

    // Step 4: Set up event callback
    println!("4. Starting event stream...\n");

    let event_callback = Arc::new(move |event: Box<dyn UnifiedEvent>| {
        let event_type = event.event_type();
        let signature = event.signature();
        let slot = event.slot();

        // Try to downcast to DynamicEvent to access custom fields
        if let Some(dynamic_event) = event.as_any().downcast_ref::<DynamicEvent>() {
            println!("┌─────────────────────────────────────────────────────");
            println!("│ 🔥 {} Event", event_type);
            println!("├─────────────────────────────────────────────────────");
            println!("│ Signature: {}", signature);
            println!("│ Slot:      {}", slot);
            println!("│ Instruction: {}", dynamic_event.instruction_name);
            println!("├─────────────────────────────────────────────────────");
            println!("│ Accounts:");
            for (name, pubkey) in &dynamic_event.accounts {
                println!("│   • {}: {}", name, pubkey);
            }
            println!("├─────────────────────────────────────────────────────");
            println!("│ Data Fields:");
            for (name, value) in &dynamic_event.data_fields {
                println!("│   • {}: {:?}", name, value);
            }
            println!("└─────────────────────────────────────────────────────\n");
        } else {
            // Handle static protocol events (Raydium, etc.)
            println!("📊 {} | Slot: {} | Sig: {}", event_type, slot, signature);
        }
    });

    // Step 5: Subscribe and stream
    let stream_handle = tokio::spawn(async move {
        match client
            .subscribe_with_callback(subscription_config, event_callback)
            .await
        {
            Ok(_) => println!("Stream completed successfully"),
            Err(e) => eprintln!("Stream error: {}", e),
        }
    });

    println!("✓ Streaming events from Jupiter V6 and Orca Whirlpool...");
    println!("  Press Ctrl+C to stop\n");

    // Wait for stream or Ctrl+C
    tokio::select! {
        _ = stream_handle => {
            println!("Stream ended");
        }
        _ = tokio::signal::ctrl_c() => {
            println!("\n\nShutting down gracefully...");
        }
    }

    Ok(())
}
