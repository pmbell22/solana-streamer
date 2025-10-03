use crate::streaming::event_parser::common::EventMetadata;
use crate::impl_unified_event;
use borsh::BorshDeserialize;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Jupiter Aggregator V6 Route (Swap) Event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct JupiterAggV6RouteEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,

    // Route instruction parameters
    pub in_amount: u64,
    pub quoted_out_amount: u64,
    pub slippage_bps: u64,
    pub platform_fee_bps: u8,

    // Account information
    pub token_program: Pubkey,
    pub user_transfer_authority: Pubkey,
    pub user_source_token_account: Pubkey,
    pub user_destination_token_account: Pubkey,
    pub destination_token_account: Pubkey,
    pub source_mint: Pubkey,
    pub destination_mint: Pubkey,
    pub platform_fee_account: Pubkey,
    pub event_authority: Pubkey,
    pub program: Pubkey,
}

impl_unified_event!(JupiterAggV6RouteEvent,);

/// Jupiter Aggregator V6 Exact Out Route Event
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct JupiterAggV6ExactOutRouteEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,

    // Exact out route instruction parameters
    pub out_amount: u64,
    pub quoted_in_amount: u64,
    pub slippage_bps: u64,
    pub platform_fee_bps: u8,

    // Account information
    pub token_program: Pubkey,
    pub user_transfer_authority: Pubkey,
    pub user_source_token_account: Pubkey,
    pub user_destination_token_account: Pubkey,
    pub destination_token_account: Pubkey,
    pub source_mint: Pubkey,
    pub destination_mint: Pubkey,
    pub platform_fee_account: Pubkey,
    pub event_authority: Pubkey,
    pub program: Pubkey,
}

impl_unified_event!(JupiterAggV6ExactOutRouteEvent,);

/// Jupiter Aggregator V6 Swap Event (emitted during actual swap execution)
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct JupiterAggV6SwapEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,

    // Swap event data
    pub amm: Pubkey,
    pub input_mint: Pubkey,
    pub input_amount: u64,
    pub output_mint: Pubkey,
    pub output_amount: u64,
}

impl_unified_event!(JupiterAggV6SwapEvent,);

/// Jupiter Aggregator V6 Fee Event (emitted during swap execution)
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, BorshDeserialize)]
pub struct JupiterAggV6FeeEvent {
    #[borsh(skip)]
    pub metadata: EventMetadata,

    // Fee event data
    pub account: Pubkey,
    pub mint: Pubkey,
    pub amount: u64,
}

impl_unified_event!(JupiterAggV6FeeEvent,);

/// Event discriminators
pub mod discriminators {
    // Instruction discriminators (from IDL)
    pub const ROUTE: &[u8] = &[229, 23, 203, 151, 122, 227, 173, 42];
    pub const EXACT_OUT_ROUTE: &[u8] = &[208, 51, 239, 151, 123, 43, 237, 92];

    // Event discriminators (Anchor event: first 8 bytes of sha256("event:<EventName>"))
    // Updated to match IDL discriminators
    pub const SWAP_EVENT: &[u8] = &[64, 198, 205, 232, 38, 8, 113, 226];
    pub const FEE_EVENT: &[u8] = &[73, 79, 78, 127, 184, 213, 13, 220];
}
