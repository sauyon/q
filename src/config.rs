use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ai: AIConfig,
    pub execution: ExecutionConfig,
    pub context: ContextConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub default_provider: String,
    pub openrouter: Option<OpenRouterConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    pub api_key: String,
    #[serde(default = "default_model")]
    pub model: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
}

fn default_model() -> String {
    "anthropic/claude-4.5-sonnet".to_string()
}

fn default_base_url() -> String {
    "https://openrouter.ai/api/v1".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    #[serde(default)]
    pub auto_confirm: bool,
    #[serde(default = "default_true")]
    pub show_explanation: bool,
    #[serde(default)]
    pub copy_to_clipboard: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextConfig {
    #[serde(default = "default_true")]
    pub include_shell_info: bool,
    #[serde(default = "default_true")]
    pub include_directory: bool,
    #[serde(default)]
    pub include_history: bool,
    /// Optional override for the shell to use (e.g., "powershell", "bash", "zsh")
    /// If not set, the shell will be auto-detected
    pub shell: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ai: AIConfig {
                default_provider: "openrouter".to_string(),
                openrouter: None,
            },
            execution: ExecutionConfig {
                auto_confirm: false,
                show_explanation: true,
                copy_to_clipboard: false,
            },
            context: ContextConfig {
                include_shell_info: true,
                include_directory: true,
                include_history: false,
                shell: None,
            },
        }
    }
}

impl Config {
    /// Get the path to the config file
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("q");

        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;

        Ok(config_dir.join("config.toml"))
    }

    /// Load configuration from file, creating default if it doesn't exist
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            let default_config = Self::default();
            default_config.save()?;

            eprintln!("Created default config at: {}", config_path.display());
            eprintln!("Please edit this file to add your OpenRouter API key.");

            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path).context("Failed to read config file")?;

        let config: Config = toml::from_str(&content).context("Failed to parse config file")?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self).context("Failed to serialize config")?;

        fs::write(&config_path, content).context("Failed to write config file")?;

        Ok(())
    }

    /// Validate that required configuration is present
    pub fn validate(&self) -> Result<()> {
        match self.ai.default_provider.as_str() {
            "openrouter" => {
                if let Some(ref config) = self.ai.openrouter {
                    if config.api_key.is_empty() || config.api_key == "sk-or-v1-..." {
                        anyhow::bail!(
                            "OpenRouter API key not configured. Please edit {} and add your API key.",
                            Self::config_path()?.display()
                        );
                    }
                } else {
                    anyhow::bail!(
                        "OpenRouter is set as default provider but not configured. Please edit {}",
                        Self::config_path()?.display()
                    );
                }
            }
            other => {
                anyhow::bail!(
                    "Unknown provider: {}. Currently only 'openrouter' is supported.",
                    other
                );
            }
        }
        Ok(())
    }
}
