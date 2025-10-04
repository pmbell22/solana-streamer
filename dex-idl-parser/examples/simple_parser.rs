use dex_idl_parser::prelude::*;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("   DEX IDL Parser - Simple Example");
    println!("═══════════════════════════════════════════════════════\n");

    // Initialize the unified DEX parser with Jupiter and Raydium (Orca IDL currently unavailable)
    let dex_parser = DexStreamParser::new(vec![
        DexProtocol::JupiterV6,
        DexProtocol::RaydiumClmm,
        // DexProtocol::OrcaWhirlpool, // TODO: Fix Orca IDL download
    ])?;

    println!("✅ Loaded {} protocols:", dex_parser.supported_program_ids().len());
    for (i, program_id) in dex_parser.supported_program_ids().iter().enumerate() {
        println!("   {}. {}", i + 1, program_id);
    }
    println!();

    // Example: Parse a Jupiter route instruction
    println!("─────────────────────────────────────────────────────");
    println!("Example 1: Jupiter Aggregator V6");
    println!("─────────────────────────────────────────────────────");

    let jupiter_parser = dex_parser
        .get_parser(&DexProtocol::JupiterV6)
        .expect("Jupiter parser should exist");

    if let Some(route_disc) = jupiter_parser.get_discriminator("route") {
        println!("Route instruction discriminator: {}", hex::encode(route_disc));
    }

    if let Some(route_inst) = jupiter_parser.get_instruction("route") {
        println!("\nRoute instruction accounts:");
        for (i, account) in route_inst.accounts.iter().enumerate() {
            println!("  {}. {} (mut: {}, signer: {})",
                i, account.name, account.is_mut, account.is_signer);
        }
    }

    // Example: Parse a Raydium CLMM instruction
    println!("\n─────────────────────────────────────────────────────");
    println!("Example 2: Raydium CLMM");
    println!("─────────────────────────────────────────────────────");

    let raydium_parser = dex_parser
        .get_parser(&DexProtocol::RaydiumClmm)
        .expect("Raydium parser should exist");

    if let Some(swap_disc) = raydium_parser.get_discriminator("swap_v2") {
        println!("Swap V2 instruction discriminator: {}", hex::encode(swap_disc));
    }

    if let Some(swap_inst) = raydium_parser.get_instruction("swap_v2") {
        println!("\nSwap V2 instruction accounts:");
        for (i, account) in swap_inst.accounts.iter().enumerate() {
            println!("  {}. {} (mut: {}, signer: {})",
                i, account.name, account.is_mut, account.is_signer);
        }

        println!("\nSwap V2 instruction arguments:");
        for (i, arg) in swap_inst.args.iter().enumerate() {
            println!("  {}. {}: {:?}", i, arg.name, arg.ty);
        }
    }

    // Demonstrate instruction lookup by program ID
    println!("\n─────────────────────────────────────────────────────");
    println!("Example 4: Program ID Lookup");
    println!("─────────────────────────────────────────────────────");

    let jupiter_program_id = Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4")?;
    println!("Jupiter Program ID: {}", jupiter_program_id);

    // Simulate parsing an instruction (with dummy data)
    let dummy_instruction_data = vec![
        0xce, 0xfd, 0xbd, 0xbf, 0x3b, 0x33, 0x60, 0x4c, // discriminator (example)
        0x00, 0x00, 0x00, 0x00, // route plan length
        0x10, 0x27, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // in_amount
        0x20, 0x4e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // quoted_out_amount
        0x32, 0x00, // slippage_bps (50 bps = 0.5%)
        0x00, // platform_fee_bps
    ];

    let dummy_accounts = vec![
        Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?, // Token program
        Pubkey::from_str("11111111111111111111111111111111")?, // System program
        // ... more accounts would be here in a real transaction
    ];

    println!("\nAttempting to parse dummy Jupiter instruction...");
    match dex_parser.parse_instruction(
        &jupiter_program_id,
        &dummy_instruction_data,
        &dummy_accounts,
        "DummySignature123".to_string(),
        123456,
        1234567890,
        Some(0),
    ) {
        Ok(event) => {
            println!("✅ Successfully parsed!");
            println!("   Protocol: {}", event.protocol.name());
            println!("   Program: {}", event.program_name());
            println!("   Instruction: {}", event.instruction_name());
            println!("   Is Swap: {}", event.is_swap());
        }
        Err(e) => {
            println!("ℹ️  Parse result: {} (expected for dummy data)", e);
        }
    }

    println!("\n═══════════════════════════════════════════════════════");
    println!("✨ Examples completed successfully!");
    println!("═══════════════════════════════════════════════════════");

    Ok(())
}
