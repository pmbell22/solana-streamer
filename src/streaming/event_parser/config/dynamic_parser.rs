use super::schema::{FieldType, InstructionConfig, ProtocolConfig};
use crate::streaming::event_parser::{
    common::{EventMetadata, EventType, ProtocolType},
    core::event_parser::GenericEventParseConfig,
    UnifiedEvent,
};
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::collections::HashMap;

/// Dynamic event that stores data from config-based parsing
#[derive(Debug, Clone)]
pub struct DynamicEvent {
    pub metadata: EventMetadata,
    pub instruction_name: String,
    pub accounts: HashMap<String, Pubkey>,
    pub data_fields: HashMap<String, DynamicFieldValue>,
}

/// Dynamic field value supporting multiple types
#[derive(Debug, Clone)]
pub enum DynamicFieldValue {
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
    Pubkey(Pubkey),
    String(String),
}

impl UnifiedEvent for DynamicEvent {
    fn event_type(&self) -> EventType {
        self.metadata.event_type.clone()
    }

    fn signature(&self) -> &Signature {
        &self.metadata.signature
    }

    fn slot(&self) -> u64 {
        self.metadata.slot
    }

    fn recv_us(&self) -> i64 {
        self.metadata.recv_us
    }

    fn handle_us(&self) -> i64 {
        self.metadata.handle_us
    }

    fn set_handle_us(&mut self, handle_us: i64) {
        self.metadata.handle_us = handle_us;
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    fn clone_boxed(&self) -> Box<dyn UnifiedEvent> {
        Box::new(self.clone())
    }

    fn set_swap_data(&mut self, _swap_data: crate::streaming::event_parser::common::SwapData) {
        // Can be implemented if needed
    }

    fn swap_data_is_parsed(&self) -> bool {
        false
    }

    fn outer_index(&self) -> i64 {
        self.metadata.outer_index
    }

    fn inner_index(&self) -> Option<i64> {
        self.metadata.inner_index
    }

    fn transaction_index(&self) -> Option<u64> {
        self.metadata.transaction_index
    }
}

/// Parser factory for dynamic config-based parsing
pub struct DynamicEventParser {
    /// Protocol configs indexed by instruction discriminator
    pub instruction_map: std::collections::HashMap<Vec<u8>, (ProtocolConfig, InstructionConfig)>,
}

impl DynamicEventParser {
    /// Create a new dynamic parser from protocol config
    pub fn new(protocol_config: ProtocolConfig) -> anyhow::Result<Self> {
        let mut instruction_map = std::collections::HashMap::new();

        for instruction in &protocol_config.instructions {
            let discriminator = instruction.discriminator_bytes()?;
            instruction_map.insert(
                discriminator,
                (protocol_config.clone(), instruction.clone()),
            );
        }

        Ok(Self { instruction_map })
    }

    /// Create parser configs from a protocol config
    /// Note: This stores instruction configs in global state for the parser function to access
    pub fn create_configs(
        protocol_config: &ProtocolConfig,
    ) -> anyhow::Result<Vec<GenericEventParseConfig>> {
        use once_cell::sync::Lazy;
        use parking_lot::RwLock;

        // Global storage for dynamic configs
        static DYNAMIC_CONFIGS: Lazy<RwLock<std::collections::HashMap<Vec<u8>, (ProtocolConfig, InstructionConfig)>>> =
            Lazy::new(|| RwLock::new(std::collections::HashMap::new()));

        let mut configs = Vec::new();
        let mut global_configs = DYNAMIC_CONFIGS.write();

        for instruction in &protocol_config.instructions {
            let discriminator = instruction.discriminator_bytes()?;
            let inner_discriminator = instruction.inner_discriminator_bytes()?.unwrap_or_default();

            // Create event type from config
            let event_type = EventType::Custom(instruction.event_type.clone());
            let protocol_type = ProtocolType::Custom(protocol_config.name.clone());

            // Store in global map for parser function to access
            global_configs.insert(
                discriminator.clone(),
                (protocol_config.clone(), instruction.clone()),
            );

            let config = GenericEventParseConfig {
                program_id: protocol_config.program_id,
                protocol_type,
                inner_instruction_discriminator: Box::leak(inner_discriminator.into_boxed_slice()),
                instruction_discriminator: Box::leak(discriminator.into_boxed_slice()),
                event_type,
                inner_instruction_parser: None,
                instruction_parser: Some(parse_dynamic_instruction),
                requires_inner_instruction: instruction.requires_inner_instruction,
            };

            configs.push(config);
        }

        Ok(configs)
    }

    /// Parse a dynamic event from instruction data
    fn parse_dynamic_event(
        _protocol_config: &ProtocolConfig,
        instruction_config: &InstructionConfig,
        data: &[u8],
        accounts: &[Pubkey],
        metadata: EventMetadata,
    ) -> Option<Box<dyn UnifiedEvent>> {
        // Parse account fields
        let mut account_map = HashMap::new();
        for (idx, account_field) in instruction_config.accounts.iter().enumerate() {
            if let Some(pubkey) = accounts.get(idx) {
                account_map.insert(account_field.name.clone(), *pubkey);
            }
        }

        // Parse data fields
        let mut data_fields = HashMap::new();
        for field in &instruction_config.data_fields {
            if let Some(value) = Self::parse_field(data, field.offset, &field.field_type) {
                data_fields.insert(field.name.clone(), value);
            }
        }

        Some(Box::new(DynamicEvent {
            metadata,
            instruction_name: instruction_config.name.clone(),
            accounts: account_map,
            data_fields,
        }))
    }

    /// Parse a single field from instruction data
    fn parse_field(data: &[u8], offset: usize, field_type: &FieldType) -> Option<DynamicFieldValue> {
        match field_type {
            FieldType::U8 => {
                if offset < data.len() {
                    Some(DynamicFieldValue::U8(data[offset]))
                } else {
                    None
                }
            }
            FieldType::U16 => {
                if offset + 2 <= data.len() {
                    let bytes = [data[offset], data[offset + 1]];
                    Some(DynamicFieldValue::U16(u16::from_le_bytes(bytes)))
                } else {
                    None
                }
            }
            FieldType::U32 => {
                if offset + 4 <= data.len() {
                    let bytes = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
                    Some(DynamicFieldValue::U32(u32::from_le_bytes(bytes)))
                } else {
                    None
                }
            }
            FieldType::U64 => {
                if offset + 8 <= data.len() {
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&data[offset..offset + 8]);
                    Some(DynamicFieldValue::U64(u64::from_le_bytes(bytes)))
                } else {
                    None
                }
            }
            FieldType::U128 => {
                if offset + 16 <= data.len() {
                    let mut bytes = [0u8; 16];
                    bytes.copy_from_slice(&data[offset..offset + 16]);
                    Some(DynamicFieldValue::U128(u128::from_le_bytes(bytes)))
                } else {
                    None
                }
            }
            FieldType::I8 => {
                if offset < data.len() {
                    Some(DynamicFieldValue::I8(data[offset] as i8))
                } else {
                    None
                }
            }
            FieldType::I16 => {
                if offset + 2 <= data.len() {
                    let bytes = [data[offset], data[offset + 1]];
                    Some(DynamicFieldValue::I16(i16::from_le_bytes(bytes)))
                } else {
                    None
                }
            }
            FieldType::I32 => {
                if offset + 4 <= data.len() {
                    let bytes = [data[offset], data[offset + 1], data[offset + 2], data[offset + 3]];
                    Some(DynamicFieldValue::I32(i32::from_le_bytes(bytes)))
                } else {
                    None
                }
            }
            FieldType::I64 => {
                if offset + 8 <= data.len() {
                    let mut bytes = [0u8; 8];
                    bytes.copy_from_slice(&data[offset..offset + 8]);
                    Some(DynamicFieldValue::I64(i64::from_le_bytes(bytes)))
                } else {
                    None
                }
            }
            FieldType::I128 => {
                if offset + 16 <= data.len() {
                    let mut bytes = [0u8; 16];
                    bytes.copy_from_slice(&data[offset..offset + 16]);
                    Some(DynamicFieldValue::I128(i128::from_le_bytes(bytes)))
                } else {
                    None
                }
            }
            FieldType::Bool => {
                if offset < data.len() {
                    Some(DynamicFieldValue::Bool(data[offset] != 0))
                } else {
                    None
                }
            }
            FieldType::Pubkey => {
                if offset + 32 <= data.len() {
                    let mut bytes = [0u8; 32];
                    bytes.copy_from_slice(&data[offset..offset + 32]);
                    Some(DynamicFieldValue::Pubkey(Pubkey::new_from_array(bytes)))
                } else {
                    None
                }
            }
            FieldType::String => {
                // Assume null-terminated or length-prefixed string
                // For simplicity, read until null or end
                let mut end = offset;
                while end < data.len() && data[end] != 0 {
                    end += 1;
                }
                if let Ok(s) = std::str::from_utf8(&data[offset..end]) {
                    Some(DynamicFieldValue::String(s.to_string()))
                } else {
                    None
                }
            }
            FieldType::Custom(_) => {
                // Custom types not yet supported in dynamic parsing
                None
            }
        }
    }
}

/// Global parser function for dynamic instructions
/// This is used as the InstructionEventParser for dynamically loaded configs
fn parse_dynamic_instruction(
    data: &[u8],
    accounts: &[Pubkey],
    metadata: EventMetadata,
) -> Option<Box<dyn UnifiedEvent>> {
    use once_cell::sync::Lazy;
    use parking_lot::RwLock;

    // Access the global config storage
    static DYNAMIC_CONFIGS: Lazy<RwLock<std::collections::HashMap<Vec<u8>, (ProtocolConfig, InstructionConfig)>>> =
        Lazy::new(|| RwLock::new(std::collections::HashMap::new()));

    // We need to find which instruction this is based on the event_type in metadata
    // Since we don't have direct access to the discriminator here, we'll iterate
    let configs = DYNAMIC_CONFIGS.read();

    for (_disc, (protocol_config, instruction_config)) in configs.iter() {
        let event_type_name = instruction_config.event_type.clone();
        if metadata.event_type == EventType::Custom(event_type_name) {
            return DynamicEventParser::parse_dynamic_event(
                protocol_config,
                instruction_config,
                data,
                accounts,
                metadata,
            );
        }
    }

    None
}
