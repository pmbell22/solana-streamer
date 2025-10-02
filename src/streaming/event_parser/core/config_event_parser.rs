use super::event_parser::EventParser;
use crate::streaming::event_parser::{
    common::filter::EventTypeFilter, config::{ConfigLoader, DynamicEventParser, ProtocolConfig}, Protocol,
};
use anyhow::Result;
use solana_sdk::pubkey::Pubkey;
use std::path::Path;

/// Extended EventParser that supports config-based protocols
pub struct ConfigurableEventParser {
    /// Base event parser
    pub parser: EventParser,
    /// Loaded protocol configs
    pub configs: Vec<ProtocolConfig>,
}

impl ConfigurableEventParser {
    /// Create a new parser from static protocols and config files
    pub fn new(
        static_protocols: Vec<Protocol>,
        config_paths: Vec<&Path>,
        event_type_filter: Option<EventTypeFilter>,
    ) -> Result<Self> {
        // Load configs from files
        let mut configs = Vec::new();
        let mut dynamic_configs = Vec::new();

        for path in config_paths {
            let protocol_config = ConfigLoader::load_from_file(path)?;
            let parser_configs = DynamicEventParser::create_configs(&protocol_config)?;
            dynamic_configs.extend(parser_configs);
            configs.push(protocol_config);
        }

        // Create base parser with static protocols
        let mut parser = EventParser::new(static_protocols, event_type_filter.clone());

        // Merge dynamic configs into the parser
        for config in dynamic_configs {
            let discriminator = config.instruction_discriminator.to_vec();
            parser
                .instruction_configs
                .entry(discriminator)
                .or_insert_with(Vec::new)
                .push(config.clone());

            if !parser.program_ids.contains(&config.program_id) {
                parser.program_ids.push(config.program_id);
            }
        }

        Ok(Self { parser, configs })
    }

    /// Create from a directory of config files
    pub fn from_config_directory<P: AsRef<Path>>(
        static_protocols: Vec<Protocol>,
        config_dir: P,
        event_type_filter: Option<EventTypeFilter>,
    ) -> Result<Self> {
        let configs = ConfigLoader::load_from_directory(&config_dir)?;
        let mut all_configs = Vec::new();
        let mut dynamic_configs = Vec::new();

        for protocol_config in configs {
            let parser_configs = DynamicEventParser::create_configs(&protocol_config)?;
            dynamic_configs.extend(parser_configs);
            all_configs.push(protocol_config);
        }

        // Create base parser with static protocols
        let mut parser = EventParser::new(static_protocols, event_type_filter);

        // Merge dynamic configs into the parser
        for config in dynamic_configs {
            let discriminator = config.instruction_discriminator.to_vec();
            parser
                .instruction_configs
                .entry(discriminator)
                .or_insert_with(Vec::new)
                .push(config.clone());

            if !parser.program_ids.contains(&config.program_id) {
                parser.program_ids.push(config.program_id);
            }
        }

        Ok(Self {
            parser,
            configs: all_configs,
        })
    }

    /// Get all loaded protocol names
    pub fn protocol_names(&self) -> Vec<String> {
        self.configs.iter().map(|c| c.name.clone()).collect()
    }

    /// Get all program IDs
    pub fn program_ids(&self) -> &[Pubkey] {
        &self.parser.program_ids
    }
}

// Delegate all EventParser methods to the inner parser
impl std::ops::Deref for ConfigurableEventParser {
    type Target = EventParser;

    fn deref(&self) -> &Self::Target {
        &self.parser
    }
}

impl std::ops::DerefMut for ConfigurableEventParser {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.parser
    }
}
