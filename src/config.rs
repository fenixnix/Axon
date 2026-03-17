//! Configuration Module - Genome
//! 
//! Manages configuration loading and validation.

use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("配置文件不存在: {0}")]
    FileNotFound(String),
    
    #[error("配置解析失败: {0}")]
    ParseError(String),
    
    #[error("配置验证失败: {0}")]
    ValidationError(String),
    
    #[error("缺少必需配置: {0}")]
    MissingField(String),
    
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
}

/// Main configuration structure
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub core: CoreConfig,
    pub llm: LlmConfig,
    pub system: SystemConfig,
    pub atoms: AtomsConfig,
}

/// Core configuration
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct CoreConfig {
    pub name: String,
    pub version: String,
}

/// LLM provider configuration
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct LlmConfig {
    pub model: String,
    #[serde(default = "default_api_key")]
    pub api_key: String,
    pub base_url: String,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_api_key() -> String {
    std::env::var("OPENAI_API_KEY").unwrap_or_default()
}

fn default_timeout() -> u64 {
    60
}

/// System configuration (persona, etc.)
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct SystemConfig {
    pub persona: String,
}

/// Atoms configuration
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct AtomsConfig {
    #[serde(default)]
    pub active: Vec<String>,
}

impl Default for AtomsConfig {
    fn default() -> Self {
        Self {
            active: vec![
                "shell_exec".to_string(),
                "file_read".to_string(),
                "file_write".to_string(),
            ],
        }
    }
}

impl Config {
    /// Load configuration from YAML file
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::FileNotFound(path.display().to_string()));
        }
        
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)
            .map_err(|e| ConfigError::ParseError(e.to_string()))?;
        
        config.validate()?;
        Ok(config)
    }
    
    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate core config
        if self.core.name.is_empty() {
            return Err(ConfigError::MissingField("core.name".to_string()));
        }
        
        // Validate LLM config
        if self.llm.model.is_empty() {
            return Err(ConfigError::MissingField("llm.model".to_string()));
        }
        if self.llm.base_url.is_empty() {
            return Err(ConfigError::MissingField("llm.base_url".to_string()));
        }
        
        // Validate system config
        if self.system.persona.is_empty() {
            return Err(ConfigError::MissingField("system.persona".to_string()));
        }
        
        Ok(())
    }
    
    /// Check if an atom is enabled
    pub fn get_atom_enabled(&self, name: &str) -> bool {
        self.atoms.active.is_empty() || self.atoms.active.contains(&name.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_config_load() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("config.yaml");
        
        let yaml = r#"
core:
  name: "Axon"
  version: "2.0.0"

llm:
  model: "gpt-4"
  api_key: "test-key"
  base_url: "https://api.openai.com/v1"
  timeout_secs: 60

system:
  persona: "You are Axon."

atoms:
  active:
    - shell_exec
"#;
        std::fs::write(&config_path, yaml).unwrap();
        
        let config = Config::load(&config_path).unwrap();
        assert_eq!(config.core.name, "Axon");
        assert_eq!(config.llm.model, "gpt-4");
    }
    
    #[test]
    fn test_atom_enabled() {
        let config = Config {
            core: CoreConfig {
                name: "Axon".to_string(),
                version: "2.0.0".to_string(),
            },
            llm: LlmConfig {
                model: "gpt-4".to_string(),
                api_key: "test".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
                timeout_secs: 60,
            },
            system: SystemConfig {
                persona: "You are Axon.".to_string(),
            },
            atoms: AtomsConfig {
                active: vec!["shell_exec".to_string()],
            },
        };
        
        assert!(config.get_atom_enabled("shell_exec"));
        assert!(!config.get_atom_enabled("file_read"));
    }
}
