use solana_sdk::pubkey::Pubkey;

use crate::streaming::event_parser::{
    common::{read_u64_le, read_u8, EventMetadata, EventType, ProtocolType},
    core::event_parser::GenericEventParseConfig,
    protocols::jupiter_agg_v6::{
        discriminators, JupiterAggV6RouteEvent, JupiterAggV6ExactOutRouteEvent,
    },
    UnifiedEvent,
};

/// Jupiter Aggregator V6 Program ID
pub const JUPITER_AGG_V6_PROGRAM_ID: Pubkey =
    solana_sdk::pubkey!("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");

// Configure all event types
pub const CONFIGS: &[GenericEventParseConfig] = &[
    GenericEventParseConfig {
        program_id: JUPITER_AGG_V6_PROGRAM_ID,
        protocol_type: ProtocolType::JupiterAggV6,
        inner_instruction_discriminator: &[],
        instruction_discriminator: discriminators::ROUTE,
        event_type: EventType::JupiterAggV6Route,
        inner_instruction_parser: None,
        instruction_parser: Some(parse_route_instruction),
        requires_inner_instruction: false,
    },
    GenericEventParseConfig {
        program_id: JUPITER_AGG_V6_PROGRAM_ID,
        protocol_type: ProtocolType::JupiterAggV6,
        inner_instruction_discriminator: &[],
        instruction_discriminator: discriminators::EXACT_OUT_ROUTE,
        event_type: EventType::JupiterAggV6ExactOutRoute,
        inner_instruction_parser: None,
        instruction_parser: Some(parse_exact_out_route_instruction),
        requires_inner_instruction: false,
    },
];

/// Parse route instruction event
fn parse_route_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    // Route instruction structure:
    // - route_plan: Vec<RoutePlanStep> (variable length)
    // - in_amount: u64
    // - quoted_out_amount: u64
    // - slippage_bps: u64
    // - platform_fee_bps: u8

    // Minimum accounts expected: token_program, user_transfer_authority,
    // user_source_token_account, user_destination_token_account,
    // destination_token_account, source_mint, destination_mint,
    // platform_fee_account, event_authority, program
    if accounts.len() < 10 {
        return None;
    }

    // The data starts with a variable-length route_plan vector
    // We need to skip it to get to the fixed fields
    // Vector format in Borsh: length (4 bytes) + elements
    if data.len() < 4 {
        return None;
    }

    let _vec_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

    // Each RoutePlanStep is variable size due to nested Swap enum
    // For simplicity, we'll estimate and look for our fixed fields at the end
    // The last 25 bytes should be: in_amount(8) + quoted_out_amount(8) + slippage_bps(8) + platform_fee_bps(1)
    if data.len() < 25 {
        return None;
    }

    let fixed_data_start = data.len() - 25;
    let in_amount = read_u64_le(data, fixed_data_start)?;
    let quoted_out_amount = read_u64_le(data, fixed_data_start + 8)?;
    let slippage_bps = read_u64_le(data, fixed_data_start + 16)?;
    let platform_fee_bps = read_u8(data, fixed_data_start + 24)?;

    Some(Box::new(JupiterAggV6RouteEvent {
        metadata,
        in_amount,
        quoted_out_amount,
        slippage_bps,
        platform_fee_bps,
        token_program: accounts[0],
        user_transfer_authority: accounts[1],
        user_source_token_account: accounts[2],
        user_destination_token_account: accounts[3],
        destination_token_account: accounts[4],
        source_mint: accounts[5],
        destination_mint: accounts[6],
        platform_fee_account: accounts[7],
        event_authority: accounts[8],
        program: accounts[9],
    }))
}

/// Parse exact out route instruction event
fn parse_exact_out_route_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    // Similar to route instruction but with different parameter meanings
    if accounts.len() < 10 {
        return None;
    }

    if data.len() < 4 {
        return None;
    }

    let _vec_len = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;

    // The last 25 bytes should be: out_amount(8) + quoted_in_amount(8) + slippage_bps(8) + platform_fee_bps(1)
    if data.len() < 25 {
        return None;
    }

    let fixed_data_start = data.len() - 25;
    let out_amount = read_u64_le(data, fixed_data_start)?;
    let quoted_in_amount = read_u64_le(data, fixed_data_start + 8)?;
    let slippage_bps = read_u64_le(data, fixed_data_start + 16)?;
    let platform_fee_bps = read_u8(data, fixed_data_start + 24)?;

    Some(Box::new(JupiterAggV6ExactOutRouteEvent {
        metadata,
        out_amount,
        quoted_in_amount,
        slippage_bps,
        platform_fee_bps,
        token_program: accounts[0],
        user_transfer_authority: accounts[1],
        user_source_token_account: accounts[2],
        user_destination_token_account: accounts[3],
        destination_token_account: accounts[4],
        source_mint: accounts[5],
        destination_mint: accounts[6],
        platform_fee_account: accounts[7],
        event_authority: accounts[8],
        program: accounts[9],
    }))
}
