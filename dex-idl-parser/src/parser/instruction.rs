use crate::idl::{Idl, IdlInstruction, IdlType, InstructionDiscriminators};
use crate::types::{FieldInfo, ParsedInstruction, ParsedInstructionData, ParsedValue, RoutePlanStep};
use anyhow::{anyhow, Result};
use borsh::BorshDeserialize;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use std::io::Cursor;

/// Instruction parser that uses IDL to parse transaction instructions
pub struct InstructionParser {
    idl: Idl,
    discriminators: InstructionDiscriminators,
    reverse_discriminators: HashMap<Vec<u8>, String>,
}

impl InstructionParser {
    /// Create a new instruction parser from an IDL
    pub fn new(idl: Idl) -> Self {
        let discriminators = crate::idl::build_instruction_discriminators(&idl);

        // Build reverse lookup: discriminator -> instruction name
        let reverse_discriminators: HashMap<Vec<u8>, String> = discriminators
            .iter()
            .map(|(name, disc)| (disc.clone(), name.clone()))
            .collect();

        Self {
            idl,
            discriminators,
            reverse_discriminators,
        }
    }

    /// Parse instruction data to identify the instruction
    pub fn parse_instruction(
        &self,
        instruction_data: &[u8],
        accounts: &[Pubkey],
    ) -> Result<ParsedInstruction> {
        // Extract discriminator (first 8 bytes for Anchor programs)
        if instruction_data.len() < 8 {
            return Err(anyhow!("Instruction data too short (< 8 bytes)"));
        }

        let discriminator = &instruction_data[0..8];

        // Look up instruction name
        let instruction_name = self
            .reverse_discriminators
            .get(discriminator)
            .ok_or_else(|| anyhow!("Unknown instruction discriminator: {:?}", hex::encode(discriminator)))?;

        // Get instruction definition
        let instruction_def = self
            .idl
            .instructions
            .iter()
            .find(|i| &i.name == instruction_name)
            .ok_or_else(|| anyhow!("Instruction definition not found: {}", instruction_name))?;

        // Parse accounts
        let parsed_accounts = self.parse_accounts(instruction_def, accounts)?;

        // Parse instruction arguments (after discriminator)
        let args_data = &instruction_data[8..];
        let parsed_args = self.parse_args(instruction_def, args_data)?;

        Ok(ParsedInstruction {
            program: self.idl.name.clone(),
            instruction: instruction_name.clone(),
            accounts: parsed_accounts,
            data: parsed_args,
            raw_discriminator: discriminator.to_vec(),
        })
    }

    /// Parse accounts based on IDL account definitions
    fn parse_accounts(
        &self,
        instruction_def: &IdlInstruction,
        accounts: &[Pubkey],
    ) -> Result<HashMap<String, Pubkey>> {
        let mut parsed_accounts = HashMap::new();

        for (i, account_def) in instruction_def.accounts.iter().enumerate() {
            if i >= accounts.len() {
                return Err(anyhow!(
                    "Not enough accounts provided. Expected at least {}, got {}",
                    i + 1,
                    accounts.len()
                ));
            }

            parsed_accounts.insert(account_def.name.clone(), accounts[i]);
        }

        Ok(parsed_accounts)
    }

    /// Parse instruction arguments
    fn parse_args(
        &self,
        instruction_def: &IdlInstruction,
        data: &[u8],
    ) -> Result<ParsedInstructionData> {
        let mut cursor = Cursor::new(data);
        let mut field_infos = Vec::new();

        // Try to deserialize each field
        for arg in &instruction_def.args {
            // Special handling for Jupiter routePlan
            let value = if arg.name == "routePlan" {
                Self::deserialize_route_plan(&mut cursor).ok()
            } else {
                Self::deserialize_field(&arg.ty, &mut cursor).ok()
            };

            field_infos.push(FieldInfo {
                name: arg.name.clone(),
                type_name: Self::format_idl_type(&arg.ty),
                value,
            });
        }

        Ok(ParsedInstructionData {
            fields: field_infos,
            raw_data: data.to_vec(),
        })
    }

    /// Deserialize Jupiter route plan
    fn deserialize_route_plan(cursor: &mut Cursor<&[u8]>) -> Result<ParsedValue> {
        let steps = Vec::<RoutePlanStep>::deserialize_reader(cursor)?;
        Ok(ParsedValue::RoutePlan(steps))
    }

    /// Deserialize a field value based on its IDL type
    fn deserialize_field(ty: &IdlType, cursor: &mut Cursor<&[u8]>) -> Result<ParsedValue> {
        match ty {
            IdlType::Simple(type_name) => match type_name.as_str() {
                "u8" => Ok(ParsedValue::U8(u8::deserialize_reader(cursor)?)),
                "u16" => Ok(ParsedValue::U16(u16::deserialize_reader(cursor)?)),
                "u32" => Ok(ParsedValue::U32(u32::deserialize_reader(cursor)?)),
                "u64" => Ok(ParsedValue::U64(u64::deserialize_reader(cursor)?)),
                "u128" => Ok(ParsedValue::U128(u128::deserialize_reader(cursor)?)),
                "i8" => Ok(ParsedValue::I8(i8::deserialize_reader(cursor)?)),
                "i16" => Ok(ParsedValue::I16(i16::deserialize_reader(cursor)?)),
                "i32" => Ok(ParsedValue::I32(i32::deserialize_reader(cursor)?)),
                "i64" => Ok(ParsedValue::I64(i64::deserialize_reader(cursor)?)),
                "i128" => Ok(ParsedValue::I128(i128::deserialize_reader(cursor)?)),
                "bool" => Ok(ParsedValue::Bool(bool::deserialize_reader(cursor)?)),
                "publicKey" | "pubkey" => {
                    let bytes = <[u8; 32]>::deserialize_reader(cursor)?;
                    Ok(ParsedValue::Pubkey(Pubkey::from(bytes)))
                }
                "string" => Ok(ParsedValue::String(String::deserialize_reader(cursor)?)),
                "bytes" => {
                    let bytes = Vec::<u8>::deserialize_reader(cursor)?;
                    Ok(ParsedValue::Bytes(bytes))
                }
                _ => {
                    // Unknown type, read remaining bytes
                    let pos = cursor.position() as usize;
                    let remaining = &cursor.get_ref()[pos..];
                    Ok(ParsedValue::Unknown(remaining.to_vec()))
                }
            },
            IdlType::Vec { vec } => {
                let len = u32::deserialize_reader(cursor)? as usize;
                let mut values = Vec::new();
                for _ in 0..len {
                    values.push(Self::deserialize_field(vec, cursor)?);
                }
                Ok(ParsedValue::Vec(values))
            }
            IdlType::Option { option } => {
                let is_some = u8::deserialize_reader(cursor)? != 0;
                if is_some {
                    Self::deserialize_field(option, cursor)
                } else {
                    Ok(ParsedValue::Unknown(vec![]))
                }
            }
            IdlType::Array { array } => {
                let mut values = Vec::new();
                for _ in 0..array.1 {
                    values.push(Self::deserialize_field(&array.0, cursor)?);
                }
                Ok(ParsedValue::Vec(values))
            }
            IdlType::DefinedSimple { .. } | IdlType::DefinedComplex { .. } => {
                // For complex/defined types, we'd need the type definition from IDL
                // For now, treat as unknown and capture remaining bytes
                let pos = cursor.position() as usize;
                let remaining = &cursor.get_ref()[pos..];
                Ok(ParsedValue::Unknown(remaining.to_vec()))
            }
        }
    }

    /// Format an IDL type as a readable string
    fn format_idl_type(ty: &IdlType) -> String {
        match ty {
            IdlType::Simple(s) => s.clone(),
            IdlType::Vec { vec } => format!("Vec<{}>", Self::format_idl_type(vec)),
            IdlType::Option { option } => format!("Option<{}>", Self::format_idl_type(option)),
            IdlType::Array { array } => {
                format!("[{}; {}]", Self::format_idl_type(&array.0), array.1)
            }
            IdlType::DefinedSimple { defined } => defined.clone(),
            IdlType::DefinedComplex { defined } => defined.name.clone(),
        }
    }

    /// Get instruction definition by name
    pub fn get_instruction(&self, name: &str) -> Option<&IdlInstruction> {
        self.idl.instructions.iter().find(|i| i.name == name)
    }

    /// Get discriminator for an instruction name
    pub fn get_discriminator(&self, instruction_name: &str) -> Option<&Vec<u8>> {
        self.discriminators.get(instruction_name)
    }

    /// Get IDL reference
    pub fn idl(&self) -> &Idl {
        &self.idl
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::idl::load_idl_from_file;

    #[test]
    fn test_parse_jupiter_route_instruction() {
        let idl = load_idl_from_file("idls/jupiter_agg_v6.json").unwrap();
        let parser = InstructionParser::new(idl);

        // Get route instruction discriminator
        let disc = parser.get_discriminator("route");
        assert!(disc.is_some());
        println!("Route discriminator: {:?}", hex::encode(disc.unwrap()));
    }

    #[test]
    fn test_parse_raydium_swap_instruction() {
        let idl = load_idl_from_file("idls/raydium_clmm.json").unwrap();
        let parser = InstructionParser::new(idl);

        // Check that we can find swap instructions
        let swap_inst = parser.get_instruction("swap_v2");
        assert!(swap_inst.is_some());
    }

    #[test]
    fn test_parse_orca_swap_instruction() {
        let idl = load_idl_from_file("idls/orca_whirlpool.json").unwrap();
        let parser = InstructionParser::new(idl);

        // Check that we can find swap instruction
        let swap_inst = parser.get_instruction("swap");
        assert!(swap_inst.is_some());
    }
}
