use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Generic trait for all pool states
pub trait PoolState {
    /// Get the current price (not sqrt price)
    fn get_price(&self) -> f64;

    /// Get the liquidity
    fn get_liquidity(&self) -> u128;

    /// Get token mint A
    fn get_token_mint_a(&self) -> Pubkey;

    /// Get token mint B
    fn get_token_mint_b(&self) -> Pubkey;
}

/// Raydium CLMM Pool State
/// Based on: https://github.com/raydium-io/raydium-clmm/blob/master/programs/amm/src/states/pool.rs
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Serialize, Deserialize)]
pub struct RaydiumClmmPoolState {
    /// Bump to identify PDA
    pub bump: [u8; 1],

    /// Which config the pool belongs
    pub amm_config: Pubkey,

    /// Pool creator
    pub owner: Pubkey,

    /// Token pair of the pool, where token_mint_0 address < token_mint_1 address
    pub token_mint_0: Pubkey,
    pub token_mint_1: Pubkey,

    /// Token pair vault
    pub token_vault_0: Pubkey,
    pub token_vault_1: Pubkey,

    /// observation account key
    pub observation_key: Pubkey,

    /// mint0 and mint1 decimals
    pub mint_decimals_0: u8,
    pub mint_decimals_1: u8,

    /// The minimum number of ticks between initialized ticks
    pub tick_spacing: u16,

    /// The currently in range liquidity available to the pool
    pub liquidity: u128,

    /// The current price of the pool as a sqrt(token_1/token_0) Q64.64 value
    pub sqrt_price_x64: u128,

    /// The current tick of the pool
    pub tick_current: i32,

    pub padding3: u16,
    pub padding4: u16,

    /// Fee growth as a Q64.64 number, collected per unit of liquidity
    pub fee_growth_global_0_x64: u128,
    pub fee_growth_global_1_x64: u128,

    /// Amounts owed to the protocol
    pub protocol_fees_token_0: u64,
    pub protocol_fees_token_1: u64,

    /// Amounts in and out of swap token_0 and token_1
    pub swap_in_amount_token_0: u128,
    pub swap_out_amount_token_1: u128,
    pub swap_in_amount_token_1: u128,
    pub swap_out_amount_token_0: u128,

    /// Bitwise representation of the state of the pool
    pub status: u8,

    pub padding: [u8; 7],

    pub recent_epoch: u64,
}

impl PoolState for RaydiumClmmPoolState {
    fn get_price(&self) -> f64 {
        // Convert from Q64.64 sqrt price to actual price
        let sqrt_price = self.sqrt_price_x64 as f64 / 2f64.powi(64);
        sqrt_price * sqrt_price
    }

    fn get_liquidity(&self) -> u128 {
        self.liquidity
    }

    fn get_token_mint_a(&self) -> Pubkey {
        self.token_mint_0
    }

    fn get_token_mint_b(&self) -> Pubkey {
        self.token_mint_1
    }
}

/// Orca Whirlpool State
/// Based on: https://github.com/orca-so/whirlpools/blob/main/programs/whirlpool/src/state/whirlpool.rs
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Serialize, Deserialize)]
pub struct OrcaWhirlpoolState {
    pub whirlpools_config: Pubkey,
    pub whirlpool_bump: [u8; 1],
    pub tick_spacing: u16,
    pub tick_spacing_seed: [u8; 2],

    /// Stored as hundredths of a basis point
    pub fee_rate: u16,

    /// Portion of fee rate taken stored as basis points
    pub protocol_fee_rate: u16,

    /// Maximum amount that can be held by Solana account
    pub liquidity: u128,

    /// Q64.64
    pub sqrt_price: u128,
    pub tick_current_index: i32,

    pub protocol_fee_owed_a: u64,
    pub protocol_fee_owed_b: u64,

    pub token_mint_a: Pubkey,
    pub token_vault_a: Pubkey,

    /// Q64.64
    pub fee_growth_global_a: u128,

    pub token_mint_b: Pubkey,
    pub token_vault_b: Pubkey,

    /// Q64.64
    pub fee_growth_global_b: u128,

    pub reward_last_updated_timestamp: u64,

    // Note: reward_infos array not included for simplicity
    // pub reward_infos: [WhirlpoolRewardInfo; 3],
}

impl PoolState for OrcaWhirlpoolState {
    fn get_price(&self) -> f64 {
        // Convert from Q64.64 sqrt price to actual price
        let sqrt_price = self.sqrt_price as f64 / 2f64.powi(64);
        sqrt_price * sqrt_price
    }

    fn get_liquidity(&self) -> u128 {
        self.liquidity
    }

    fn get_token_mint_a(&self) -> Pubkey {
        self.token_mint_a
    }

    fn get_token_mint_b(&self) -> Pubkey {
        self.token_mint_b
    }
}

/// Meteora DLMM Pool State (simplified)
#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Serialize, Deserialize)]
pub struct MeteoraDlmmPoolState {
    pub parameters: Pubkey,
    pub reserve_x: Pubkey,
    pub reserve_y: Pubkey,
    pub mint_x: Pubkey,
    pub mint_y: Pubkey,
    pub bin_step: u16,
    pub base_factor: u16,
    pub activation_point: i32,
    pub status: u8,
    pub active_id: i32,
    pub bin_step_num: [u8; 2],
    pub padding1: u8,
    pub liquidity: u128,
    // Additional fields would be added based on actual Meteora IDL
}

impl PoolState for MeteoraDlmmPoolState {
    fn get_price(&self) -> f64 {
        // Meteora uses bins/ticks differently, simplified price calculation
        // Price = (1 + bin_step/10000)^active_id
        let bin_step_decimal = self.bin_step as f64 / 10000.0;
        (1.0 + bin_step_decimal).powi(self.active_id)
    }

    fn get_liquidity(&self) -> u128 {
        self.liquidity
    }

    fn get_token_mint_a(&self) -> Pubkey {
        self.mint_x
    }

    fn get_token_mint_b(&self) -> Pubkey {
        self.mint_y
    }
}

/// Enum to hold different pool state types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DexPoolState {
    RaydiumClmm(RaydiumClmmPoolState),
    OrcaWhirlpool(OrcaWhirlpoolState),
    MeteoraDlmm(MeteoraDlmmPoolState),
}

impl DexPoolState {
    pub fn get_price(&self) -> f64 {
        match self {
            DexPoolState::RaydiumClmm(pool) => pool.get_price(),
            DexPoolState::OrcaWhirlpool(pool) => pool.get_price(),
            DexPoolState::MeteoraDlmm(pool) => pool.get_price(),
        }
    }

    pub fn get_liquidity(&self) -> u128 {
        match self {
            DexPoolState::RaydiumClmm(pool) => pool.get_liquidity(),
            DexPoolState::OrcaWhirlpool(pool) => pool.get_liquidity(),
            DexPoolState::MeteoraDlmm(pool) => pool.get_liquidity(),
        }
    }

    pub fn get_token_pair(&self) -> (Pubkey, Pubkey) {
        match self {
            DexPoolState::RaydiumClmm(pool) => (pool.get_token_mint_a(), pool.get_token_mint_b()),
            DexPoolState::OrcaWhirlpool(pool) => (pool.get_token_mint_a(), pool.get_token_mint_b()),
            DexPoolState::MeteoraDlmm(pool) => (pool.get_token_mint_a(), pool.get_token_mint_b()),
        }
    }
}

/// DEX Protocol enum for identifying which DEX a pool belongs to
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DexProtocol {
    RaydiumClmm,
    OrcaWhirlpool,
    MeteoraDlmm,
    CremaFinance,
    DefiTuna,
}

impl DexProtocol {
    /// Get the program ID for this DEX
    pub fn program_id(&self) -> &'static str {
        match self {
            DexProtocol::RaydiumClmm => "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK",
            DexProtocol::OrcaWhirlpool => "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc",
            DexProtocol::MeteoraDlmm => "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo",
            DexProtocol::CremaFinance => "6MLxLqiXaaSUpkgMnWDTuejNZEz3kE7k2woyHGVFw319",
            DexProtocol::DefiTuna => "tunaxCwbGJ84Ra6YkbV4pLXd8PzJHfKgRLsWELk19C5",
        }
    }

    /// Get the name of the DEX
    pub fn name(&self) -> &'static str {
        match self {
            DexProtocol::RaydiumClmm => "Raydium CLMM",
            DexProtocol::OrcaWhirlpool => "Orca Whirlpool",
            DexProtocol::MeteoraDlmm => "Meteora DLMM",
            DexProtocol::CremaFinance => "Crema Finance",
            DexProtocol::DefiTuna => "DefiTuna",
        }
    }
}
