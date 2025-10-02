use super::schema::ProtocolConfig;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

/// Configuration file loader supporting multiple formats
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load a protocol configuration from a file
    /// Supports .json and .toml files based on extension
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<ProtocolConfig> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        match extension {
            "json" => Self::load_from_json(&content),
            "toml" => Self::load_from_toml(&content),
            _ => anyhow::bail!(
                "Unsupported config file format: {}. Use .json or .toml",
                extension
            ),
        }
    }

    /// Load from JSON string
    pub fn load_from_json(json: &str) -> Result<ProtocolConfig> {
        let config: ProtocolConfig = serde_json::from_str(json)
            .context("Failed to parse JSON config")?;
        config.validate()?;
        Ok(config)
    }

    /// Load from TOML string
    pub fn load_from_toml(toml: &str) -> Result<ProtocolConfig> {
        let config: ProtocolConfig = toml::from_str(toml)
            .context("Failed to parse TOML config")?;
        config.validate()?;
        Ok(config)
    }

    /// Load multiple configs from a directory
    pub fn load_from_directory<P: AsRef<Path>>(dir: P) -> Result<Vec<ProtocolConfig>> {
        let dir = dir.as_ref();
        let mut configs = Vec::new();

        if !dir.is_dir() {
            anyhow::bail!("{} is not a directory", dir.display());
        }

        for entry in fs::read_dir(dir)
            .with_context(|| format!("Failed to read directory: {}", dir.display()))?
        {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if extension == "json" || extension == "toml" {
                    match Self::load_from_file(&path) {
                        Ok(config) => configs.push(config),
                        Err(e) => {
                            log::warn!("Failed to load config from {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }

        Ok(configs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_json() {
        let json = r#"{
            "name": "test_protocol",
            "version": "1.0.0",
            "program_id": "11111111111111111111111111111111",
            "instructions": [
                {
                    "name": "test_instruction",
                    "discriminator": "09",
                    "event_type": "TestEvent",
                    "accounts": []
                }
            ]
        }"#;

        let config = ConfigLoader::load_from_json(json).unwrap();
        assert_eq!(config.name, "test_protocol");
        assert_eq!(config.version, "1.0.0");
    }
}
