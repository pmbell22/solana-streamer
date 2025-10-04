use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a complete IDL (Interface Definition Language) file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Idl {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub instructions: Vec<IdlInstruction>,
    #[serde(default)]
    pub accounts: Vec<IdlAccountDef>,
    #[serde(default)]
    pub types: Vec<IdlTypeDef>,
    #[serde(default)]
    pub events: Vec<IdlEvent>,
    #[serde(default)]
    pub errors: Vec<IdlErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<IdlMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

/// Metadata for an IDL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

/// Represents an instruction in the IDL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlInstruction {
    pub name: String,
    #[serde(default)]
    pub docs: Vec<String>,
    pub accounts: Vec<IdlAccountItem>,
    pub args: Vec<IdlField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<IdlDiscriminator>,
}

/// Account item in an instruction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlAccountItem {
    pub name: String,
    #[serde(default)]
    pub docs: Vec<String>,
    #[serde(alias = "isMut", alias = "writable", default)]
    pub is_mut: bool,
    #[serde(alias = "isSigner", alias = "signer", default)]
    pub is_signer: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pda: Option<IdlPda>,
}

/// PDA (Program Derived Address) definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlPda {
    pub seeds: Vec<IdlSeed>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program_id: Option<IdlSeed>,
}

/// Seed for PDA derivation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum IdlSeed {
    Const { value: Vec<u8> },
    Arg { path: String },
    Account { path: String },
}

/// Field definition (for instruction args or type fields)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlField {
    pub name: String,
    #[serde(default)]
    pub docs: Vec<String>,
    #[serde(rename = "type")]
    pub ty: IdlType,
}

/// Type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IdlType {
    // Complex types (must come first for untagged enum)
    Vec { vec: Box<IdlType> },
    Option { option: Box<IdlType> },
    Array { array: (Box<IdlType>, usize) },

    // Defined types - two formats supported
    // Format 1: { "defined": "TypeName" }
    DefinedSimple { defined: String },
    // Format 2: { "defined": { "name": "TypeName" } }
    DefinedComplex { defined: IdlDefinedType },

    // Simple string representation (catches all string types like "u8", "bool", etc.)
    Simple(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlDefinedType {
    pub name: String,
}

/// Account definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlAccountDef {
    pub name: String,
    #[serde(default)]
    pub docs: Vec<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub ty: Option<IdlAccountDefTy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<IdlDiscriminator>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlAccountDefTy {
    pub kind: String,
    #[serde(default)]
    pub fields: Vec<IdlField>,
}

/// Type definition for custom types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlTypeDef {
    pub name: String,
    #[serde(default)]
    pub docs: Vec<String>,
    #[serde(rename = "type")]
    pub ty: IdlTypeDefTy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum IdlTypeDefTy {
    Struct { kind: String, #[serde(default)] fields: Vec<IdlField> },
    Enum { kind: String, #[serde(default)] variants: Vec<IdlEnumVariant> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlEnumVariant {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<IdlField>>,
}

/// Event definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlEvent {
    pub name: String,
    #[serde(default)]
    pub docs: Vec<String>,
    #[serde(default)]
    pub fields: Vec<IdlField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discriminator: Option<IdlDiscriminator>,
}

/// Error code definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlErrorCode {
    pub code: u32,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg: Option<String>,
}

/// Discriminator for instructions/events (8-byte identifier)
pub type IdlDiscriminator = Vec<u8>;

/// Instruction discriminator lookup
pub type InstructionDiscriminators = HashMap<String, Vec<u8>>;
