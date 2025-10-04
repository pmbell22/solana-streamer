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
    /// Field names and types from IDL
    pub fields: Vec<FieldInfo>,
    /// Raw instruction data (after discriminator)
    pub raw_data: Vec<u8>,
}

/// Information about a field
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub type_name: String,
    pub value: Option<ParsedValue>,
}

/// Represents a parsed value from instruction data
#[derive(Debug, Clone)]
pub enum ParsedValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    U128(u128),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    Bool(bool),
    String(String),
    Pubkey(Pubkey),
    Vec(Vec<ParsedValue>),
    Bytes(Vec<u8>),
    Struct(HashMap<String, ParsedValue>),
    Unknown(Vec<u8>),
}

impl std::fmt::Display for ParsedValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsedValue::U8(v) => write!(f, "{}", v),
            ParsedValue::U16(v) => write!(f, "{}", v),
            ParsedValue::U32(v) => write!(f, "{}", v),
            ParsedValue::U64(v) => write!(f, "{}", v),
            ParsedValue::U128(v) => write!(f, "{}", v),
            ParsedValue::I8(v) => write!(f, "{}", v),
            ParsedValue::I16(v) => write!(f, "{}", v),
            ParsedValue::I32(v) => write!(f, "{}", v),
            ParsedValue::I64(v) => write!(f, "{}", v),
            ParsedValue::I128(v) => write!(f, "{}", v),
            ParsedValue::Bool(v) => write!(f, "{}", v),
            ParsedValue::String(v) => write!(f, "\"{}\"", v),
            ParsedValue::Pubkey(v) => write!(f, "{}", v),
            ParsedValue::Vec(v) => {
                write!(f, "[")?;
                for (i, val) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", val)?;
                }
                write!(f, "]")
            }
            ParsedValue::Bytes(v) => write!(f, "0x{}", hex::encode(v)),
            ParsedValue::Struct(fields) => {
                write!(f, "{{")?;
                for (i, (name, val)) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", name, val)?;
                }
                write!(f, "}}")
            }
            ParsedValue::Unknown(v) => write!(f, "0x{}", hex::encode(v)),
        }
    }
}

impl std::fmt::Display for FieldInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(value) = &self.value {
            write!(f, "{}: {} = {}", self.name, self.type_name, value)
        } else {
            write!(f, "{}: {}", self.name, self.type_name)
        }
    }
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
