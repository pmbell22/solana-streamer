use dex_idl_parser::prelude::*;
use log::{debug, info};
use solana_streamer_sdk::streaming::{
    grpc::ClientConfig,
    yellowstone_grpc::{AccountFilter, TransactionFilter},
    YellowstoneGrpc,
};
use solana_sdk::pubkey::Pubkey;
use std::collections::{HashMap, HashSet};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, RwLock,
};
use tokio::time::{interval, Duration};

/// Extract pool address from DEX event based on protocol-specific account names
fn extract_pool_address(event: &DexEvent) -> Option<String> {
    // Different protocols use different account names for pools
    let pool_account_names = [
        "whirlpool",    // Orca Whirlpool
        "pool_state",   // Raydium CLMM
        "lb_pair",      // Meteora DLMM
        "lbPair",       // Meteora DLMM (alternative)
    ];

    for account_name in &pool_account_names {
        if let Some(pool_pubkey) = event.instruction.accounts.get(*account_name) {
            return Some(pool_pubkey.to_string());
        }
    }

    None
}

/// Parse pool account data based on DEX protocol
fn parse_pool_account_data(protocol: &DexProtocol, data: &[u8]) {
    if data.len() < 8 {
        info!("  ⚠️  Data too short to parse (need at least 8 bytes for discriminator)");
        return;
    }

    // Read discriminator (first 8 bytes)
    let discriminator = &data[0..8];
    info!("  Discriminator: {}", hex::encode(discriminator));

    match protocol {
        DexProtocol::OrcaWhirlpool => parse_whirlpool_pool(data),
        DexProtocol::RaydiumClmm => parse_raydium_clmm_pool(data),
        DexProtocol::MeteoraDlmm => parse_meteora_pool(data),
        _ => info!("  ⚠️  Parsing not implemented for this protocol"),
    }
}

/// Parse Orca Whirlpool pool account data
fn parse_whirlpool_pool(data: &[u8]) {
    // Whirlpool account structure (simplified - key fields for arbitrage)
    // See: https://github.com/orca-so/whirlpools
    // Note: Only Whirlpool pools are ~653+ bytes
    // Other accounts (Position, TickArray, Config, etc.) are smaller and should be skipped
    if data.len() < 653 {
        info!("  ⚠️  Not a Whirlpool pool account (size: {} bytes, expected: 653+)", data.len());
        info!("  📝 Likely a Position, TickArray, Config, or other account type - skipping");
        return;
    }

    // Skip discriminator (8 bytes)
    let mut offset = 8;

    // Read whirlpools_config (32 bytes)
    if let Ok(config) = Pubkey::try_from(&data[offset..offset + 32]) {
        info!("  Config:          {}", config);
    }
    offset += 32;

    // Read whirlpool_bump (1 byte)
    offset += 1;

    // Read tick_spacing (2 bytes)
    let tick_spacing = u16::from_le_bytes([data[offset], data[offset + 1]]);
    info!("  Tick Spacing:    {}", tick_spacing);
    offset += 2;

    // Read tick_spacing_seed (2 bytes)
    offset += 2;

    // Read fee_rate (2 bytes)
    let fee_rate = u16::from_le_bytes([data[offset], data[offset + 1]]);
    info!("  Fee Rate:        {} bps", fee_rate);
    offset += 2;

    // Read protocol_fee_rate (2 bytes)
    offset += 2;

    // Read liquidity (16 bytes - u128)
    let liquidity_bytes: [u8; 16] = data[offset..offset + 16].try_into().unwrap_or([0u8; 16]);
    let liquidity = u128::from_le_bytes(liquidity_bytes);
    info!("  Liquidity:       {}", liquidity);
    offset += 16;

    // Read sqrt_price (16 bytes - u128)
    let sqrt_price_bytes: [u8; 16] = data[offset..offset + 16].try_into().unwrap_or([0u8; 16]);
    let sqrt_price = u128::from_le_bytes(sqrt_price_bytes);
    info!("  Sqrt Price:      {}", sqrt_price);
    offset += 16;

    // Read tick_current_index (4 bytes - i32)
    let tick_current = i32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0u8; 4]));
    info!("  Current Tick:    {}", tick_current);
    offset += 4;

    // Skip to token vaults and mints
    offset += 2; // protocol_fee_owed_a
    offset += 8;
    offset += 8;
    offset += 8;

    // Token A vault (32 bytes)
    if let Ok(vault_a) = Pubkey::try_from(&data[offset..offset + 32]) {
        info!("  Token A Vault:   {}", vault_a);
    }
    offset += 32;

    // Token B vault (32 bytes)
    if let Ok(vault_b) = Pubkey::try_from(&data[offset..offset + 32]) {
        info!("  Token B Vault:   {}", vault_b);
    }
    offset += 32;

    // Token A mint (32 bytes)
    if let Ok(mint_a) = Pubkey::try_from(&data[offset..offset + 32]) {
        info!("  Token A Mint:    {}", mint_a);
    }
    offset += 32;

    // Token B mint (32 bytes)
    if let Ok(mint_b) = Pubkey::try_from(&data[offset..offset + 32]) {
        info!("  Token B Mint:    {}", mint_b);
    }
}

/// Parse Raydium CLMM pool account data
fn parse_raydium_clmm_pool(data: &[u8]) {
    // Raydium CLMM PoolState structure
    if data.len() < 1544 {
        info!("  ⚠️  Data too short for Raydium CLMM pool account");
        return;
    }

    let mut offset = 8; // Skip discriminator

    // Read amm_config (32 bytes)
    if let Ok(config) = Pubkey::try_from(&data[offset..offset + 32]) {
        info!("  AMM Config:      {}", config);
    }
    offset += 32;

    // Skip owner (32 bytes)
    offset += 32;

    // Token mint 0 (32 bytes)
    if let Ok(mint_0) = Pubkey::try_from(&data[offset..offset + 32]) {
        info!("  Token Mint 0:    {}", mint_0);
    }
    offset += 32;

    // Token mint 1 (32 bytes)
    if let Ok(mint_1) = Pubkey::try_from(&data[offset..offset + 32]) {
        info!("  Token Mint 1:    {}", mint_1);
    }
    offset += 32;

    // Token vault 0 (32 bytes)
    if let Ok(vault_0) = Pubkey::try_from(&data[offset..offset + 32]) {
        info!("  Token Vault 0:   {}", vault_0);
    }
    offset += 32;

    // Token vault 1 (32 bytes)
    if let Ok(vault_1) = Pubkey::try_from(&data[offset..offset + 32]) {
        info!("  Token Vault 1:   {}", vault_1);
    }
    offset += 32;

    // Skip observation_key (32 bytes)
    offset += 32;

    // Read tick_spacing (2 bytes)
    let tick_spacing = u16::from_le_bytes([data[offset], data[offset + 1]]);
    info!("  Tick Spacing:    {}", tick_spacing);
    offset += 2;

    // Read liquidity (16 bytes - u128)
    let liquidity_bytes: [u8; 16] = data[offset..offset + 16].try_into().unwrap_or([0u8; 16]);
    let liquidity = u128::from_le_bytes(liquidity_bytes);
    info!("  Liquidity:       {}", liquidity);
    offset += 16;

    // Read sqrt_price_x64 (16 bytes - u128)
    let sqrt_price_bytes: [u8; 16] = data[offset..offset + 16].try_into().unwrap_or([0u8; 16]);
    let sqrt_price_x64 = u128::from_le_bytes(sqrt_price_bytes);
    info!("  Sqrt Price X64:  {}", sqrt_price_x64);
    offset += 16;

    // Read tick_current (4 bytes - i32)
    let tick_current = i32::from_le_bytes(data[offset..offset + 4].try_into().unwrap_or([0u8; 4]));
    info!("  Current Tick:    {}", tick_current);
}

/// Parse Meteora DLMM pool account data
fn parse_meteora_pool(data: &[u8]) {
    // Meteora LbPair account structure from IDL
    // Discriminator(8) + StaticParameters(32) + VariableParameters(32) + main fields
    if data.len() < 150 {
        info!("  ⚠️  Data too short for Meteora pool account (need at least 150 bytes)");
        return;
    }

    // === StaticParameters (offset 8-39, 32 bytes) ===
    let base_factor = u16::from_le_bytes(data[8..10].try_into().unwrap_or([0u8; 2]));
    info!("  Base Factor:     {}", base_factor);

    let min_bin_id = i32::from_le_bytes(data[24..28].try_into().unwrap_or([0u8; 4]));
    let max_bin_id = i32::from_le_bytes(data[28..32].try_into().unwrap_or([0u8; 4]));
    info!("  Bin ID Range:    {} to {}", min_bin_id, max_bin_id);

    // === VariableParameters (offset 40-71, 32 bytes) ===
    let index_reference = i32::from_le_bytes(data[48..52].try_into().unwrap_or([0u8; 4]));
    info!("  Index Reference: {} (last swap bin)", index_reference);

    let last_update = i64::from_le_bytes(data[56..64].try_into().unwrap_or([0u8; 8]));
    if last_update > 0 {
        info!("  Last Update:     {}", last_update);
    }

    // === Main LbPair fields (offset 72+) ===
    let pair_type = data[75];
    info!("  Pair Type:       {}", pair_type);

    // ⭐ ACTIVE BIN ID at offset 76 (4 bytes, i32)
    let active_id = i32::from_le_bytes(data[76..80].try_into().unwrap_or([0u8; 4]));
    info!("  Active Bin ID:   {} ⭐", active_id);

    // Bin Step at offset 80 (2 bytes, u16)
    let bin_step = u16::from_le_bytes(data[80..82].try_into().unwrap_or([0u8; 2]));
    info!("  Bin Step:        {}", bin_step);

    // Status at offset 82 (1 byte, u8)
    let status = data[82];
    info!("  Status:          {}", status);

    // Continue reading remaining fields if data is long enough
    if data.len() < 200 {
        return;
    }

    // Skip requireBaseFactorSeed (1), baseFactorSeed (2), activationType (1), creatorPoolOnOffControl (1)
    let offset = 83 + 5; // offset 88

    // tokenXMint: Pubkey (32 bytes)
    if data.len() >= offset + 32 {
        if let Ok(mint_x) = Pubkey::try_from(&data[offset..offset + 32]) {
            info!("  Token X Mint:    {}", mint_x);
        }
    }

    // tokenYMint: Pubkey (32 bytes)
    if data.len() >= offset + 64 {
        if let Ok(mint_y) = Pubkey::try_from(&data[offset + 32..offset + 64]) {
            info!("  Token Y Mint:    {}", mint_y);
        }
    }

    // reserveX: Pubkey (32 bytes)
    if data.len() >= offset + 96 {
        if let Ok(reserve_x) = Pubkey::try_from(&data[offset + 64..offset + 96]) {
            info!("  Reserve X:       {}", reserve_x);
        }
    }

    // reserveY: Pubkey (32 bytes)
    if data.len() >= offset + 128 {
        if let Ok(reserve_y) = Pubkey::try_from(&data[offset + 96..offset + 128]) {
            info!("  Reserve Y:       {}", reserve_y);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging - set RUST_LOG=info to see qualified events
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("═══════════════════════════════════════════════════════");
    println!("   WSOL/USDC Arbitrage Monitor");
    println!("   (OrcaWhirlpool, RaydiumCLMM, Meteora)");
    println!("═══════════════════════════════════════════════════════");
    println!();

    // Token mints for WSOL/USDC filtering
    let wsol_mint: Pubkey = "So11111111111111111111111111111111111111112".parse()?;
    let usdc_mint: Pubkey = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".parse()?;

    println!("Target pair:");
    println!("  WSOL: {}", wsol_mint);
    println!("  USDC: {}", usdc_mint);
    println!();

    // Initialize the unified DEX parser with only the three protocols we need
    let dex_parser = DexStreamParser::new(vec![
        DexProtocol::OrcaWhirlpool,
        DexProtocol::RaydiumClmm,
        DexProtocol::MeteoraDlmm,
    ])?;

    println!("Loaded protocols:");
    for program_id in dex_parser.supported_program_ids() {
        println!("  - {}", program_id);
    }
    println!();

    // Pool address cache for WSOL/USDC pairs
    // Maps pool address -> (protocol_name, is_wsol_usdc_pair)
    let wsol_usdc_pools: Arc<RwLock<HashSet<Pubkey>>> = Arc::new(RwLock::new(HashSet::new()));

    // Target event names for pool state changes
    let target_events: HashSet<String> = [
        // Orca Whirlpool
        "Traded",
        "LiquidityIncreased",
        "LiquidityDecreased",
        "PoolInitialized",
        // Raydium CLMM
        "SwapEvent",
        "LiquidityChangeEvent",
        "IncreaseLiquidityEvent",
        "DecreaseLiquidityEvent",
        "LiquidityCalculateEvent",
        "PoolCreatedEvent",
        "LpChangeEvent",
        "PoolState",
        // Meteora DLMM
        "Swap",
        "AddLiquidity",
        "RemoveLiquidity",
        "LbPairCreate",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();

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
    println!("Target events:");
    for (i, event_name) in target_events.iter().enumerate() {
        println!("  {}. {}", i + 1, event_name);
    }
    println!();

    // Event counters by protocol (using atomic counters for thread-safe updates)
    let event_counters: Arc<HashMap<String, Arc<AtomicU64>>> = Arc::new({
        let mut map = HashMap::new();
        map.insert("Raydium CLMM".to_string(), Arc::new(AtomicU64::new(0)));
        map.insert("Orca Whirlpool".to_string(), Arc::new(AtomicU64::new(0)));
        map.insert("Meteora DLMM".to_string(), Arc::new(AtomicU64::new(0)));
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
            println!("│   📊 WSOL/USDC Pool Events (Last 10 seconds)        │");
            println!("├──────────────────────────────────────────────────────┤");

            let raydium_clmm = stats_counters.get("Raydium CLMM").unwrap().swap(0, Ordering::Relaxed);
            let orca_whirlpool = stats_counters.get("Orca Whirlpool").unwrap().swap(0, Ordering::Relaxed);
            let meteora_dlmm = stats_counters.get("Meteora DLMM").unwrap().swap(0, Ordering::Relaxed);

            let total = raydium_clmm + orca_whirlpool + meteora_dlmm;

            println!("│  Raydium CLMM:          {:>6} events                │", raydium_clmm);
            println!("│  Orca Whirlpool:        {:>6} events                │", orca_whirlpool);
            println!("│  Meteora DLMM:          {:>6} events                │", meteora_dlmm);
            println!("├──────────────────────────────────────────────────────┤");
            println!("│  TOTAL:                 {:>6} events                │", total);
            println!("└──────────────────────────────────────────────────────┘\n");
        }
    });

    let callback_counters = Arc::clone(&event_counters);
    let _callback_pools = Arc::clone(&wsol_usdc_pools);  // Reserved for future WSOL/USDC filtering
    let target_events_clone = target_events.clone();

    // Subscribe to raw gRPC events for custom parsing with DexStreamParser
    grpc.subscribe_raw(
        vec![transaction_filter],
        vec![account_filter],
        None,
        move |update| {
            use yellowstone_grpc_proto::geyser::subscribe_update::UpdateOneof;

            // Log what type of update we received
            match &update.update_oneof {
                Some(UpdateOneof::Transaction(_)) => {
                    // eprintln!("📨 Received transaction update");
                }
                Some(UpdateOneof::Account(_)) => {
                    // eprintln!("📨 Received account update");
                }
                Some(UpdateOneof::Slot(_)) => {
                    // Too verbose, skip
                }
                Some(_other) => {
                    // eprintln!("📨 Received other update: {:?}", _other);
                }
                None => {
                    // eprintln!("⚠️  Received empty update");
                }
            }

            match update.update_oneof {
                Some(UpdateOneof::Transaction(tx_update)) => {
                    // Extract transaction info and metadata
                    if let Some(grpc_tx) = &tx_update.transaction {
                        let slot = tx_update.slot;
                        let block_time = None; // Block time would come from block meta events

                        // Parse all DEX events from this transaction
                        let events = dex_parser.parse_from_grpc_transaction(grpc_tx, slot, block_time);

                        // Debug: log all events received
                        if !events.is_empty() {
                            debug!("📥 Received {} event(s) from transaction", events.len());
                            for (i, event) in events.iter().enumerate() {
                                debug!("  Event {}: {} - {}", i + 1, event.protocol.name(), event.instruction_name());
                            }
                        }

                        for event in events {
                            let instruction_name = event.instruction_name();

                            // Filter: only process target pool state events
                            if !target_events_clone.contains(instruction_name) {
                                debug!("⚠️  Filtering out event: {} (not in target list)", instruction_name);
                                continue;
                            }

                            // Try to extract pool address from the event's accounts
                            let pool_address = extract_pool_address(&event);

                            // For now, we'll process all events since we don't have pool discovery yet
                            // TODO: Add pool discovery mechanism to identify WSOL/USDC pools
                            // if let Some(pool_addr) = pool_address {
                            //     let pools = callback_pools.read().unwrap();
                            //     if !pools.contains(&pool_addr) {
                            //         continue; // Skip pools that aren't WSOL/USDC
                            //     }
                            // }

                            // Increment the counter for this protocol
                            let protocol_name = event.protocol.name().to_string();
                            if let Some(counter) = callback_counters.get(&protocol_name) {
                                counter.fetch_add(1, Ordering::Relaxed);
                            }

                            // Show the actual event type name with an appropriate icon
                            let instruction_name = event.instruction_name();
                            let icon = match instruction_name {
                                // Swap events
                                "SwapEvent" | "Traded" | "Swap" => "💱",
                                // Pool creation events
                                "PoolCreatedEvent" | "PoolInitialized" | "CreatePool" | "LbPairCreate" => "🆕",
                                // Liquidity add events
                                "IncreaseLiquidityEvent" | "LiquidityIncreased" | "AddLiquidity" => "💰",
                                // Liquidity remove events
                                "DecreaseLiquidityEvent" | "LiquidityDecreased" | "RemoveLiquidity" => "💸",
                                // Liquidity change/calculate events
                                "LiquidityChangeEvent" | "LpChangeEvent" | "LiquidityCalculateEvent" => "📊",
                                // Pool state update
                                "PoolState" => "⚙️",
                                // Generic categorization
                                _ => {
                                    if event.is_swap() {
                                        "💱"
                                    } else if event.is_liquidity_provision() {
                                        "💰"
                                    } else if event.is_liquidity_removal() {
                                        "💸"
                                    } else {
                                        "⚙️"
                                    }
                                }
                            };

                            // Log qualified events at INFO level
                            info!("═══════════════════════════════════════════════════════");
                            info!("{} {}", icon, instruction_name.to_uppercase());
                            info!("═══════════════════════════════════════════════════════");
                            info!("Protocol:     {}", event.protocol.name());
                            info!("Instruction:  {}", instruction_name);

                            if let Some(pool_addr) = pool_address {
                                info!("Pool:         {}", pool_addr);
                            }

                            info!("Signature:    {}", event.signature);
                            info!("Slot:         {}", event.slot);

                            // Print ALL instruction data fields for arbitrage detection
                            info!("📊 Event Data (All Fields):");
                            if !event.instruction.data.fields.is_empty() {
                                for field in event.instruction.data.fields.iter() {
                                    info!("  • {:<25} {}", field.name, field.value.as_ref().map(|v| format!("{:?}", v)).unwrap_or("None".to_string()));
                                }
                            } else {
                                info!("  (No fields)");
                            }

                            // Print all accounts involved
                            info!("🔑 Accounts:");
                            for (account_name, account_pubkey) in event.instruction.accounts.iter() {
                                info!("  • {:<25} {}", account_name, account_pubkey);
                            }

                            info!("═══════════════════════════════════════════════════════");
                            info!("");
                        }
                    }
                }
                Some(UpdateOneof::Account(account_update)) => {
                    // Handle account updates (pool state changes)
                    if let Some(account_info) = &account_update.account {
                        // Parse pubkey and owner
                        let pubkey = match Pubkey::try_from(account_info.pubkey.as_slice()) {
                            Ok(pk) => pk,
                            Err(_) => {
                                eprintln!("⚠️  Failed to parse account pubkey");
                                return;
                            }
                        };

                        let owner = match Pubkey::try_from(account_info.owner.as_slice()) {
                            Ok(pk) => pk,
                            Err(_) => {
                                eprintln!("⚠️  Failed to parse owner pubkey");
                                return;
                            }
                        };

                        let events = dex_parser.parse_from_grpc_transaction(grpc_tx, slot, block_time);

                        // Identify which DEX protocol this account belongs to
                        let protocol = if owner.to_string() == DexProtocol::OrcaWhirlpool.program_id() {
                            Some(DexProtocol::OrcaWhirlpool)
                        } else if owner.to_string() == DexProtocol::RaydiumClmm.program_id() {
                            Some(DexProtocol::RaydiumClmm)
                        } else if owner.to_string() == DexProtocol::MeteoraDlmm.program_id() {
                            Some(DexProtocol::MeteoraDlmm)
                        } else {
                            None
                        };

                        let slot = account_update.slot;
                        let is_startup = account_update.is_startup;

                        // Skip if not one of our target protocols
                        let Some(protocol) = protocol else {
                            debug!("⚠️  Skipping account from non-target protocol: {}", owner);
                            return;
                        };

                        // Filter by account size - only process likely pool accounts
                        let min_pool_size = match protocol {
                            DexProtocol::OrcaWhirlpool => 653,
                            DexProtocol::RaydiumClmm => 1544,
                            DexProtocol::MeteoraDlmm => 150,
                            _ => 0,
                        };

                        if account_info.data.len() < min_pool_size {
                            debug!("⏭️  Skipping {} account (size: {} bytes, pool min: {} bytes)",
                                protocol.name(), account_info.data.len(), min_pool_size);
                            return;
                        }

                        // Determine account type based on protocol
                        let account_type = match protocol {
                            DexProtocol::OrcaWhirlpool => "WHIRLPOOL POOL STATE UPDATE",
                            DexProtocol::RaydiumClmm => "RAYDIUM CLMM POOL STATE UPDATE",
                            DexProtocol::MeteoraDlmm => "METEORA DLMM POOL STATE UPDATE",
                            _ => "POOL ACCOUNT UPDATE",
                        };

                        // Log pool state updates at INFO level
                        info!("═══════════════════════════════════════════════════════");
                        info!("📦 {}", account_type);
                        info!("═══════════════════════════════════════════════════════");
                        info!("DEX Protocol: {}", protocol.name());
                        info!("Account Type: {}", account_type);
                        info!("Account:      {}", pubkey);
                        info!("Owner:        {}", owner);
                        info!("Slot:         {}", slot);
                        info!("Is Startup:   {}", is_startup);
                        info!("Data size:    {} bytes", account_info.data.len());
                        info!("Lamports:     {}", account_info.lamports);
                        info!("Executable:   {}", account_info.executable);
                        info!("Rent Epoch:   {}", account_info.rent_epoch);

                        // Parse pool data based on protocol
                        info!("📊 Pool Data:");
                        parse_pool_account_data(&protocol, &account_info.data);

                        // Show raw data (first 256 bytes)
                        info!("🔢 Raw Data (first 256 bytes):");
                        let data_preview = if account_info.data.len() > 256 {
                            &account_info.data[..256]
                        } else {
                            &account_info.data[..]
                        };
                        info!("  {}", hex::encode(data_preview));

                        info!("═══════════════════════════════════════════════════════");
                        info!("");
                    }
                }
                _ => {}
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
