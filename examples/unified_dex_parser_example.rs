use dex_idl_parser::prelude::*;
use solana_streamer_sdk::streaming::{
    grpc::ClientConfig,
    yellowstone_grpc::{AccountFilter, TransactionFilter},
    YellowstoneGrpc,
};

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
                                println!("  {}. {}", i + 1, field);
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
