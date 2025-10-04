use dex_idl_parser::prelude::*;
use solana_streamer_sdk::streaming::{
    grpc::ClientConfig,
    yellowstone_grpc::{AccountFilter, TransactionFilter},
    YellowstoneGrpc,
};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::time::{interval, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("   Unified DEX Parser Example (IDL-Based)");
    println!("═══════════════════════════════════════════════════════");
    println!();

    // Initialize the unified DEX parser with all protocols
    let dex_parser = DexStreamParser::new_all_protocols()?;

    println!("Loaded protocols:");
    for program_id in dex_parser.supported_program_ids() {
        println!("  - {}", program_id);
    }
    println!();

    // // Verify Raydium AMM V4 is loaded
    // let raydium_amm_v4_id = "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8";
    // if dex_parser.supported_program_ids().contains(&raydium_amm_v4_id.parse().unwrap()) {
    //     println!("✅ Raydium AMM V4 (675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8) is loaded");
    // } else {
    //     println!("❌ WARNING: Raydium AMM V4 is NOT loaded!");
    // }
    // println!();

    // Create Yellowstone gRPC client
    let mut config = ClientConfig::low_latency();
    config.enable_metrics = true;

    let grpc = YellowstoneGrpc::new_with_config(
        "https://solana-yellowstone-grpc.publicnode.com:443".to_string(),
        None,
        config,
    )?;

    println!("✅ gRPC client created successfully");

    // Setup transaction filter for all supported DEX programs
    let program_ids = dex_parser.supported_program_ids();

    let transaction_filter = TransactionFilter {
        account_include: program_ids.clone(),
        account_exclude: vec![],
        account_required: vec![],
    };

    let account_filter = AccountFilter {
        account: vec![],
        owner: program_ids.clone(),
        filters: vec![],
    };

    println!("Starting to listen for DEX events...");
    println!("Monitoring programs:");
    for (i, program_id) in program_ids.iter().enumerate() {
        println!("  {}. {}", i + 1, program_id);
    }
    println!();

    // Event counters by protocol (using atomic counters for thread-safe updates)
    let event_counters: Arc<HashMap<String, Arc<AtomicU64>>> = Arc::new({
        let mut map = HashMap::new();
        map.insert("Raydium CPMM".to_string(), Arc::new(AtomicU64::new(0)));
        map.insert("Raydium CLMM".to_string(), Arc::new(AtomicU64::new(0)));
        map.insert("Raydium AMM V4".to_string(), Arc::new(AtomicU64::new(0)));
        map.insert("Jupiter Aggregator V6".to_string(), Arc::new(AtomicU64::new(0)));
        map.insert("Orca Whirlpool".to_string(), Arc::new(AtomicU64::new(0)));
        map.insert("Meteora DLMM".to_string(), Arc::new(AtomicU64::new(0)));
        // map.insert("Other".to_string(), Arc::new(AtomicU64::new(0)));
        map
    });

    // Spawn a task to log statistics every 10 seconds
    let stats_counters = Arc::clone(&event_counters);
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(10));
        ticker.tick().await; // Skip the first immediate tick

        loop {
            ticker.tick().await;

            println!("\n┌──────────────────────────────────────────────────────┐");
            println!("│        📊 Event Statistics (Last 10 seconds)        │");
            println!("├──────────────────────────────────────────────────────┤");

            let raydium_cpmm = stats_counters.get("Raydium CPMM").unwrap().swap(0, Ordering::Relaxed);
            let raydium_clmm = stats_counters.get("Raydium CLMM").unwrap().swap(0, Ordering::Relaxed);
            let raydium_amm_v4 = stats_counters.get("Raydium AMM V4").unwrap().swap(0, Ordering::Relaxed);
            let jupiter_agg_v6 = stats_counters.get("Jupiter Aggregator V6").unwrap().swap(0, Ordering::Relaxed);
            let orca_whirlpool = stats_counters.get("Orca Whirlpool").unwrap().swap(0, Ordering::Relaxed);
            let meteora_dlmm = stats_counters.get("Meteora DLMM").unwrap().swap(0, Ordering::Relaxed);
            let other = stats_counters.get("Other").unwrap().swap(0, Ordering::Relaxed);

            let total = raydium_cpmm + raydium_clmm + raydium_amm_v4 + jupiter_agg_v6 + orca_whirlpool + meteora_dlmm + other;

            println!("│  Raydium CPMM:          {:>6} events                │", raydium_cpmm);
            println!("│  Raydium CLMM:          {:>6} events                │", raydium_clmm);
            println!("│  Raydium AMM V4:        {:>6} events                │", raydium_amm_v4);
            println!("│  Jupiter Agg V6:        {:>6} events                │", jupiter_agg_v6);
            println!("│  Orca Whirlpool:        {:>6} events                │", orca_whirlpool);
            println!("│  Meteora DLMM:          {:>6} events                │", meteora_dlmm);
            println!("│  Other:                 {:>6} events                │", other);
            println!("├──────────────────────────────────────────────────────┤");
            println!("│  TOTAL:                 {:>6} events                │", total);
            println!("└──────────────────────────────────────────────────────┘\n");
        }
    });

    let callback_counters = Arc::clone(&event_counters);

    // Subscribe to raw gRPC events for custom parsing with DexStreamParser
    grpc.subscribe_raw(
        vec![transaction_filter],
        vec![account_filter],
        None,
        move |update| {
            use yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof;

            if let Some(UpdateOneof::Transaction(tx_update)) = update.update_oneof {
                // Extract transaction info and metadata
                if let Some(grpc_tx) = &tx_update.transaction {
                    let slot = tx_update.slot;
                    let block_time = None; // Block time would come from block meta events

                    // Parse all DEX events from this transaction
                    let events = dex_parser.parse_from_grpc_transaction(grpc_tx, slot, block_time);

                    for event in events {
                        // Increment the counter for this protocol
                        let protocol_name = event.protocol.name().to_string();
                        if let Some(counter) = callback_counters.get(&protocol_name) {
                            counter.fetch_add(1, Ordering::Relaxed);
                        } else {
                            // Log unknown protocol names to help debug
                            eprintln!("⚠️  Unknown protocol: '{}' - adding to Other", protocol_name);
                            if let Some(counter) = callback_counters.get("Other") {
                                counter.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                        println!("═══════════════════════════════════════════════════════");
                        println!("🎯 DEX Event Detected!");
                        println!("═══════════════════════════════════════════════════════");
                        println!("Protocol:     {}", event.protocol.name());
                        println!("Program:      {}", event.program_name());
                        println!("Instruction:  {}", event.instruction_name());
                        println!("Signature:    {}", event.signature);
                        println!("Slot:         {}", event.slot);
                        println!("Block Time:   {}", event.block_time);

                        if let Some(tx_idx) = event.transaction_index {
                            println!("TX Index:     {}", tx_idx);
                        }

                        // Print instruction type
                        if event.is_swap() {
                            println!("Type:         💱 SWAP");
                        } else if event.is_liquidity_provision() {
                            println!("Type:         💰 LIQUIDITY ADD");
                        } else if event.is_liquidity_removal() {
                            println!("Type:         💸 LIQUIDITY REMOVE");
                        } else {
                            println!("Type:         ⚙️  OTHER");
                        }

                        // Print accounts
                        println!("\n📋 Accounts:");
                        for (name, pubkey) in &event.instruction.accounts {
                            println!("  • {:<30} {}", name, pubkey);
                        }

                        // Print instruction data fields
                        println!("\n📊 Instruction Data Fields:");
                        if event.instruction.data.fields.is_empty() {
                            println!("  (No parsed fields available)");
                        } else {
                            for (i, field) in event.instruction.data.fields.iter().enumerate() {
                                // Special handling for routePlan field
                                if field.name == "routePlan" {
                                    if let Some(dex_idl_parser::types::ParsedValue::RoutePlan(steps)) = &field.value {
                                        println!("  {}. routePlan:", i + 1);
                                        for (step_idx, step) in steps.iter().enumerate() {
                                            println!("    Step {}:", step_idx);
                                            println!("      swap: {:?}", step.swap);
                                            println!("      percent: {}", step.percent);
                                            println!("      inputIndex: {}", step.input_index);
                                            println!("      outputIndex: {}", step.output_index);
                                        }
                                    } else {
                                        println!("  {}. {}", i + 1, field);
                                    }
                                } else {
                                    println!("  {}. {}", i + 1, field);
                                }
                            }
                        }

                        println!("\n🔢 Raw Data:");
                        println!("  Discriminator: {}", hex::encode(&event.instruction.raw_discriminator));
                        println!("  Data (hex):    {}", hex::encode(&event.instruction.data.raw_data));
                        println!("═══════════════════════════════════════════════════════");
                        println!();
                    }
                }
            }
        },
    )
    .await?;

    // Keep running until Ctrl+C
    println!("Press Ctrl+C to stop...");
    tokio::signal::ctrl_c().await?;

    println!("\n👋 Shutting down...");
    Ok(())
}
