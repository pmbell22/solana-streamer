use market_streaming::prelude::*;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use std::sync::Arc;
use solana_streamer_sdk::streaming::{
    grpc::ClientConfig,
    yellowstone_grpc::{AccountFilter, TransactionFilter},
    YellowstoneGrpc,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logger
    env_logger::init();

    // Initialize rustls crypto provider
    let _ = rustls::crypto::ring::default_provider().install_default().ok();

    // Create state cache
    let state_cache = Arc::new(PoolStateCache::new());

    // Example pool addresses - replace with actual high-TVL pools
    let raydium_sol_usdc = Pubkey::from_str("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK")?;
    let orca_sol_usdc = Pubkey::from_str("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc")?;
    let meteora_sol_usdc = Pubkey::from_str("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo")?;

    // Configure streaming
    let config: StreamConfig = StreamConfig {
        grpc_endpoint: std::env::var("GRPC_ENDPOINT")
            .unwrap_or_else(|_| "https://solana-yellowstone-grpc.publicnode.com:443".to_string()),
        auth_token: std::env::var("GRPC_AUTH_TOKEN").ok(),
        pool_pubkeys: vec![
            raydium_sol_usdc,
            orca_sol_usdc,
            meteora_sol_usdc,
        ],
        protocols: vec![
            DexProtocol::RaydiumClmm,
            DexProtocol::OrcaWhirlpool,
            DexProtocol::MeteoraDlmm,
        ],
        commitment: yellowstone_grpc_proto::prelude::CommitmentLevel::Processed,
    };

    let mut config = ClientConfig::low_latency();
    config.enable_metrics = true;

    let grpc = YellowstoneGrpc::new_with_config(
        "https://grpc.mainnet.solana.tools:443".to_string(),
        None,
        config,
    )?;

    // Create and start pool stream client
    let client = PoolStreamClient::new(config, state_cache.clone());

    println!("Starting DEX pool monitoring...");
    println!("Monitoring {} pools across {} DEXs",
        client.state_cache().len(),
        3
    );
    println!("Press Ctrl+C to stop\n");

    // Spawn a task to periodically print cache statistics
    let cache_clone = state_cache.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
        loop {
            interval.tick().await;
            let stats = cache_clone.stats();
            println!("\n=== Cache Statistics ===");
            println!("Total entries: {}", stats.total_entries);
            println!("Fresh entries: {}", stats.fresh_entries);
            println!("Stale entries: {}", stats.stale_entries);
            println!("Max age: {}ms", stats.max_age_ms);

            // Print current prices
            for (pubkey, cached) in cache_clone.get_all_fresh() {
                let (token_a, token_b) = cached.state.get_token_pair();
                println!(
                    "\nPool: {}\n  Price: {:.8}\n  Liquidity: {}\n  Tokens: {} / {}",
                    pubkey,
                    cached.state.get_price(),
                    cached.state.get_liquidity(),
                    token_a,
                    token_b
                );
            }
            println!("========================\n");
        }
    });

    // Start streaming (this will run indefinitely)
    client.start().await?;

    Ok(())
}
