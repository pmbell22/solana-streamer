use solana_streamer_sdk::{
    match_event,
    streaming::{
        event_parser::{
            common::{filter::EventTypeFilter, EventType},
            protocols::{
                jupiter_agg_v6::{
                    parser::JUPITER_AGG_V6_PROGRAM_ID,
                    JupiterAggV6RouteEvent,
                    JupiterAggV6ExactOutRouteEvent,
                    JupiterAggV6SwapEvent,
                },
                BlockMetaEvent,
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
    println!("Starting Jupiter Aggregator V6 Yellowstone gRPC Streamer...");
    test_jupiter_agg_v6_grpc().await?;
    Ok(())
}

async fn test_jupiter_agg_v6_grpc() -> Result<(), Box<dyn std::error::Error>> {
    println!("Subscribing to Jupiter Aggregator V6 events...");

    // Create low-latency configuration
    let mut config: ClientConfig = ClientConfig::low_latency();
    // Enable performance monitoring, has performance overhead, disabled by default
    config.enable_metrics = true;
    let grpc = YellowstoneGrpc::new_with_config(
        "https://solana-yellowstone-grpc.publicnode.com:443".to_string(),
        None,
        config,
    )?;

    println!("GRPC client created successfully");

    let callback = create_event_callback();

    // Monitor Jupiter Aggregator V6 protocol
    let protocols = vec![
        Protocol::JupiterAggV6,
    ];

    println!("Protocols to monitor: {:?}", protocols);

    // Filter accounts - listen to Jupiter Aggregator V6 program
    let account_include = vec![
        JUPITER_AGG_V6_PROGRAM_ID.to_string(),
    ];
    let account_exclude = vec![];
    let account_required = vec![];

    // Listen to transaction data
    let transaction_filter = TransactionFilter {
        account_include: account_include.clone(),
        account_exclude,
        account_required,
    };

    // Listen to account data belonging to owner programs -> account event monitoring
    let account_filter = AccountFilter {
        account: vec![],
        owner: account_include.clone(),
        filters: vec![]
    };

    // Event filtering - Include Jupiter Aggregator V6 event types
    // Note: Currently only Route events are captured (instruction-based).
    // SwapEvents (log-based) require additional log parsing infrastructure.
    // Route events contain: in_amount, quoted_out_amount, source_mint, destination_mint
    // which is sufficient for arbitrage opportunity detection.
    let event_type_filter = Some(EventTypeFilter {
        include: vec![
            EventType::JupiterAggV6Route,
            EventType::JupiterAggV6ExactOutRoute,
            // EventType::JupiterAggV6Swap,  // Requires log parsing (not yet implemented)
        ],
    });

    println!("Starting to listen for Jupiter Aggregator V6 events, press Ctrl+C to stop...");
    println!("Monitoring program: {}", JUPITER_AGG_V6_PROGRAM_ID);

    println!("Starting subscription...");

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

    let grpc_clone = grpc.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1000)).await;
        grpc_clone.stop().await;
    });

    println!("Waiting for Ctrl+C to stop...");
    tokio::signal::ctrl_c().await?;

    Ok(())
}

fn create_event_callback() -> impl Fn(Box<dyn UnifiedEvent>) {
    |event: Box<dyn UnifiedEvent>| {
        // Define target mints to filter (SOL and USDC) - uncomment filter code below to use
        let _target_mints = vec![
            "So11111111111111111111111111111111111111112",  // SOL (Wrapped SOL)
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v", // USDC
        ];

        println!(
            "🔔 Jupiter Swap Event | Type: {:?} | TX: {:?}",
            event.event_type(),
            event.transaction_index()
        );

        match_event!(event, {
            // -------------------------- block meta -----------------------
            BlockMetaEvent => |e: BlockMetaEvent| {
                println!("BlockMeta | Handle Time: {} μs", e.metadata.handle_us);
            },
            // -------------------------- Jupiter Aggregator V6 Route (Swap Intent) -----------------------
            JupiterAggV6RouteEvent => |e: JupiterAggV6RouteEvent| {
                // Uncomment to filter for specific token pairs
                // let source_mint_str = e.source_mint.to_string();
                // let dest_mint_str = e.destination_mint.to_string();
                // if !target_mints.contains(&source_mint_str.as_str()) ||
                //    !target_mints.contains(&dest_mint_str.as_str()) {
                //     return; // Skip this event
                // }

                println!("═══════════════════════════════════════════════════════");
                println!("JUPITER SWAP (Route)");
                println!("═══════════════════════════════════════════════════════");
                println!("  Swap: {} → {}", e.source_mint, e.destination_mint);
                println!("  Input Amount: {}", e.in_amount);
                println!("  Quoted Output: {}", e.quoted_out_amount);
                println!("  Slippage: {} bps", e.slippage_bps);
                println!("  Platform Fee: {} bps", e.platform_fee_bps);
                println!("  User Accounts:");
                println!("    Source: {}", e.user_source_token_account);
                println!("    Dest:   {}", e.user_destination_token_account);
                println!("  Signature: {}", e.metadata.signature);
                println!("  Slot: {}", e.metadata.slot);
                println!("═══════════════════════════════════════════════════════");
            },
            JupiterAggV6ExactOutRouteEvent => |e: JupiterAggV6ExactOutRouteEvent| {
                // Uncomment to filter for specific token pairs
                // let source_mint_str = e.source_mint.to_string();
                // let dest_mint_str = e.destination_mint.to_string();
                // if !target_mints.contains(&source_mint_str.as_str()) ||
                //    !target_mints.contains(&dest_mint_str.as_str()) {
                //     return; // Skip this event
                // }

                println!("═══════════════════════════════════════════════════════");
                println!("JUPITER SWAP (Exact Out Route)");
                println!("═══════════════════════════════════════════════════════");
                println!("  Swap: {} → {}", e.source_mint, e.destination_mint);
                println!("  Quoted Input: {}", e.quoted_in_amount);
                println!("  Output Amount: {}", e.out_amount);
                println!("  Slippage: {} bps", e.slippage_bps);
                println!("  Platform Fee: {} bps", e.platform_fee_bps);
                println!("  User Accounts:");
                println!("    Source: {}", e.user_source_token_account);
                println!("    Dest:   {}", e.user_destination_token_account);
                println!("  Signature: {}", e.metadata.signature);
                println!("  Slot: {}", e.metadata.slot);
                println!("═══════════════════════════════════════════════════════");
            },
            JupiterAggV6SwapEvent => |e: JupiterAggV6SwapEvent| {
                // This event type requires log parsing (not yet implemented)
                println!("═══════════════════════════════════════════════════════");
                println!("JUPITER SWAP (Execution Log - Not Yet Implemented)");
                println!("═══════════════════════════════════════════════════════");
                println!("  AMM: {}", e.amm);
                println!("  Input: {} {}", e.input_amount, e.input_mint);
                println!("  Output: {} {}", e.output_amount, e.output_mint);
                println!("  Signature: {}", e.metadata.signature);
                println!("═══════════════════════════════════════════════════════");
            },
        });
    }
}
