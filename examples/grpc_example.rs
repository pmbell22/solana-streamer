use solana_streamer_sdk::{
    match_event,
    streaming::{
        event_parser::{
            common::{filter::EventTypeFilter, EventType},
            core::account_event_parser::{NonceAccountEvent, TokenAccountEvent, TokenInfoEvent},
            protocols::{
                raydium_amm_v4::{
                    parser::RAYDIUM_AMM_V4_PROGRAM_ID, RaydiumAmmV4AmmInfoAccountEvent,
                    RaydiumAmmV4DepositEvent, RaydiumAmmV4Initialize2Event, RaydiumAmmV4SwapEvent,
                    RaydiumAmmV4WithdrawEvent, RaydiumAmmV4WithdrawPnlEvent,
                },
                raydium_clmm::{
                    parser::RAYDIUM_CLMM_PROGRAM_ID, RaydiumClmmAmmConfigAccountEvent,
                    RaydiumClmmClosePositionEvent, RaydiumClmmCreatePoolEvent,
                    RaydiumClmmDecreaseLiquidityV2Event, RaydiumClmmIncreaseLiquidityV2Event,
                    RaydiumClmmOpenPositionV2Event, RaydiumClmmOpenPositionWithToken22NftEvent,
                    RaydiumClmmPoolStateAccountEvent, RaydiumClmmSwapEvent, RaydiumClmmSwapV2Event,
                    RaydiumClmmTickArrayStateAccountEvent,
                },
                raydium_cpmm::{
                    parser::RAYDIUM_CPMM_PROGRAM_ID, RaydiumCpmmAmmConfigAccountEvent,
                    RaydiumCpmmDepositEvent, RaydiumCpmmInitializeEvent,
                    RaydiumCpmmPoolStateAccountEvent, RaydiumCpmmSwapEvent,
                    RaydiumCpmmWithdrawEvent,
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
    println!("Starting Yellowstone gRPC Streamer...");
    test_grpc().await?;
    Ok(())
}

async fn test_grpc() -> Result<(), Box<dyn std::error::Error>> {
    println!("Subscribing to Yellowstone gRPC events...");

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

    // Will try to parse corresponding protocol events from transactions
    let protocols = vec![
        Protocol::RaydiumCpmm,
        Protocol::RaydiumClmm,
        Protocol::RaydiumAmmV4,
    ];

    println!("Protocols to monitor: {:?}", protocols);

    // Filter accounts
    let account_include = vec![
        RAYDIUM_CPMM_PROGRAM_ID.to_string(),   // Listen to raydium_cpmm program ID
        RAYDIUM_CLMM_PROGRAM_ID.to_string(),   // Listen to raydium_clmm program ID
        RAYDIUM_AMM_V4_PROGRAM_ID.to_string(), // Listen to raydium_amm_v4 program ID
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
    let account_filter = AccountFilter { account: vec![], owner: account_include.clone(), filters: vec![] };

    // Event filtering
    // Include all Raydium event types
    let event_type_filter = Some(EventTypeFilter {
        include: vec![
            // Raydium CPMM events
            EventType::RaydiumCpmmSwapBaseInput,
            EventType::RaydiumCpmmSwapBaseOutput,
            EventType::RaydiumCpmmDeposit,
            EventType::RaydiumCpmmInitialize,
            EventType::RaydiumCpmmWithdraw,
            // Raydium CLMM events
            EventType::RaydiumClmmSwap,
            EventType::RaydiumClmmSwapV2,
            EventType::RaydiumClmmClosePosition,
            EventType::RaydiumClmmIncreaseLiquidityV2,
            EventType::RaydiumClmmDecreaseLiquidityV2,
            EventType::RaydiumClmmCreatePool,
            EventType::RaydiumClmmOpenPositionWithToken22Nft,
            EventType::RaydiumClmmOpenPositionV2,
            // Raydium AMM V4 events
            EventType::RaydiumAmmV4SwapBaseIn,
            EventType::RaydiumAmmV4SwapBaseOut,
            EventType::RaydiumAmmV4Deposit,
            EventType::RaydiumAmmV4Initialize2,
            EventType::RaydiumAmmV4Withdraw,
            EventType::RaydiumAmmV4WithdrawPnl,
        ],
    });
    // Only include PumpSwapBuy events and PumpSwapSell events
    // let event_type_filter = Some(EventTypeFilter { include: vec![EventType::PumpFunTrade] });

    println!("Starting to listen for events, press Ctrl+C to stop...");
    println!("Monitoring programs: {:?}", account_include);

    println!("Starting subscription...");

    grpc.subscribe_events_immediate(
        protocols,
        None, // bot_wallet parameter
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
            "🎉 Event received! Type: {:?}, transaction_index: {:?}",
            event.event_type(),
            event.transaction_index()
        );
        match_event!(event, {
            // -------------------------- block meta -----------------------
            BlockMetaEvent => |e: BlockMetaEvent| {
                println!("BlockMetaEvent: {:?}", e.metadata.handle_us);
            },
            // -------------------------- raydium_cpmm -----------------------
            RaydiumCpmmSwapEvent => |e: RaydiumCpmmSwapEvent| {
                println!("RaydiumCpmmSwapEvent: {e:?}");
            },
            RaydiumCpmmDepositEvent => |e: RaydiumCpmmDepositEvent| {
                println!("RaydiumCpmmDepositEvent: {e:?}");
            },
            RaydiumCpmmInitializeEvent => |e: RaydiumCpmmInitializeEvent| {
                println!("RaydiumCpmmInitializeEvent: {e:?}");
            },
            RaydiumCpmmWithdrawEvent => |e: RaydiumCpmmWithdrawEvent| {
                println!("RaydiumCpmmWithdrawEvent: {e:?}");
            },
            // -------------------------- raydium_clmm -----------------------
            RaydiumClmmSwapEvent => |e: RaydiumClmmSwapEvent| {
                println!("RaydiumClmmSwapEvent: {e:?}");
            },
            RaydiumClmmSwapV2Event => |e: RaydiumClmmSwapV2Event| {
                println!("RaydiumClmmSwapV2Event: {e:?}");
            },
            RaydiumClmmClosePositionEvent => |e: RaydiumClmmClosePositionEvent| {
                println!("RaydiumClmmClosePositionEvent: {e:?}");
            },
            RaydiumClmmDecreaseLiquidityV2Event => |e: RaydiumClmmDecreaseLiquidityV2Event| {
                println!("RaydiumClmmDecreaseLiquidityV2Event: {e:?}");
            },
            RaydiumClmmCreatePoolEvent => |e: RaydiumClmmCreatePoolEvent| {
                println!("RaydiumClmmCreatePoolEvent: {e:?}");
            },
            RaydiumClmmIncreaseLiquidityV2Event => |e: RaydiumClmmIncreaseLiquidityV2Event| {
                println!("RaydiumClmmIncreaseLiquidityV2Event: {e:?}");
            },
            RaydiumClmmOpenPositionWithToken22NftEvent => |e: RaydiumClmmOpenPositionWithToken22NftEvent| {
                println!("RaydiumClmmOpenPositionWithToken22NftEvent: {e:?}");
            },
            RaydiumClmmOpenPositionV2Event => |e: RaydiumClmmOpenPositionV2Event| {
                println!("RaydiumClmmOpenPositionV2Event: {e:?}");
            },
            // -------------------------- raydium_amm_v4 -----------------------
            RaydiumAmmV4SwapEvent => |e: RaydiumAmmV4SwapEvent| {
                println!("RaydiumAmmV4SwapEvent: {e:?}");
            },
            RaydiumAmmV4DepositEvent => |e: RaydiumAmmV4DepositEvent| {
                println!("RaydiumAmmV4DepositEvent: {e:?}");
            },
            RaydiumAmmV4Initialize2Event => |e: RaydiumAmmV4Initialize2Event| {
                println!("RaydiumAmmV4Initialize2Event: {e:?}");
            },
            RaydiumAmmV4WithdrawEvent => |e: RaydiumAmmV4WithdrawEvent| {
                println!("RaydiumAmmV4WithdrawEvent: {e:?}");
            },
            RaydiumAmmV4WithdrawPnlEvent => |e: RaydiumAmmV4WithdrawPnlEvent| {
                println!("RaydiumAmmV4WithdrawPnlEvent: {e:?}");
            },
            RaydiumAmmV4AmmInfoAccountEvent => |e: RaydiumAmmV4AmmInfoAccountEvent| {
                println!("RaydiumAmmV4AmmInfoAccountEvent: {e:?}");
            },
            RaydiumClmmAmmConfigAccountEvent => |e: RaydiumClmmAmmConfigAccountEvent| {
                println!("RaydiumClmmAmmConfigAccountEvent: {e:?}");
            },
            RaydiumClmmPoolStateAccountEvent => |e: RaydiumClmmPoolStateAccountEvent| {
                println!("RaydiumClmmPoolStateAccountEvent: {e:?}");
            },
            RaydiumClmmTickArrayStateAccountEvent => |e: RaydiumClmmTickArrayStateAccountEvent| {
                println!("RaydiumClmmTickArrayStateAccountEvent: {e:?}");
            },
            RaydiumCpmmAmmConfigAccountEvent => |e: RaydiumCpmmAmmConfigAccountEvent| {
                println!("RaydiumCpmmAmmConfigAccountEvent: {e:?}");
            },
            RaydiumCpmmPoolStateAccountEvent => |e: RaydiumCpmmPoolStateAccountEvent| {
                println!("RaydiumCpmmPoolStateAccountEvent: {e:?}");
            },
            TokenAccountEvent => |e: TokenAccountEvent| {
                println!("TokenAccountEvent: {e:?}");
            },
            NonceAccountEvent => |e: NonceAccountEvent| {
                println!("NonceAccountEvent: {e:?}");
            },
            TokenInfoEvent => |e: TokenInfoEvent| {
                println!("TokenInfoEvent: {e:?}");
            },
        });
    }
}
