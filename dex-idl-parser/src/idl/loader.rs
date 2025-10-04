use super::types::{Idl, InstructionDiscriminators};
use anyhow::{Context, Result};
use std::collections::HashMap;

/// Load an IDL from JSON string
pub fn load_idl_from_json(json: &str) -> Result<Idl> {
    serde_json::from_str(json).context("Failed to parse IDL JSON")
}

/// Load an IDL from a file
pub fn load_idl_from_file(path: &str) -> Result<Idl> {
    let json = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read IDL file: {}", path))?;
    load_idl_from_json(&json)
}

/// Build instruction discriminator lookup from IDL
/// Returns a HashMap mapping instruction name to its discriminator bytes
pub fn build_instruction_discriminators(idl: &Idl) -> InstructionDiscriminators {
    let mut discriminators = HashMap::new();

    for instruction in &idl.instructions {
        if let Some(disc) = &instruction.discriminator {
            discriminators.insert(instruction.name.clone(), disc.clone());
        } else {
            // If no discriminator provided, compute it from the instruction name
            // This uses the Anchor discriminator algorithm: first 8 bytes of sha256("global:{name}")
            let disc = compute_anchor_discriminator(&format!("global:{}", instruction.name));
            discriminators.insert(instruction.name.clone(), disc);
        }
    }

    discriminators
}

/// Compute Anchor-style discriminator from a string
fn compute_anchor_discriminator(preimage: &str) -> Vec<u8> {
    use solana_sdk::hash::hash;
    let hash = hash(preimage.as_bytes());
    hash.to_bytes()[0..8].to_vec()
}

/// DEX protocol identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DexProtocol {
    JupiterV6,
    RaydiumClmm,
    RaydiumCpmm,
    RaydiumAmmV4,
    OrcaWhirlpool,
    MeteoraDlmm,
}

impl DexProtocol {
    pub fn name(&self) -> &'static str {
        match self {
            DexProtocol::JupiterV6 => "Jupiter Aggregator V6",
            DexProtocol::RaydiumClmm => "Raydium CLMM",
            DexProtocol::RaydiumCpmm => "Raydium CPMM",
            DexProtocol::RaydiumAmmV4 => "Raydium AMM V4",
            DexProtocol::OrcaWhirlpool => "Orca Whirlpool",
            DexProtocol::MeteoraDlmm => "Meteora DLMM",
        }
    }

    pub fn program_id(&self) -> &'static str {
        match self {
            DexProtocol::JupiterV6 => "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
            DexProtocol::RaydiumClmm => "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK",
            DexProtocol::RaydiumCpmm => "CPMMoo8L3F4NbTegBCKVNunggL7H1ZpdTHKxQB5qKP1C",
            DexProtocol::RaydiumAmmV4 => "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8",
            DexProtocol::OrcaWhirlpool => "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc",
            DexProtocol::MeteoraDlmm => "LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo",
        }
    }

    pub fn idl_path(&self) -> &'static str {
        match self {
            DexProtocol::JupiterV6 => "dex-idl-parser/idls/jupiter_agg_v6.json",
            DexProtocol::RaydiumClmm => "dex-idl-parser/idls/raydium_clmm.json",
            DexProtocol::RaydiumCpmm => "dex-idl-parser/idls/raydium_amm.json",
            DexProtocol::RaydiumAmmV4 => "dex-idl-parser/idls/raydium_amm_v4.json",
            DexProtocol::OrcaWhirlpool => "dex-idl-parser/idls/orca_whirlpool.json",
            DexProtocol::MeteoraDlmm => "dex-idl-parser/idls/meteora.json",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_jupiter_idl() {
        let idl = load_idl_from_file("dex-idl-parser/idls/jupiter_agg_v6.json");
        assert!(idl.is_ok());
        let idl = idl.unwrap();
        assert_eq!(idl.name, "jupiter");
        assert!(!idl.instructions.is_empty());
    }

    #[test]
    fn test_load_raydium_idl() {
        let idl = load_idl_from_file("dex-idl-parser/idls/raydium_clmm.json");
        assert!(idl.is_ok());
        let idl = idl.unwrap();
        assert_eq!(idl.name, "amm_v3");
        assert!(!idl.instructions.is_empty());
    }

    #[test]
    fn test_load_orca_idl() {
        let idl = load_idl_from_file("dex-idl-parser/idls/orca_whirlpool.json");
        assert!(idl.is_ok());
        let idl = idl.unwrap();
        assert_eq!(idl.name, "whirlpool");
        assert!(!idl.instructions.is_empty());
    }

    #[test]
    fn test_load_meteora_idl() {
        let idl = load_idl_from_file("dex-idl-parser/idls/meteora.json");
        assert!(idl.is_ok());
        let idl = idl.unwrap();
        assert_eq!(idl.name, "meteora");
        assert!(!idl.instructions.is_empty());
    }
}
