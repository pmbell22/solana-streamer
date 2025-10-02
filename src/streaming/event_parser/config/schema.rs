use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// IDL-like configuration for a protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolConfig {
    /// Protocol name (e.g., "raydium_amm_v4", "orca_whirlpool")
    pub name: String,

    /// Protocol version
    pub version: String,

    /// Program ID
    #[serde(with = "pubkey_string")]
    pub program_id: Pubkey,

    /// Description
    pub description: Option<String>,

    /// All instruction definitions for this protocol
    pub instructions: Vec<InstructionConfig>,

    /// Custom type definitions (for complex nested structures)
    #[serde(default)]
    pub types: HashMap<String, Vec<AccountField>>,
}

/// Configuration for a single instruction type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionConfig {
    /// Instruction name (e.g., "swap_base_in", "deposit")
    pub name: String,

    /// Instruction discriminator (hex string)
    pub discriminator: String,

    /// Event type identifier
    pub event_type: String,

    /// Account layout - ordered list of accounts this instruction expects
    pub accounts: Vec<AccountField>,

    /// Instruction data fields (after discriminator)
    #[serde(default)]
    pub data_fields: Vec<DataField>,

    /// Whether this instruction requires inner instructions
    #[serde(default)]
    pub requires_inner_instruction: bool,

    /// Inner instruction discriminator if needed
    #[serde(default)]
    pub inner_discriminator: Option<String>,
}

/// Account field definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountField {
    /// Field name
    pub name: String,

    /// Whether this account is mutable
    #[serde(default)]
    pub is_mut: bool,

    /// Whether this account is a signer
    #[serde(default)]
    pub is_signer: bool,

    /// Optional description
    pub description: Option<String>,
}

/// Data field definition for instruction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataField {
    /// Field name
    pub name: String,

    /// Field type
    pub field_type: FieldType,

    /// Byte offset in instruction data
    pub offset: usize,

    /// Optional description
    pub description: Option<String>,
}

/// Field type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    Bool,
    Pubkey,
    String,
    /// Custom type reference
    Custom(String),
}

/// Event configuration for runtime event creation
#[derive(Debug, Clone)]
pub struct EventConfig {
    pub event_type: String,
    pub account_map: HashMap<String, usize>,
    pub data_map: HashMap<String, (usize, FieldType)>,
}

/// Serde helper for Pubkey serialization
mod pubkey_string {
    use serde::{Deserialize, Deserializer, Serializer};
    use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    pub fn serialize<S>(pubkey: &Pubkey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&pubkey.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Pubkey::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl ProtocolConfig {
    /// Validate the configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.name.is_empty() {
            anyhow::bail!("Protocol name cannot be empty");
        }

        if self.instructions.is_empty() {
            anyhow::bail!("Protocol must have at least one instruction");
        }

        for instruction in &self.instructions {
            instruction.validate()?;
        }

        Ok(())
    }
}

impl InstructionConfig {
    /// Validate the instruction configuration
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.name.is_empty() {
            anyhow::bail!("Instruction name cannot be empty");
        }

        if self.discriminator.is_empty() {
            anyhow::bail!("Instruction discriminator cannot be empty");
        }

        // Validate discriminator is valid hex
        hex::decode(&self.discriminator)
            .map_err(|e| anyhow::anyhow!("Invalid discriminator hex: {}", e))?;

        Ok(())
    }

    /// Get discriminator as bytes
    pub fn discriminator_bytes(&self) -> anyhow::Result<Vec<u8>> {
        hex::decode(&self.discriminator)
            .map_err(|e| anyhow::anyhow!("Failed to decode discriminator: {}", e))
    }

    /// Get inner discriminator as bytes
    pub fn inner_discriminator_bytes(&self) -> anyhow::Result<Option<Vec<u8>>> {
        if let Some(ref inner_disc) = self.inner_discriminator {
            Ok(Some(hex::decode(inner_disc)?))
        } else {
            Ok(None)
        }
    }
}
