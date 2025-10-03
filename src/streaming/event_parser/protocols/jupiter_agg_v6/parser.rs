use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;

use crate::streaming::event_parser::{
    common::{read_u64_le, read_u8, EventMetadata, EventType, ProtocolType},
    core::event_parser::GenericEventParseConfig,
    protocols::jupiter_agg_v6::{
        discriminators, types::{JupiterSwapEvent, JupiterFeeEvent}, JupiterAggV6RouteEvent,
        JupiterAggV6ExactOutRouteEvent, JupiterAggV6SwapEvent, JupiterAggV6FeeEvent,
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

/// Parse SwapEvent from transaction logs
/// This is an Anchor event emitted during swap execution
pub fn parse_swap_event_from_log(
    log_data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    // Log data format: 8-byte discriminator + borsh-encoded event data
    if log_data.len() < 8 {
        return None;
    }

    // Verify discriminator
    if &log_data[0..8] != discriminators::SWAP_EVENT {
        return None;
    }

    // Deserialize the event data (skip discriminator)
    let swap_event = JupiterSwapEvent::deserialize(&mut &log_data[8..]).ok()?;

    Some(Box::new(JupiterAggV6SwapEvent {
        metadata,
        amm: swap_event.amm,
        input_mint: swap_event.input_mint,
        input_amount: swap_event.input_amount,
        output_mint: swap_event.output_mint,
        output_amount: swap_event.output_amount,
    }))
}

/// Parse FeeEvent from transaction logs
/// This is an Anchor event emitted during fee collection
pub fn parse_fee_event_from_log(
    log_data: &[u8],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    // Log data format: 8-byte discriminator + borsh-encoded event data
    if log_data.len() < 8 {
        return None;
    }

    // Verify discriminator
    if &log_data[0..8] != discriminators::FEE_EVENT {
        return None;
    }

    // Deserialize the event data (skip discriminator)
    let fee_event = JupiterFeeEvent::deserialize(&mut &log_data[8..]).ok()?;

    Some(Box::new(JupiterAggV6FeeEvent {
        metadata,
        account: fee_event.account,
        mint: fee_event.mint,
        amount: fee_event.amount,
    }))
}

/// Parse SwapEvents and FeeEvents from transaction log messages
/// Looks for "Program data: " prefix and decodes base64 anchor events
pub fn parse_events_from_logs(
    log_messages: &[String],
    signature: solana_sdk::signature::Signature,
    slot: u64,
    block_time: Option<prost_types::Timestamp>,
    recv_us: i64,
    transaction_index: Option<u64>,
) -> Vec<Box<dyn UnifiedEvent>> {
    use crate::streaming::event_parser::common::utils::extract_program_data;

    let mut events = Vec::new();

    for log in log_messages {
        if let Some(data_str) = extract_program_data(log) {
            // Decode base64 data
            if let Ok(log_data) = solana_sdk::bs58::decode(data_str).into_vec() {
                let timestamp = block_time.unwrap_or(prost_types::Timestamp { seconds: 0, nanos: 0 });
                let block_time_ms = timestamp.seconds * 1000 + (timestamp.nanos as i64) / 1_000_000;

                // Try parsing as SwapEvent
                if log_data.len() >= 8 && &log_data[0..8] == discriminators::SWAP_EVENT {
                    let metadata = EventMetadata::new(
                        signature,
                        slot,
                        timestamp.seconds,
                        block_time_ms,
                        ProtocolType::JupiterAggV6,
                        EventType::JupiterAggV6Swap,
                        JUPITER_AGG_V6_PROGRAM_ID,
                        0,
                        None,
                        recv_us,
                        transaction_index,
                    );

                    if let Some(event) = parse_swap_event_from_log(&log_data, metadata) {
                        events.push(event);
                    }
                }
                // Try parsing as FeeEvent
                else if log_data.len() >= 8 && &log_data[0..8] == discriminators::FEE_EVENT {
                    let metadata = EventMetadata::new(
                        signature,
                        slot,
                        timestamp.seconds,
                        block_time_ms,
                        ProtocolType::JupiterAggV6,
                        EventType::JupiterAggV6Fee,
                        JUPITER_AGG_V6_PROGRAM_ID,
                        0,
                        None,
                        recv_us,
                        transaction_index,
                    );

                    if let Some(event) = parse_fee_event_from_log(&log_data, metadata) {
                        events.push(event);
                    }
                }
            }
        }
    }

    events
}
