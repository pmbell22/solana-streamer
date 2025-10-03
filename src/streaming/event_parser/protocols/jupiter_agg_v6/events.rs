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

/// Event discriminators
pub mod discriminators {
    // Instruction discriminators
    pub const ROUTE: &[u8] = &[229, 23, 203, 151, 122, 227, 173, 42];
    pub const EXACT_OUT_ROUTE: &[u8] = &[208, 51, 239, 151, 123, 43, 237, 92];
}
