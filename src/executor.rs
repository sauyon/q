use crate::models::CommandSuggestion;
use anyhow::{Context, Result};
use colored::*;
use dialoguer::{Confirm, Input};
use regex::Regex;
use std::process::Command;

pub struct Executor {
    auto_confirm: bool,
    show_explanation: bool,
}

impl Executor {
    pub fn new(auto_confirm: bool, show_explanation: bool) -> Self {
        Self {
            auto_confirm,
            show_explanation,
        }
    }

    /// Display the command suggestion and optionally execute it
    pub fn handle_suggestion(&self, suggestion: CommandSuggestion) -> Result<()> {
        // Display the suggested command
        println!("\n{}", "💡 Suggested command:".bright_cyan().bold());
        println!("{}", suggestion.command.bright_white());

        // Display explanation if enabled
        if self.show_explanation {
            println!("\n{}", "Explanation:".bright_yellow());
            println!("{}", suggestion.explanation);
        }

        // Display warning if present
        if let Some(warning) = &suggestion.warning {
            println!("\n{}", "⚠️  WARNING:".bright_red().bold());
            println!("{}", warning.bright_red());
        }

        // Ask for confirmation unless auto-confirm is enabled
        let should_execute = if self.auto_confirm {
            true
        } else {
            println!();
            Confirm::new()
                .with_prompt("Run this command?")
                .default(false)
                .interact()
                .context("Failed to get user confirmation")?
        };

        if should_execute {
            let final_command = self.resolve_variables(&suggestion.command)?;
            self.execute_command(&final_command)?;
        } else {
            println!("{}", "Command not executed.".bright_black());
        }

        Ok(())
    }

    /// Execute a command in the appropriate shell
    fn execute_command(&self, command: &str) -> Result<()> {
        println!("\n{}", "Executing...".bright_green());

        let output = if cfg!(windows) {
            // On Windows, use PowerShell by default
            Command::new("powershell")
                .arg("-NoProfile")
                .arg("-Command")
                .arg(command)
                .output()
                .context("Failed to execute command")?
        } else {
            // On Unix-like systems, use bash
            Command::new("bash")
                .arg("-c")
                .arg(command)
                .output()
                .context("Failed to execute command")?
        };

        // Display stdout
        if !output.stdout.is_empty() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("{}", stdout);
        }

        // Display stderr
        if !output.stderr.is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("{}", stderr.bright_red());
        }

        // Check exit status
        if !output.status.success() {
            let code = output.status.code().unwrap_or(-1);
            anyhow::bail!("Command failed with exit code: {}", code);
        }

        println!("\n{}", "✓ Command completed successfully".bright_green());

        Ok(())
    }

    /// Find variables in the command ({{VAR}}) and prompt the user for values
    fn resolve_variables(&self, command: &str) -> Result<String> {
        let re = Regex::new(r"\{\{([A-Z0-9_]+)\}\}").unwrap();
        let mut final_command = command.to_string();
        let mut variables_found = false;

        // Find all unique variables
        let mut vars = Vec::new();
        for cap in re.captures_iter(command) {
            let var_name = cap.get(1).unwrap().as_str().to_string();
            if !vars.contains(&var_name) {
                vars.push(var_name);
            }
        }

        if !vars.is_empty() {
            println!("\n{}", "📝 Input required:".bright_yellow());
            variables_found = true;
        }

        for var in vars {
            let value: String = Input::new()
                .with_prompt(format!("Enter value for {}", var))
                .interact_text()
                .context("Failed to read input")?;
            
            final_command = final_command.replace(&format!("{{{{{}}}}}", var), &value);
        }

        if variables_found {
            println!("\n{}", "Final command:".bright_cyan());
            println!("{}", final_command.bright_white());
        }

        Ok(final_command)
    }
}