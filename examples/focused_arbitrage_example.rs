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
                raydium_clmm::{
                    events::{RaydiumClmmSwapEvent, RaydiumClmmSwapV2Event},
                    parser::RAYDIUM_CLMM_PROGRAM_ID,
                },
                raydium_cpmm::{
                    events::RaydiumCpmmSwapEvent,
                    parser::RAYDIUM_CPMM_PROGRAM_ID,
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
use solana_sdk::pubkey::Pubkey;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

/// Configuration for token pairs to monitor
#[derive(Clone)]
pub struct TokenPairConfig {
    pub name: String,
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub pools: Vec<PoolInfo>,
}

#[derive(Clone)]
pub struct PoolInfo {
    pub dex: String,
    pub pool_address: Pubkey,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Starting Focused Arbitrage Detector...");
    println!("Monitoring specific token pairs only");
    println!("================================================\n");

    run_focused_arbitrage_detector().await?;
    Ok(())
}

async fn run_focused_arbitrage_detector() -> Result<(), Box<dyn std::error::Error>> {
    // Define specific token pairs to monitor
    let token_pairs = get_monitored_token_pairs();

    println!("Monitoring {} token pairs:", token_pairs.len());
    for pair in &token_pairs {
        println!("  - {} ({} pools)", pair.name, pair.pools.len());
    }
    println!();

    // Create arbitrage detector
    let detector = Arc::new(Mutex::new(ArbitrageDetector::new(0.3, 30)));

    // Create token pair filter for post-processing
    let monitored_pairs = Arc::new(create_token_pair_filter(&token_pairs));

    // Extract pool addresses to subscribe to
    let pool_addresses: Vec<String> = token_pairs
        .iter()
        .flat_map(|pair| pair.pools.iter().map(|p| p.pool_address.to_string()))
        .collect();

    println!("Subscribing to {} specific pool accounts", pool_addresses.len());

    // Create GRPC client
    let mut config = ClientConfig::low_latency();
    config.enable_metrics = true;

    let grpc = YellowstoneGrpc::new_with_config(
        "https://solana-yellowstone-grpc.publicnode.com:443".to_string(),
        None,
        config,
    )?;

    let callback = create_focused_arbitrage_callback(
        detector.clone(),
        monitored_pairs.clone(),
    );

    let protocols = vec![
        Protocol::JupiterAggV6,
        Protocol::RaydiumClmm,
        Protocol::RaydiumCpmm,
    ];

    // Transaction filter - only transactions involving our token mints
    let token_mints: HashSet<String> = token_pairs
        .iter()
        .flat_map(|pair| vec![pair.token_a.to_string(), pair.token_b.to_string()])
        .collect();

    let transaction_filter = TransactionFilter {
        account_include: token_mints.iter().cloned().collect(),
        account_exclude: vec![],
        account_required: vec![],
    };

    // Account filter - if no specific pools, subscribe to accounts owned by DEX programs
    // Otherwise, subscribe to specific pool accounts
    let program_ids = vec![
        JUPITER_AGG_V6_PROGRAM_ID.to_string(),
        RAYDIUM_CLMM_PROGRAM_ID.to_string(),
        RAYDIUM_CPMM_PROGRAM_ID.to_string(),
    ];

    let account_filters = if pool_addresses.is_empty() {
        println!("No specific pools configured - using program owner filtering");
        vec![AccountFilter {
            account: vec![],
            owner: program_ids,
            filters: vec![],
        }]
    } else {
        println!("Using pool-specific filtering for {} pools", pool_addresses.len());
        vec![AccountFilter {
            account: pool_addresses.clone(),
            owner: vec![],
            filters: vec![],
        }]
    };

    // Event type filter
    let event_type_filter = Some(EventTypeFilter {
        include: vec![
            EventType::JupiterAggV6Route,
            EventType::JupiterAggV6ExactOutRoute,
            EventType::JupiterAggV6Fee,
            EventType::RaydiumClmmSwap,
            EventType::RaydiumClmmSwapV2,
            EventType::RaydiumCpmmSwapBaseInput,
            EventType::RaydiumCpmmSwapBaseOutput,
        ],
    });

    println!("Starting subscription...\n");
    println!("Monitored tokens ({}):", token_mints.len());
    for mint in token_mints.iter().take(10) {
        println!("  {}", mint);
    }
    if token_mints.len() > 10 {
        println!("  ... and {} more", token_mints.len() - 10);
    }
    println!("\nPress Ctrl+C to stop...\n");
    println!("================================================\n");

    grpc.subscribe_events_immediate(
        protocols,
        None,
        vec![transaction_filter],
        account_filters,
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

    println!("\n================================================");
    println!("Shutting down...");
    let detector_lock = detector.lock().unwrap();
    println!("Tracked token pairs: {}", detector_lock.get_tracked_pairs().len());
    println!("================================================");

    Ok(())
}

/// Define the token pairs you want to monitor
fn get_monitored_token_pairs() -> Vec<TokenPairConfig> {
    // Common Solana token addresses
    let sol = Pubkey::from_str("So11111111111111111111111111111111111111112").unwrap();
    let usdc = Pubkey::from_str("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap();
    let usdt = Pubkey::from_str("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB").unwrap();
    let bonk = Pubkey::from_str("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263").unwrap();
    let jup = Pubkey::from_str("JUPyiwrYJFskUPiHa7hkeR8VUtAeFoSYbKedZNsDvCN").unwrap();

    vec![
        TokenPairConfig {
            name: "SOL/USDC".to_string(),
            token_a: sol,
            token_b: usdc,
            pools: vec![
                // Add known pool addresses here for each DEX
                // You can find these on-chain or from DEX APIs
                // Example:
                // PoolInfo {
                //     dex: "Raydium CPMM".to_string(),
                //     pool_address: Pubkey::from_str("...").unwrap(),
                // },
            ],
        },
        TokenPairConfig {
            name: "SOL/USDT".to_string(),
            token_a: sol,
            token_b: usdt,
            pools: vec![],
        },
        TokenPairConfig {
            name: "BONK/SOL".to_string(),
            token_a: bonk,
            token_b: sol,
            pools: vec![],
        },
        TokenPairConfig {
            name: "JUP/USDC".to_string(),
            token_a: jup,
            token_b: usdc,
            pools: vec![],
        },
    ]
}

/// Create a set of monitored token pairs for filtering
fn create_token_pair_filter(configs: &[TokenPairConfig]) -> HashSet<(Pubkey, Pubkey)> {
    let mut pairs = HashSet::new();
    for config in configs {
        // Add both orderings since TokenPair normalizes them
        pairs.insert((config.token_a, config.token_b));
        pairs.insert((config.token_b, config.token_a));
    }
    pairs
}

fn create_focused_arbitrage_callback(
    detector: Arc<Mutex<ArbitrageDetector>>,
    monitored_pairs: Arc<HashSet<(Pubkey, Pubkey)>>,
) -> impl Fn(Box<dyn UnifiedEvent>) {
    move |event: Box<dyn UnifiedEvent>| {
        let mut opportunities = Vec::new();

        match_event!(event, {
            BlockMetaEvent => |_e: BlockMetaEvent| {
                // Ignore block meta events
            },
            JupiterAggV6FeeEvent => |e: JupiterAggV6FeeEvent| {
                let mut detector = detector.lock().unwrap();
                detector.process_fee_event(&e);
            },
            JupiterAggV6RouteEvent => |e: JupiterAggV6RouteEvent| {
                // Check if this pair is in our monitored list
                if monitored_pairs.contains(&(e.source_mint, e.destination_mint)) {
                    println!("🔵 Jupiter Swap [MONITORED]: {} -> {} ({} -> {})",
                        e.source_mint,
                        e.destination_mint,
                        e.in_amount,
                        e.quoted_out_amount
                    );

                    let mut detector = detector.lock().unwrap();
                    opportunities.extend(detector.process_jupiter_route(&e));
                }
            },
            JupiterAggV6ExactOutRouteEvent => |e: JupiterAggV6ExactOutRouteEvent| {
                if monitored_pairs.contains(&(e.source_mint, e.destination_mint)) {
                    println!("🔵 Jupiter ExactOut Swap [MONITORED]: {} -> {}",
                        e.source_mint,
                        e.destination_mint
                    );

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
                }
            },
            RaydiumClmmSwapEvent => |e: RaydiumClmmSwapEvent| {
                let mut detector = detector.lock().unwrap();
                opportunities.extend(detector.process_raydium_clmm_swap(&e));
            },
            RaydiumClmmSwapV2Event => |e: RaydiumClmmSwapV2Event| {
                if monitored_pairs.contains(&(e.input_vault_mint, e.output_vault_mint)) {
                    println!("🟣 Raydium CLMM V2 Swap [MONITORED]: {} -> {} ({} -> {})",
                        e.input_vault_mint,
                        e.output_vault_mint,
                        e.amount,
                        e.other_amount_threshold
                    );

                    let mut detector = detector.lock().unwrap();
                    opportunities.extend(detector.process_raydium_clmm_swap_v2(&e));
                }
            },
            RaydiumCpmmSwapEvent => |e: RaydiumCpmmSwapEvent| {
                if monitored_pairs.contains(&(e.input_token_mint, e.output_token_mint)) {
                    let (in_amt, out_amt) = if e.amount_in > 0 {
                        (e.amount_in, e.minimum_amount_out)
                    } else {
                        (e.max_amount_in, e.amount_out)
                    };

                    println!("🟣 Raydium CPMM Swap [MONITORED]: {} -> {} ({} -> {})",
                        e.input_token_mint,
                        e.output_token_mint,
                        in_amt,
                        out_amt
                    );

                    let mut detector = detector.lock().unwrap();
                    opportunities.extend(detector.process_raydium_cpmm_swap(&e));
                }
            },
        });

        // Print arbitrage opportunities
        for opp in opportunities {
            print_arbitrage_opportunity(&opp);
        }
    }
}

fn print_arbitrage_opportunity(opp: &ArbitrageOpportunity) {
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

    println!("================================================\n");
}
