use crate::idl::{Idl, IdlInstruction, InstructionDiscriminators};
use crate::types::{ParsedInstruction, ParsedInstructionData};
use anyhow::{anyhow, Result};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

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
    /// Note: This is a simplified implementation. For production, you'd want
    /// proper borsh deserialization based on the IDL type definitions
    fn parse_args(
        &self,
        instruction_def: &IdlInstruction,
        data: &[u8],
    ) -> Result<ParsedInstructionData> {
        // For now, return raw data and field names
        // In a full implementation, you would deserialize based on the IDL types
        let field_names: Vec<String> = instruction_def
            .args
            .iter()
            .map(|arg| arg.name.clone())
            .collect();

        Ok(ParsedInstructionData {
            fields: field_names,
            raw_data: data.to_vec(),
        })
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
