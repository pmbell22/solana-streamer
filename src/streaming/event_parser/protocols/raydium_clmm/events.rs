use crate::streaming::event_parser::common::EventMetadata;
use crate::streaming::event_parser::protocols::raydium_clmm::types::{PoolState, TickArrayState};
use crate::{
    impl_unified_event, streaming::event_parser::protocols::raydium_clmm::types::AmmConfig,
};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// 交易
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaydiumClmmSwapEvent {
    pub metadata: EventMetadata,
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit_x64: u128,
    pub is_base_input: bool,
    pub payer: Pubkey,
    pub amm_config: Pubkey,
    pub pool_state: Pubkey,
    pub input_token_account: Pubkey,
    pub output_token_account: Pubkey,
    pub input_vault: Pubkey,
    pub output_vault: Pubkey,
    pub observation_state: Pubkey,
    pub token_program: Pubkey,
    pub tick_array: Pubkey,
    pub remaining_accounts: Vec<Pubkey>,
}

impl_unified_event!(RaydiumClmmSwapEvent,);

/// 交易v2
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaydiumClmmSwapV2Event {
    pub metadata: EventMetadata,
    pub amount: u64,
    pub other_amount_threshold: u64,
    pub sqrt_price_limit_x64: u128,
    pub is_base_input: bool,
    pub payer: Pubkey,
    pub amm_config: Pubkey,
    pub pool_state: Pubkey,
    pub input_token_account: Pubkey,
    pub output_token_account: Pubkey,
    pub input_vault: Pubkey,
    pub output_vault: Pubkey,
    pub observation_state: Pubkey,
    pub token_program: Pubkey,
    pub token_program2022: Pubkey,
    pub memo_program: Pubkey,
    pub input_vault_mint: Pubkey,
    pub output_vault_mint: Pubkey,
    pub remaining_accounts: Vec<Pubkey>,
}
impl_unified_event!(RaydiumClmmSwapV2Event,);

/// 关闭仓位
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaydiumClmmClosePositionEvent {
    pub metadata: EventMetadata,
    pub nft_owner: Pubkey,
    pub position_nft_mint: Pubkey,
    pub position_nft_account: Pubkey,
    pub personal_position: Pubkey,
    pub system_program: Pubkey,
    pub token_program: Pubkey,
}
impl_unified_event!(RaydiumClmmClosePositionEvent,);

/// 减少流动性v2
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaydiumClmmDecreaseLiquidityV2Event {
    pub metadata: EventMetadata,
    pub liquidity: u128,
    pub amount0_min: u64,
    pub amount1_min: u64,
    pub nft_owner: Pubkey,
    pub nft_account: Pubkey,
    pub personal_position: Pubkey,
    pub pool_state: Pubkey,
    pub protocol_position: Pubkey,
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
    pub tick_array_lower: Pubkey,
    pub tick_array_upper: Pubkey,
    pub recipient_token_account0: Pubkey,
    pub recipient_token_account1: Pubkey,
    pub token_program: Pubkey,
    pub token_program2022: Pubkey,
    pub memo_program: Pubkey,
    pub vault0_mint: Pubkey,
    pub vault1_mint: Pubkey,
    pub remaining_accounts: Vec<Pubkey>,
}
impl_unified_event!(RaydiumClmmDecreaseLiquidityV2Event,);

/// 创建池
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaydiumClmmCreatePoolEvent {
    pub metadata: EventMetadata,
    pub sqrt_price_x64: u128,
    pub open_time: u64,
    pub pool_creator: Pubkey,
    pub amm_config: Pubkey,
    pub pool_state: Pubkey,
    pub token_mint0: Pubkey,
    pub token_mint1: Pubkey,
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
    pub observation_state: Pubkey,
    pub tick_array_bitmap: Pubkey,
    pub token_program0: Pubkey,
    pub token_program1: Pubkey,
    pub system_program: Pubkey,
    pub rent: Pubkey,
}
impl_unified_event!(RaydiumClmmCreatePoolEvent,);

/// 增加流动性v2
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaydiumClmmIncreaseLiquidityV2Event {
    pub metadata: EventMetadata,
    pub liquidity: u128,
    pub amount0_max: u64,
    pub amount1_max: u64,
    pub base_flag: Option<bool>,
    pub nft_owner: Pubkey,
    pub nft_account: Pubkey,
    pub pool_state: Pubkey,
    pub protocol_position: Pubkey,
    pub personal_position: Pubkey,
    pub tick_array_lower: Pubkey,
    pub tick_array_upper: Pubkey,
    pub token_account0: Pubkey,
    pub token_account1: Pubkey,
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
    pub token_program: Pubkey,
    pub token_program2022: Pubkey,
    pub vault0_mint: Pubkey,
    pub vault1_mint: Pubkey,
}
impl_unified_event!(RaydiumClmmIncreaseLiquidityV2Event,);

/// 打开仓位v2
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaydiumClmmOpenPositionWithToken22NftEvent {
    pub metadata: EventMetadata,
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub tick_array_lower_start_index: i32,
    pub tick_array_upper_start_index: i32,
    pub liquidity: u128,
    pub amount0_max: u64,
    pub amount1_max: u64,
    pub with_metadata: bool,
    pub base_flag: Option<bool>,

    pub payer: Pubkey,
    pub position_nft_owner: Pubkey,
    pub position_nft_mint: Pubkey,
    pub position_nft_account: Pubkey,
    pub pool_state: Pubkey,
    pub protocol_position: Pubkey,
    pub tick_array_lower: Pubkey,
    pub tick_array_upper: Pubkey,
    pub personal_position: Pubkey,
    pub token_account0: Pubkey,
    pub token_account1: Pubkey,
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
    pub rent: Pubkey,
    pub system_program: Pubkey,
    pub token_program: Pubkey,
    pub associated_token_program: Pubkey,
    pub token_program2022: Pubkey,
    pub vault0_mint: Pubkey,
    pub vault1_mint: Pubkey,
}
impl_unified_event!(RaydiumClmmOpenPositionWithToken22NftEvent,);

/// 打开仓位V2
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaydiumClmmOpenPositionV2Event {
    pub metadata: EventMetadata,
    pub tick_lower_index: i32,
    pub tick_upper_index: i32,
    pub tick_array_lower_start_index: i32,
    pub tick_array_upper_start_index: i32,
    pub liquidity: u128,
    pub amount0_max: u64,
    pub amount1_max: u64,
    pub with_metadata: bool,
    pub base_flag: Option<bool>,

    pub payer: Pubkey,
    pub position_nft_owner: Pubkey,
    pub position_nft_mint: Pubkey,
    pub position_nft_account: Pubkey,
    pub metadata_account: Pubkey,
    pub pool_state: Pubkey,
    pub protocol_position: Pubkey,
    pub tick_array_lower: Pubkey,
    pub tick_array_upper: Pubkey,
    pub personal_position: Pubkey,
    pub token_account0: Pubkey,
    pub token_account1: Pubkey,
    pub token_vault0: Pubkey,
    pub token_vault1: Pubkey,
    pub rent: Pubkey,
    pub system_program: Pubkey,
    pub token_program: Pubkey,
    pub associated_token_program: Pubkey,
    pub metadata_program: Pubkey,
    pub token_program2022: Pubkey,
    pub vault0_mint: Pubkey,
    pub vault1_mint: Pubkey,
    pub remaining_accounts: Vec<Pubkey>,
}
impl_unified_event!(RaydiumClmmOpenPositionV2Event,);

/// 池配置
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaydiumClmmAmmConfigAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub amm_config: AmmConfig,
}
impl_unified_event!(RaydiumClmmAmmConfigAccountEvent,);

/// 池状态
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaydiumClmmPoolStateAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub pool_state: PoolState,
}
impl_unified_event!(RaydiumClmmPoolStateAccountEvent,);

/// 池状态
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RaydiumClmmTickArrayStateAccountEvent {
    pub metadata: EventMetadata,
    pub pubkey: Pubkey,
    pub executable: bool,
    pub lamports: u64,
    pub owner: Pubkey,
    pub rent_epoch: u64,
    pub tick_array_state: TickArrayState,
}
impl_unified_event!(RaydiumClmmTickArrayStateAccountEvent,);

/// 事件鉴别器常量
pub mod discriminators {
    // 指令鉴别器 (calculated from SHA256("global:instruction_name"))
    // FIXED: Corrected discriminators to match Anchor calculation
    pub const SWAP: &[u8] = &[248, 198, 158, 145, 225, 117, 135, 200];             // swap (correct)
    pub const SWAP_V2: &[u8] = &[114, 113, 45, 226, 179, 239, 106, 225];           // swapV2 (FIXED)
    pub const CLOSE_POSITION: &[u8] = &[123, 134, 81, 0, 49, 68, 98, 98];          // closePosition (verify)
    pub const INCREASE_LIQUIDITY_V2: &[u8] = &[67, 78, 196, 105, 211, 25, 62, 252]; // increaseLiquidityV2 (FIXED)
    pub const DECREASE_LIQUIDITY_V2: &[u8] = &[82, 1, 46, 234, 207, 210, 241, 169]; // decreaseLiquidityV2 (FIXED)
    pub const CREATE_POOL: &[u8] = &[244, 236, 117, 4, 18, 0, 62, 88];             // createPool (FIXED)
    pub const OPEN_POSITION_WITH_TOKEN_22_NFT: &[u8] = &[77, 255, 174, 82, 125, 29, 201, 46]; // verify
    pub const OPEN_POSITION_V2: &[u8] = &[218, 45, 162, 175, 86, 17, 83, 121];     // openPositionV2 (FIXED)

    // 账号鉴别器 (calculated from SHA256("account:account_name"))
    pub const AMM_CONFIG: &[u8] = &[218, 244, 33, 104, 203, 203, 43, 111];    // AmmConfig (correct)
    pub const POOL_STATE: &[u8] = &[247, 237, 227, 245, 215, 195, 222, 70];   // PoolState (correct)
    pub const TICK_ARRAY_STATE: &[u8] = &[192, 155, 85, 205, 49, 249, 129, 42]; // TickArrayState (verify)
}
