use crate::idl::DexProtocol;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// Parsed instruction from a transaction
#[derive(Debug, Clone)]
pub struct ParsedInstruction {
    /// Program name from IDL
    pub program: String,
    /// Instruction name
    pub instruction: String,
    /// Named accounts
    pub accounts: HashMap<String, Pubkey>,
    /// Parsed instruction data
    pub data: ParsedInstructionData,
    /// Raw discriminator bytes
    pub raw_discriminator: Vec<u8>,
}

/// Parsed instruction data
#[derive(Debug, Clone)]
pub struct ParsedInstructionData {
    /// Field names from IDL
    pub fields: Vec<String>,
    /// Raw instruction data (after discriminator)
    pub raw_data: Vec<u8>,
}

/// Unified DEX event that can be streamed via Yellowstone gRPC
#[derive(Debug, Clone)]
pub struct DexEvent {
    /// Protocol (Jupiter, Raydium, Orca)
    pub protocol: DexProtocol,
    /// Transaction signature
    pub signature: String,
    /// Slot number
    pub slot: u64,
    /// Block time (Unix timestamp)
    pub block_time: i64,
    /// Parsed instruction
    pub instruction: ParsedInstruction,
    /// Transaction index (if available)
    pub transaction_index: Option<u64>,
}

impl DexEvent {
    pub fn new(
        protocol: DexProtocol,
        signature: String,
        slot: u64,
        block_time: i64,
        instruction: ParsedInstruction,
        transaction_index: Option<u64>,
    ) -> Self {
        Self {
            protocol,
            signature,
            slot,
            block_time,
            instruction,
            transaction_index,
        }
    }

    /// Get the instruction name
    pub fn instruction_name(&self) -> &str {
        &self.instruction.instruction
    }

    /// Get the program name
    pub fn program_name(&self) -> &str {
        &self.instruction.program
    }

    /// Get an account by name
    pub fn get_account(&self, name: &str) -> Option<&Pubkey> {
        self.instruction.accounts.get(name)
    }

    /// Check if this is a swap instruction
    pub fn is_swap(&self) -> bool {
        let name = self.instruction_name().to_lowercase();
        name.contains("swap") || name.contains("route")
    }

    /// Check if this is a liquidity provision instruction
    pub fn is_liquidity_provision(&self) -> bool {
        let name = self.instruction_name().to_lowercase();
        name.contains("deposit") || name.contains("add_liquidity") || name.contains("increase")
    }

    /// Check if this is a liquidity removal instruction
    pub fn is_liquidity_removal(&self) -> bool {
        let name = self.instruction_name().to_lowercase();
        name.contains("withdraw") || name.contains("remove_liquidity") || name.contains("decrease")
    }
}

impl std::fmt::Display for DexEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} - {} @ slot {} (sig: {}...)",
            self.protocol.name(),
            self.program_name(),
            self.instruction_name(),
            self.slot,
            &self.signature[0..8]
        )
    }
}
