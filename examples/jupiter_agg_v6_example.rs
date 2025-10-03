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
    let event_type_filter = Some(EventTypeFilter {
        include: vec![
            EventType::JupiterAggV6Route,
            EventType::JupiterAggV6ExactOutRoute,
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
        println!(
            "🎉 Jupiter Event received! Type: {:?}, transaction_index: {:?}",
            event.event_type(),
            event.transaction_index()
        );
        match_event!(event, {
            // -------------------------- block meta -----------------------
            BlockMetaEvent => |e: BlockMetaEvent| {
                println!("BlockMetaEvent: {:?}", e.metadata.handle_us);
            },
            // -------------------------- Jupiter Aggregator V6 -----------------------
            JupiterAggV6RouteEvent => |e: JupiterAggV6RouteEvent| {
                println!("JupiterAggV6RouteEvent:");
                println!("  In Amount: {}", e.in_amount);
                println!("  Quoted Out Amount: {}", e.quoted_out_amount);
                println!("  Slippage BPS: {}", e.slippage_bps);
                println!("  Platform Fee BPS: {}", e.platform_fee_bps);
                println!("  Source Mint: {}", e.source_mint);
                println!("  Destination Mint: {}", e.destination_mint);
                println!("  User Source Token Account: {}", e.user_source_token_account);
                println!("  User Destination Token Account: {}", e.user_destination_token_account);
                println!("  Full event: {e:?}");
            },
            JupiterAggV6ExactOutRouteEvent => |e: JupiterAggV6ExactOutRouteEvent| {
                println!("JupiterAggV6ExactOutRouteEvent:");
                println!("  Out Amount: {}", e.out_amount);
                println!("  Quoted In Amount: {}", e.quoted_in_amount);
                println!("  Slippage BPS: {}", e.slippage_bps);
                println!("  Platform Fee BPS: {}", e.platform_fee_bps);
                println!("  Source Mint: {}", e.source_mint);
                println!("  Destination Mint: {}", e.destination_mint);
                println!("  User Source Token Account: {}", e.user_source_token_account);
                println!("  User Destination Token Account: {}", e.user_destination_token_account);
                println!("  Full event: {e:?}");
            },
        });
    }
}
