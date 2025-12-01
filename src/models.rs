use serde::{Deserialize, Serialize};

/// Represents a command suggestion from the AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSuggestion {
    /// The actual command to execute
    pub command: String,
    /// Human-readable explanation of what the command does
    pub explanation: String,
    /// Optional safety warning
    pub warning: Option<String>,
}

/// System context information to help AI generate better suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContext {
    /// Operating system (Windows, macOS, Linux)
    pub os: String,
    /// Current shell (powershell, bash, zsh, cmd, etc.)
    pub shell: String,
    /// Current working directory
    pub current_dir: String,
}

impl SystemContext {
    /// Gather system context information
    ///
    /// # Arguments
    /// * `shell_override` - Optional shell override from config. If provided, uses this instead of auto-detection.
    pub fn gather(shell_override: Option<String>) -> anyhow::Result<Self> {
        let os = std::env::consts::OS.to_string();
        let shell = shell_override.unwrap_or_else(detect_shell);
        let current_dir = std::env::current_dir()?.to_string_lossy().to_string();

        Ok(Self {
            os,
            shell,
            current_dir,
        })
    }
}

/// Detect the current shell
fn detect_shell() -> String {
    // Check SHELL environment variable (Unix-like systems)
    if let Ok(shell) = std::env::var("SHELL") {
        return shell.split('/').last().unwrap_or("unknown").to_string();
    }

    // On Windows, check for PowerShell or cmd
    #[cfg(windows)]
    {
        // Check if running in PowerShell
        if std::env::var("PSModulePath").is_ok() {
            return "powershell".to_string();
        }
        return "cmd".to_string();
    }

    #[cfg(not(windows))]
    {
        "bash".to_string()
    }
}
