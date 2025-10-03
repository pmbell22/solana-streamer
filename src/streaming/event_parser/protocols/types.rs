use crate::streaming::event_parser::protocols::{
    jupiter_agg_v6::parser::JUPITER_AGG_V6_PROGRAM_ID,
    raydium_amm_v4::parser::RAYDIUM_AMM_V4_PROGRAM_ID,
    raydium_clmm::parser::RAYDIUM_CLMM_PROGRAM_ID, raydium_cpmm::parser::RAYDIUM_CPMM_PROGRAM_ID,
};
use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;

/// 支持的协议
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Protocol {
    RaydiumCpmm,
    RaydiumClmm,
    RaydiumAmmV4,
    JupiterAggV6,
}

impl Protocol {
    pub fn get_program_id(&self) -> Vec<Pubkey> {
        match self {
            Protocol::RaydiumCpmm => vec![RAYDIUM_CPMM_PROGRAM_ID],
            Protocol::RaydiumClmm => vec![RAYDIUM_CLMM_PROGRAM_ID],
            Protocol::RaydiumAmmV4 => vec![RAYDIUM_AMM_V4_PROGRAM_ID],
            Protocol::JupiterAggV6 => vec![JUPITER_AGG_V6_PROGRAM_ID],
        }
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::RaydiumCpmm => write!(f, "RaydiumCpmm"),
            Protocol::RaydiumClmm => write!(f, "RaydiumClmm"),
            Protocol::RaydiumAmmV4 => write!(f, "RaydiumAmmV4"),
            Protocol::JupiterAggV6 => write!(f, "JupiterAggV6"),
        }
    }
}

impl std::str::FromStr for Protocol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "raydiumcpmm" => Ok(Protocol::RaydiumCpmm),
            "raydiumclmm" => Ok(Protocol::RaydiumClmm),
            "raydiumammv4" => Ok(Protocol::RaydiumAmmV4),
            "jupiteraggv6" => Ok(Protocol::JupiterAggV6),
            _ => Err(anyhow!("Unsupported protocol: {}", s)),
        }
    }
}
