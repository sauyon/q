mod ai;
mod config;
mod executor;
mod models;

use ai::openrouter::OpenRouterProvider;
use ai::AIProvider;
use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use config::Config;
use executor::Executor;
use models::SystemContext;

#[derive(Parser)]
#[command(name = "q")]
#[command(about = "AI-powered terminal command assistant", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// The query describing what you want to do
    #[arg(allow_hyphen_values = true)]
    query: Vec<String>,

    /// Show the config file path and exit
    #[arg(long)]
    config_path: bool,

    /// Skip confirmation and execute immediately (use with caution!)
    #[arg(short = 'y', long)]
    yes: bool,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Configure the application
    Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Handle --config-path flag
    if cli.config_path {
        let path = Config::config_path()?;
        println!("{}", path.display());
        return Ok(());
    }

    // Handle subcommands
    if let Some(Commands::Config) = cli.command {
        let mut config = Config::load().unwrap_or_else(|_| Config::default());

        println!("{}", "Configuring OpenRouter...".bright_cyan());
        println!("Enter your OpenRouter API Key:");

        let mut api_key = String::new();
        std::io::stdin()
            .read_line(&mut api_key)
            .context("Failed to read input")?;
        let api_key = api_key.trim().to_string();

        if api_key.is_empty() {
            eprintln!("{}", "API Key cannot be empty.".bright_red());
            std::process::exit(1);
        }

        if let Some(ref mut openrouter) = config.ai.openrouter {
            openrouter.api_key = api_key;
        } else {
            config.ai.openrouter = Some(config::OpenRouterConfig {
                api_key,
                model: "anthropic/claude-3.5-sonnet".to_string(),
                base_url: "https://openrouter.ai/api/v1".to_string(),
            });
        }

        config.save().context("Failed to save configuration")?;
        println!("{}", "Configuration saved successfully!".bright_green());
        return Ok(());
    }

    // Load configuration
    let config = Config::load().context("Failed to load configuration")?;

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("{}", format!("Configuration error: {}", e).bright_red());
        eprintln!(
            "\nTo edit your config, run: {}",
            "q --config-path".bright_cyan()
        );
        std::process::exit(1);
    }

    // Join query parts into a single string
    let query = cli.query.join(" ");

    if query.trim().is_empty() {
        // If no query and no subcommand, show help
        use clap::CommandFactory;
        Cli::command().print_help()?;
        return Ok(());
    }

    // Gather system context
    let context = SystemContext::gather(config.context.shell.clone())
        .context("Failed to gather system context")?;

    // Create AI provider
    let provider: Box<dyn AIProvider> = match config.ai.default_provider.as_str() {
        "openrouter" => {
            let openrouter_config = config
                .ai
                .openrouter
                .context("OpenRouter configuration not found")?;
            Box::new(OpenRouterProvider::new(openrouter_config))
        }
        other => {
            anyhow::bail!("Unsupported provider: {}", other);
        }
    };

    // Show a loading indicator
    println!("{}", "🤔 Thinking...".bright_cyan());

    // Generate command suggestion
    let suggestion = provider
        .generate_command(&query, &context)
        .await
        .context("Failed to generate command suggestion")?;

    // Create executor with config settings (override auto_confirm if -y flag is used)
    let auto_confirm = cli.yes || config.execution.auto_confirm;
    let executor = Executor::new(auto_confirm, config.execution.show_explanation);

    // Handle the suggestion (display and optionally execute)
    executor.handle_suggestion(suggestion)?;

    Ok(())
}
