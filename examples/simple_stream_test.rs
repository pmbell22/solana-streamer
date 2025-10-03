use solana_streamer_sdk::{
    match_event,
    streaming::{
        event_parser::{
            common::{filter::EventTypeFilter, EventType},
            protocols::{
                jupiter_agg_v6::{
                    events::JupiterAggV6RouteEvent,
                    parser::JUPITER_AGG_V6_PROGRAM_ID,
                },
                block::block_meta_event::BlockMetaEvent,
            },
            Protocol, UnifiedEvent,
        },
        grpc::ClientConfig,
        yellowstone_grpc::{AccountFilter, TransactionFilter},
        YellowstoneGrpc,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting simple stream test...");
    println!("This will show ALL Jupiter swaps (no filtering)");
    println!("================================================\n");

    // Create GRPC client
    let mut config = ClientConfig::low_latency();
    config.enable_metrics = true;

    let grpc = YellowstoneGrpc::new_with_config(
        "https://solana-yellowstone-grpc.publicnode.com:443".to_string(),
        None,
        config,
    )?;

    let callback = |event: Box<dyn UnifiedEvent>| {
        match_event!(event, {
            BlockMetaEvent => |e: BlockMetaEvent| {
                println!("📦 Block: slot {}", e.slot);
            },
            JupiterAggV6RouteEvent => |e: JupiterAggV6RouteEvent| {
                println!("🔵 Jupiter Swap: {} {} -> {} {} (sig: {})",
                    e.in_amount,
                    e.source_mint,
                    e.quoted_out_amount,
                    e.destination_mint,
                    e.metadata.signature
                );
            },
        });
    };

    let protocols = vec![Protocol::JupiterAggV6];

    // Simple filters - just Jupiter program
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

    let event_type_filter = Some(EventTypeFilter {
        include: vec![
            EventType::JupiterAggV6Route,
            EventType::JupiterAggV6ExactOutRoute,
        ],
    });

    println!("Subscribing to Jupiter Agg V6 program: {}", JUPITER_AGG_V6_PROGRAM_ID);
    println!("Waiting for swaps...\n");
    println!("Press Ctrl+C to stop\n");
    println!("================================================\n");

    grpc.subscribe_events_immediate(
        protocols,
        None,
        vec![transaction_filter],
        vec![account_filter],
        event_type_filter,
        None,
        callback,
    )
    .await?;

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    println!("\nShutting down...");
    Ok(())
}
