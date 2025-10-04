use borsh::{BorshDeserialize, io};

/// Jupiter Swap enum - represents different DEX types
#[derive(Debug, Clone)]
pub enum Swap {
    Saber,
    SaberAddDecimalsDeposit,
    SaberAddDecimalsWithdraw,
    TokenSwap,
    Sencha,
    Step,
    Cropper,
    Raydium,
    Crema { a_to_b: bool },
    Lifinity,
    Mercurial,
    Cykura,
    Serum { side: Side },
    MarinadeDeposit,
    MarinadeUnstake,
    Aldrin { side: Side },
    AldrinV2 { side: Side },
    Whirlpool { a_to_b: bool },
    Invariant { x_to_y: bool },
    Meteora,
    GooseFX,
    DeltaFi { stable: bool },
    Balansol,
    MarcoPolo { x_to_y: bool },
    Dradex { side: Side },
    LifinityV2,
    RaydiumClmm,
    Openbook { side: Side },
    Phoenix { side: Side },
    Symmetry { from_token_id: u64, to_token_id: u64 },
    TokenSwapV2,
    HeliumTreasuryManagementRedeemV0,
    StakeDexStakeWrappedSol,
    StakeDexSwapViaStake { bridge_stake_seed: u32 },
    GooseFXV2,
    Perps,
    PerpsAddLiquidity,
    PerpsRemoveLiquidity,
    MeteoraDlmm,
    /// Unknown swap type (for future/undocumented variants)
    Unknown(u8),
}

impl BorshDeserialize for Swap {
    fn deserialize_reader<R: io::Read>(reader: &mut R) -> io::Result<Self> {
        let variant = u8::deserialize_reader(reader)?;

        match variant {
            0 => Ok(Swap::Saber),
            1 => Ok(Swap::SaberAddDecimalsDeposit),
            2 => Ok(Swap::SaberAddDecimalsWithdraw),
            3 => Ok(Swap::TokenSwap),
            4 => Ok(Swap::Sencha),
            5 => Ok(Swap::Step),
            6 => Ok(Swap::Cropper),
            7 => Ok(Swap::Raydium),
            8 => Ok(Swap::Crema { a_to_b: bool::deserialize_reader(reader)? }),
            9 => Ok(Swap::Lifinity),
            10 => Ok(Swap::Mercurial),
            11 => Ok(Swap::Cykura),
            12 => Ok(Swap::Serum { side: Side::deserialize_reader(reader)? }),
            13 => Ok(Swap::MarinadeDeposit),
            14 => Ok(Swap::MarinadeUnstake),
            15 => Ok(Swap::Aldrin { side: Side::deserialize_reader(reader)? }),
            16 => Ok(Swap::AldrinV2 { side: Side::deserialize_reader(reader)? }),
            17 => Ok(Swap::Whirlpool { a_to_b: bool::deserialize_reader(reader)? }),
            18 => Ok(Swap::Invariant { x_to_y: bool::deserialize_reader(reader)? }),
            19 => Ok(Swap::Meteora),
            20 => Ok(Swap::GooseFX),
            21 => Ok(Swap::DeltaFi { stable: bool::deserialize_reader(reader)? }),
            22 => Ok(Swap::Balansol),
            23 => Ok(Swap::MarcoPolo { x_to_y: bool::deserialize_reader(reader)? }),
            24 => Ok(Swap::Dradex { side: Side::deserialize_reader(reader)? }),
            25 => Ok(Swap::LifinityV2),
            26 => Ok(Swap::RaydiumClmm),
            27 => Ok(Swap::Openbook { side: Side::deserialize_reader(reader)? }),
            28 => Ok(Swap::Phoenix { side: Side::deserialize_reader(reader)? }),
            29 => Ok(Swap::Symmetry {
                from_token_id: u64::deserialize_reader(reader)?,
                to_token_id: u64::deserialize_reader(reader)?,
            }),
            30 => Ok(Swap::TokenSwapV2),
            31 => Ok(Swap::HeliumTreasuryManagementRedeemV0),
            32 => Ok(Swap::StakeDexStakeWrappedSol),
            33 => Ok(Swap::StakeDexSwapViaStake {
                bridge_stake_seed: u32::deserialize_reader(reader)?,
            }),
            34 => Ok(Swap::GooseFXV2),
            35 => Ok(Swap::Perps),
            36 => Ok(Swap::PerpsAddLiquidity),
            37 => Ok(Swap::PerpsRemoveLiquidity),
            38 => Ok(Swap::MeteoraDlmm),
            _ => Ok(Swap::Unknown(variant)),
        }
    }
}

#[derive(Debug, Clone, BorshDeserialize)]
pub enum Side {
    Bid,
    Ask,
}

/// Jupiter RoutePlanStep - one hop in a multi-hop swap
#[derive(Debug, Clone, BorshDeserialize)]
pub struct RoutePlanStep {
    pub swap: Swap,
    pub percent: u8,
    pub input_index: u8,
    pub output_index: u8,
}

impl Swap {
    /// Get the human-readable DEX name
    pub fn dex_name(&self) -> String {
        match self {
            Swap::Saber | Swap::SaberAddDecimalsDeposit | Swap::SaberAddDecimalsWithdraw => "Saber".to_string(),
            Swap::TokenSwap | Swap::TokenSwapV2 => "Token Swap".to_string(),
            Swap::Sencha => "Sencha".to_string(),
            Swap::Step => "Step Finance".to_string(),
            Swap::Cropper => "Cropper".to_string(),
            Swap::Raydium => "Raydium".to_string(),
            Swap::RaydiumClmm => "Raydium CLMM".to_string(),
            Swap::Crema { .. } => "Crema".to_string(),
            Swap::Lifinity | Swap::LifinityV2 => "Lifinity".to_string(),
            Swap::Mercurial => "Mercurial".to_string(),
            Swap::Cykura => "Cykura".to_string(),
            Swap::Serum { .. } => "Serum".to_string(),
            Swap::MarinadeDeposit | Swap::MarinadeUnstake => "Marinade".to_string(),
            Swap::Aldrin { .. } | Swap::AldrinV2 { .. } => "Aldrin".to_string(),
            Swap::Whirlpool { .. } => "Orca Whirlpool".to_string(),
            Swap::Invariant { .. } => "Invariant".to_string(),
            Swap::Meteora => "Meteora".to_string(),
            Swap::MeteoraDlmm => "Meteora DLMM".to_string(),
            Swap::GooseFX | Swap::GooseFXV2 => "GooseFX".to_string(),
            Swap::DeltaFi { .. } => "DeltaFi".to_string(),
            Swap::Balansol => "Balansol".to_string(),
            Swap::MarcoPolo { .. } => "Marco Polo".to_string(),
            Swap::Dradex { .. } => "Dradex".to_string(),
            Swap::Openbook { .. } => "OpenBook".to_string(),
            Swap::Phoenix { .. } => "Phoenix".to_string(),
            Swap::Symmetry { .. } => "Symmetry".to_string(),
            Swap::HeliumTreasuryManagementRedeemV0 => "Helium".to_string(),
            Swap::StakeDexStakeWrappedSol | Swap::StakeDexSwapViaStake { .. } => "StakeDex".to_string(),
            Swap::Perps | Swap::PerpsAddLiquidity | Swap::PerpsRemoveLiquidity => "Perps".to_string(),
            Swap::Unknown(variant) => format!("Unknown({})", variant),
        }
    }
}

impl std::fmt::Display for RoutePlanStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.percent == 100 {
            write!(f, "{}", self.swap.dex_name())
        } else {
            write!(f, "{} ({}%)", self.swap.dex_name(), self.percent)
        }
    }
}

/// Format a vector of RoutePlanSteps as a readable route
pub fn format_route(steps: &[RoutePlanStep]) -> String {
    if steps.is_empty() {
        return "Direct".to_string();
    }

    steps
        .iter()
        .map(|step| step.to_string())
        .collect::<Vec<_>>()
        .join(" → ")
}
