use crate::idl::{load_idl_from_file, DexProtocol};
use crate::parser::InstructionParser;
use crate::types::DexEvent;
use anyhow::{Context, Result};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// Multi-DEX parser for streaming events
pub struct DexStreamParser {
    parsers: HashMap<DexProtocol, InstructionParser>,
    protocol_by_program_id: HashMap<String, DexProtocol>,
}

impl DexStreamParser {
    /// Create a new DEX stream parser with specified protocols
    pub fn new(protocols: Vec<DexProtocol>) -> Result<Self> {
        let mut parsers = HashMap::new();
        let mut protocol_by_program_id = HashMap::new();

        for protocol in protocols {
            // Load IDL for this protocol
            let idl = load_idl_from_file(protocol.idl_path())
                .with_context(|| format!("Failed to load IDL for {}", protocol.name()))?;

            // Create parser
            let parser = InstructionParser::new(idl);
            parsers.insert(protocol, parser);

            // Map program ID to protocol
            protocol_by_program_id.insert(protocol.program_id().to_string(), protocol);
        }

        Ok(Self {
            parsers,
            protocol_by_program_id,
        })
    }

    /// Create parser with all supported protocols
    pub fn new_all_protocols() -> Result<Self> {
        Self::new(vec![
            // DexProtocol::JupiterV6,
            DexProtocol::RaydiumClmm,
            // DexProtocol::RaydiumCpmm,
            // DexProtocol::RaydiumAmmV4,
            DexProtocol::OrcaWhirlpool,
            DexProtocol::MeteoraDlmm,
        ])
    }

    /// Parse a transaction instruction to a DEX event
    pub fn parse_instruction(
        &self,
        program_id: &Pubkey,
        instruction_data: &[u8],
        accounts: &[Pubkey],
        signature: String,
        slot: u64,
        block_time: i64,
        transaction_index: Option<u64>,
    ) -> Result<DexEvent> {
        // Identify protocol by program ID
        let protocol = self
            .protocol_by_program_id
            .get(&program_id.to_string())
            .ok_or_else(|| anyhow::anyhow!("Unknown program ID: {}", program_id))?;

        // Get parser for this protocol
        let parser = self
            .parsers
            .get(protocol)
            .ok_or_else(|| anyhow::anyhow!("No parser for protocol: {:?}", protocol))?;

        // Parse instruction
        let parsed_instruction = parser.parse_instruction(instruction_data, accounts)?;

        // Create DEX event
        Ok(DexEvent::new(
            *protocol,
            signature,
            slot,
            block_time,
            parsed_instruction,
            transaction_index,
        ))
    }

    /// Parse from Yellowstone gRPC transaction data
    pub fn parse_from_grpc_transaction(
        &self,
        grpc_tx: &yellowstone_grpc_proto::geyser::SubscribeUpdateTransactionInfo,
        slot: u64,
        block_time: Option<&prost_types::Timestamp>,
    ) -> Vec<DexEvent> {
        let mut events = Vec::new();

        let Some(tx_data) = &grpc_tx.transaction else {
            return events;
        };

        let Some(message) = &tx_data.message else {
            return events;
        };

        // Get the first signature (primary transaction signature)
        let signature = if let Some(sig) = tx_data.signatures.first() {
            bs58::encode(sig).into_string()
        } else {
            return events; // No signature, skip this transaction
        };

        // Extract block time
        let block_time_secs = block_time
            .map(|t| t.seconds)
            .unwrap_or(0);

        // Parse each instruction
        for (idx, instruction) in message.instructions.iter().enumerate() {
            // Get program ID
            let program_id_index = instruction.program_id_index as usize;
            if program_id_index >= message.account_keys.len() {
                continue;
            }

            let program_id_bytes = &message.account_keys[program_id_index];
            let Ok(program_id) = Pubkey::try_from(program_id_bytes.as_slice()) else {
                continue;
            };

            // Check if this is a DEX program we're tracking
            if !self.protocol_by_program_id.contains_key(&program_id.to_string()) {
                continue;
            }

            // Get instruction accounts
            let mut accounts = Vec::new();
            for &account_index in &instruction.accounts {
                if (account_index as usize) < message.account_keys.len() {
                    if let Ok(pubkey) = Pubkey::try_from(message.account_keys[account_index as usize].as_slice()) {
                        accounts.push(pubkey);
                    }
                }
            }

            // Parse instruction
            if let Ok(event) = self.parse_instruction(
                &program_id,
                &instruction.data,
                &accounts,
                signature.clone(),
                slot,
                block_time_secs,
                Some(idx as u64),
            ) {
                events.push(event);
            }
        }

        events
    }

    /// Get supported program IDs
    pub fn supported_program_ids(&self) -> Vec<String> {
        self.protocol_by_program_id.keys().cloned().collect()
    }

    /// Get parser for a specific protocol
    pub fn get_parser(&self, protocol: &DexProtocol) -> Option<&InstructionParser> {
        self.parsers.get(protocol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_create_parser() {
        let parser = DexStreamParser::new_all_protocols();
        if let Err(e) = &parser {
            eprintln!("Parser creation failed: {:?}", e);
        }
        assert!(parser.is_ok());

        let parser = parser.unwrap();
        let program_ids = parser.supported_program_ids();
        assert_eq!(program_ids.len(), 6);
    }

    #[test]
    fn test_protocol_lookup() {
        let parser = DexStreamParser::new_all_protocols().unwrap();

        let jupiter_id = Pubkey::from_str("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4").unwrap();
        assert!(parser.protocol_by_program_id.contains_key(&jupiter_id.to_string()));
    }
}
