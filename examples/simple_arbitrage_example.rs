use solana_streamer_sdk::{
    match_event,
    streaming::{
        arbitrage::{ArbitrageDetector, ArbitrageOpportunity},
        event_parser::{
            common::{filter::EventTypeFilter, EventType},
            protocols::{
                raydium_amm_v4::{events::RaydiumAmmV4SwapEvent, parser::RAYDIUM_AMM_V4_PROGRAM_ID},
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
    println!("Starting Raydium Arbitrage Detector...");
    println!("Monitoring Raydium DEXes for arbitrage opportunities");
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

    // Monitor Raydium protocols
    let protocols = vec![
        Protocol::RaydiumAmmV4,
        Protocol::RaydiumClmm,
        Protocol::RaydiumCpmm,
    ];

    println!("Monitoring protocols: {:?}\n", protocols);

    // Filter accounts - listen to Raydium DEX programs
    let account_include = vec![
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

    // Event filtering - Include all swap events
    let event_type_filter = Some(EventTypeFilter {
        include: vec![
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
    println!("\n🚀 ARBITRAGE OPPORTUNITY DETECTED! 🚀");
    println!("================================================");
    println!("Token Pair: {} <-> {}", opp.token_pair.base, opp.token_pair.quote);
    println!("Buy on:  {:?} at price {:.6}", opp.buy_dex, opp.buy_price);
    println!("Sell on: {:?} at price {:.6}", opp.sell_dex, opp.sell_price);
    println!("Profit:  {:.2}%", opp.profit_percentage);

    // Calculate example profit for 1000 units
    let example_input = 1000.0;
    let example_profit = opp.calculate_profit(example_input);
    println!("Example: {} units input -> {:.2} units profit", example_input, example_profit);

    println!("Buy Quote:  {} in -> {} out", opp.buy_quote.input_amount, opp.buy_quote.output_amount);
    println!("Sell Quote: {} in -> {} out", opp.sell_quote.input_amount, opp.sell_quote.output_amount);
    println!("================================================\n");
}
