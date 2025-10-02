use solana_streamer_sdk::streaming::{
    event_parser::{
        config::dynamic_parser::DynamicEvent,
        core::ConfigurableEventParser,
        UnifiedEvent,
    },
    yellowstone_grpc::{AccountFilter, TransactionFilter},
    YellowstoneGrpc,
};
use solana_streamer_sdk::streaming::grpc::ClientConfig;
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::init();

    println!("=== Jupiter V6 Swap Streaming ===\n");

    // Load Jupiter V6 protocol config
    println!("1. Loading Jupiter V6 configuration...");
    let config_dir = Path::new("configs/protocols");

    let parser = ConfigurableEventParser::from_config_directory(
        vec![], // No static protocols, only config-based ones
        config_dir,
        None,
    )?;

    println!("   ✓ Loaded protocols: {:?}", parser.protocol_names());
    println!("   ✓ Program ID: {}\n", parser.program_ids()[0]);

    // Set up gRPC connection
    println!("2. Connecting to Yellowstone gRPC...");
    let grpc_endpoint = std::env::var("GRPC_ENDPOINT")
        .unwrap_or_else(|_| "https://solana-yellowstone-grpc.publicnode.com:443".to_string());
    let grpc_x_token = std::env::var("GRPC_X_TOKEN").ok();

    println!("   Endpoint: {}", grpc_endpoint);

    // let grpc = YellowstoneGrpc::new(
    //     grpc_endpoint.clone(),
    //     grpc_x_token.clone(),
    // )?;
    
    let mut config: ClientConfig = ClientConfig::low_latency();
    let grpc = YellowstoneGrpc::new_with_config(
        "https://solana-yellowstone-grpc.publicnode.com:443".to_string(),
        None,
        config,
    )?;

    // Build subscription filters
    println!("\n3. Building subscription...");

    let program_ids: Vec<String> = parser.program_ids()
        .iter()
        .map(|p| {
            println!("   Subscribing to: {}", p);
            p.to_string()
        })
        .collect();

    let transaction_filter = TransactionFilter {
        account_include: program_ids.clone(),
        account_exclude: vec![],
        account_required: vec![],
    };

    let account_filter = AccountFilter {
        account: vec![],
        owner: program_ids,
        filters: vec![],
    };

    // Event callback
    println!("\n4. Starting Jupiter swap stream...\n");

    let event_callback = move |event: Box<dyn UnifiedEvent>| {
        if let Some(dynamic_event) = event.as_any().downcast_ref::<DynamicEvent>() {
            println!("┌─────────────────────────────────────────────────────");
            println!("│ 🪐 {} Swap", dynamic_event.instruction_name);
            println!("├─────────────────────────────────────────────────────");
            println!("│ Signature: {}", event.signature());
            println!("│ Slot:      {}", event.slot());
            println!("├─────────────────────────────────────────────────────");

            // Print key accounts
            if let Some(source) = dynamic_event.accounts.get("user_source_token_account") {
                println!("│ Source Account: {}", source);
            }
            if let Some(dest) = dynamic_event.accounts.get("user_destination_token_account") {
                println!("│ Dest Account:   {}", dest);
            }

            println!("├─────────────────────────────────────────────────────");

            // Print swap data
            if let Some(in_amount) = dynamic_event.data_fields.get("in_amount") {
                println!("│ In Amount:       {:?}", in_amount);
            }
            if let Some(out_amount) = dynamic_event.data_fields.get("quoted_out_amount") {
                println!("│ Quoted Out:      {:?}", out_amount);
            }
            if let Some(slippage) = dynamic_event.data_fields.get("slippage_bps") {
                println!("│ Slippage (bps):  {:?}", slippage);
            }

            println!("└─────────────────────────────────────────────────────\n");
        }
    };

    // Subscribe to events
    println!("✓ Starting subscription...");

    grpc.subscribe_events_immediate(
        vec![], // No static protocols, only config-based ones
        None,
        vec![transaction_filter],
        vec![account_filter],
        None, // No event type filtering
        None, // Default commitment (Confirmed)
        event_callback,
    )
    .await?;

    println!("✓ Streaming Jupiter V6 swaps...");
    println!("  Press Ctrl+C to stop\n");

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;
    println!("\n\nShutting down gracefully...");

    Ok(())
}
