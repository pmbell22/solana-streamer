use solana_streamer_sdk::{
    match_event,
    streaming::{
        ArbitrageDetector, ArbitrageOpportunity,
        event_parser::{
            common::{filter::EventTypeFilter, EventType},
            protocols::{
                jupiter_agg_v6::{
                    events::{JupiterAggV6ExactOutRouteEvent, JupiterAggV6RouteEvent, JupiterAggV6FeeEvent},
                    parser::JUPITER_AGG_V6_PROGRAM_ID,
                },
                raydium_amm_v4::{
                    events::RaydiumAmmV4SwapEvent, parser::RAYDIUM_AMM_V4_PROGRAM_ID,
                },
                raydium_clmm::{
                    events::{RaydiumClmmSwapEvent, RaydiumClmmSwapV2Event},
                    parser::RAYDIUM_CLMM_PROGRAM_ID,
                },
                raydium_cpmm::{events::RaydiumCpmmSwapEvent, parser::RAYDIUM_CPMM_PROGRAM_ID},
                block::block_meta_event::BlockMetaEvent,
            },
            Protocol, UnifiedEvent,
        },
        grpc::ClientConfig,
        yellowstone_grpc::{AccountFilter, TransactionFilter},
        YellowstoneGrpc,
    },
};
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Arbitrage Detector...");
    println!("Monitoring Jupiter and Raydium DEXes for arbitrage opportunities");
    println!("================================================\n");

    run_arbitrage_detector().await?;
    Ok(())
}

async fn run_arbitrage_detector() -> Result<(), Box<dyn std::error::Error>> {
    // Create arbitrage detector with:
    // - Minimum 0.5% profit threshold
    // - 30 second maximum quote age
    let detector = Arc::new(Mutex::new(ArbitrageDetector::new(0.5, 30)));

    // Create low-latency configuration
    let mut config: ClientConfig = ClientConfig::low_latency();
    config.enable_metrics = true;

    let grpc = YellowstoneGrpc::new_with_config(
        "https://solana-yellowstone-grpc.publicnode.com:443".to_string(),
        None,
        config,
    )?;

    println!("GRPC client created successfully\n");

    let callback = create_arbitrage_callback(detector.clone());

    // Monitor Jupiter and Raydium protocols
    let protocols = vec![
        Protocol::JupiterAggV6,
        Protocol::RaydiumAmmV4,
        Protocol::RaydiumClmm,
        Protocol::RaydiumCpmm,
    ];

    println!("Monitoring protocols: {:?}\n", protocols);

    // Filter accounts - listen to all DEX programs
    let account_include = vec![
        JUPITER_AGG_V6_PROGRAM_ID.to_string(),
        RAYDIUM_AMM_V4_PROGRAM_ID.to_string(),
        RAYDIUM_CLMM_PROGRAM_ID.to_string(),
        RAYDIUM_CPMM_PROGRAM_ID.to_string(),
    ];

    // Listen to transaction data
    let transaction_filter = TransactionFilter {
        account_include: account_include.clone(),
        account_exclude: vec![],
        account_required: vec![],
    };

    // Listen to account data
    let account_filter = AccountFilter {
        account: vec![],
        owner: account_include.clone(),
        filters: vec![],
    };

    // Event filtering - Include all swap events and fee events
    let event_type_filter = Some(EventTypeFilter {
        include: vec![
            // Jupiter events
            EventType::JupiterAggV6Route,
            EventType::JupiterAggV6ExactOutRoute,
            EventType::JupiterAggV6Fee, // Fee tracking for accurate profit calculation
            // Raydium events
            EventType::RaydiumAmmV4SwapBaseIn,
            EventType::RaydiumAmmV4SwapBaseOut,
            EventType::RaydiumClmmSwap,
            EventType::RaydiumClmmSwapV2,
            EventType::RaydiumCpmmSwapBaseInput,
            EventType::RaydiumCpmmSwapBaseOutput,
        ],
    });

    println!("Starting subscription to DEX events...\n");
    println!("Monitoring programs:");
    println!("  - Jupiter Agg V6:  {}", JUPITER_AGG_V6_PROGRAM_ID);
    println!("  - Raydium AMM V4:  {}", RAYDIUM_AMM_V4_PROGRAM_ID);
    println!("  - Raydium CLMM:    {}", RAYDIUM_CLMM_PROGRAM_ID);
    println!("  - Raydium CPMM:    {}", RAYDIUM_CPMM_PROGRAM_ID);
    println!("\nPress Ctrl+C to stop...\n");
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

    // Auto-stop after 1000 seconds
    let grpc_clone = grpc.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(1000)).await;
        grpc_clone.stop().await;
    });

    // Wait for Ctrl+C
    tokio::signal::ctrl_c().await?;

    // Print final statistics
    println!("\n================================================");
    println!("Shutting down...");
    let detector_lock = detector.lock().unwrap();
    println!("Tracked token pairs: {}", detector_lock.get_tracked_pairs().len());
    println!("================================================");

    Ok(())
}

fn create_arbitrage_callback(
    detector: Arc<Mutex<ArbitrageDetector>>,
) -> impl Fn(Box<dyn UnifiedEvent>) {
    move |event: Box<dyn UnifiedEvent>| {
        let mut opportunities = Vec::new();

        match_event!(event, {
            BlockMetaEvent => |_e: BlockMetaEvent| {
                // Ignore block meta events for arbitrage detection
            },
            // Jupiter Fee Event (for accurate profit calculation)
            JupiterAggV6FeeEvent => |e: JupiterAggV6FeeEvent| {
                println!("💰 Jupiter Fee: {} lamports (mint: {})",
                    e.amount,
                    e.mint
                );

                let mut detector = detector.lock().unwrap();
                detector.process_fee_event(&e);
            },
            // Jupiter Aggregator V6 Route Event
            JupiterAggV6RouteEvent => |e: JupiterAggV6RouteEvent| {
                println!("🔵 Jupiter Swap: {} {} -> {} {}",
                    e.in_amount,
                    e.source_mint,
                    e.quoted_out_amount,
                    e.destination_mint
                );

                let mut detector = detector.lock().unwrap();
                opportunities.extend(detector.process_jupiter_route(&e));
            },
            // Jupiter Aggregator V6 Exact Out Route Event
            JupiterAggV6ExactOutRouteEvent => |e: JupiterAggV6ExactOutRouteEvent| {
                println!("🔵 Jupiter ExactOut Swap: {} {} -> {} {}",
                    e.quoted_in_amount,
                    e.source_mint,
                    e.out_amount,
                    e.destination_mint
                );

                // Convert to route event format for processing
                let route_event = JupiterAggV6RouteEvent {
                    metadata: e.metadata,
                    in_amount: e.quoted_in_amount,
                    quoted_out_amount: e.out_amount,
                    slippage_bps: e.slippage_bps,
                    platform_fee_bps: e.platform_fee_bps,
                    token_program: e.token_program,
                    user_transfer_authority: e.user_transfer_authority,
                    user_source_token_account: e.user_source_token_account,
                    user_destination_token_account: e.user_destination_token_account,
                    destination_token_account: e.destination_token_account,
                    source_mint: e.source_mint,
                    destination_mint: e.destination_mint,
                    platform_fee_account: e.platform_fee_account,
                    event_authority: e.event_authority,
                    program: e.program,
                };

                let mut detector = detector.lock().unwrap();
                opportunities.extend(detector.process_jupiter_route(&route_event));
            },
            // Raydium AMM V4 Swap Event
            RaydiumAmmV4SwapEvent => |e: RaydiumAmmV4SwapEvent| {
                println!("🟣 Raydium AMM V4 Swap: pool {}", e.amm);

                let mut detector = detector.lock().unwrap();
                opportunities.extend(detector.process_raydium_amm_v4_swap(&e));
            },
            // Raydium CLMM Swap Event
            RaydiumClmmSwapEvent => |e: RaydiumClmmSwapEvent| {
                println!("🟣 Raydium CLMM Swap: {} -> {} (pool: {})",
                    e.amount,
                    e.other_amount_threshold,
                    e.pool_state
                );

                let mut detector = detector.lock().unwrap();
                opportunities.extend(detector.process_raydium_clmm_swap(&e));
            },
            // Raydium CLMM Swap V2 Event
            RaydiumClmmSwapV2Event => |e: RaydiumClmmSwapV2Event| {
                println!("🟣 Raydium CLMM V2 Swap: {} -> {} (pool: {})",
                    e.amount,
                    e.other_amount_threshold,
                    e.pool_state
                );

                let mut detector = detector.lock().unwrap();
                opportunities.extend(detector.process_raydium_clmm_swap_v2(&e));
            },
            // Raydium CPMM Swap Event
            RaydiumCpmmSwapEvent => |e: RaydiumCpmmSwapEvent| {
                let (in_amt, out_amt) = if e.amount_in > 0 {
                    (e.amount_in, e.minimum_amount_out)
                } else {
                    (e.max_amount_in, e.amount_out)
                };

                println!("🟣 Raydium CPMM Swap: {} {} -> {} {} (pool: {})",
                    in_amt,
                    e.input_token_mint,
                    out_amt,
                    e.output_token_mint,
                    e.pool_state
                );

                let mut detector = detector.lock().unwrap();
                opportunities.extend(detector.process_raydium_cpmm_swap(&e));
            },
        });

        // Print arbitrage opportunities
        for opp in opportunities {
            print_arbitrage_opportunity(&opp);
        }
    }
}

fn print_arbitrage_opportunity(opp: &ArbitrageOpportunity) {
    // Only show opportunities that are profitable after fees
    let is_profitable = opp.is_profitable_after_fees();
    let icon = if is_profitable { "🚀" } else { "⚠️" };

    println!("\n{} ARBITRAGE OPPORTUNITY DETECTED! {}", icon, icon);
    println!("================================================");
    println!("Token Pair: {} <-> {}", opp.token_pair.base, opp.token_pair.quote);
    println!("Buy on:  {:?} at price {:.6}", opp.buy_dex, opp.buy_price);
    println!("Sell on: {:?} at price {:.6}", opp.sell_dex, opp.sell_price);
    println!("\n--- Profit Analysis ---");
    println!("Gross Profit:     {:.2}%", opp.profit_percentage);
    println!("Total Fees:       {:.2}%", opp.total_fee_percentage);
    println!("Est. Gas Cost:    {:.2}%", opp.estimated_gas_cost / 100.0);
    println!("Net Profit:       {:.2}%", opp.net_profit_percentage);

    if !is_profitable {
        println!("\n⚠️  WARNING: Not profitable after fees and gas costs!");
    }

    // Calculate example profit for 1 SOL (1_000_000_000 lamports)
    let example_input = 1_000_000_000.0;
    let example_gross_profit = opp.calculate_profit(example_input);
    let example_net_profit = opp.calculate_net_profit(example_input);

    println!("\n--- Example (1 SOL input) ---");
    println!("Gross Profit: {:.6} SOL ({:.2} lamports)",
        example_gross_profit / 1_000_000_000.0,
        example_gross_profit
    );
    println!("Net Profit:   {:.6} SOL ({:.2} lamports)",
        example_net_profit / 1_000_000_000.0,
        example_net_profit
    );

    println!("\n--- Quote Details ---");
    println!("Buy:  {} in -> {} out (fee: {}bps)",
        opp.buy_quote.input_amount,
        opp.buy_quote.output_amount,
        opp.buy_quote.platform_fee_bps.unwrap_or(0)
    );
    println!("Sell: {} in -> {} out (fee: {}bps)",
        opp.sell_quote.input_amount,
        opp.sell_quote.output_amount,
        opp.sell_quote.platform_fee_bps.unwrap_or(0)
    );
    println!("================================================\n");
}
