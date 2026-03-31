//! CLI Module - Dendrite
//!
//! Command-line interface and argument parsing.

use crate::atoms::create_default_registry;
use crate::config::Config;
use crate::executor::Executor;
use crate::llm::LlmClient;
use crate::llm::LlmConfig;
use crate::memory::Memory;
use crate::memory::Message;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Parser)]
#[command(name = "axon")]
#[command(version = "2.0.0")]
#[command(about = "Axon - Biological-inspired CLI Agent", long_about = None)]
pub struct Cli {
    /// Config file path
    #[arg(short, long, default_value = "config.yaml")]
    pub config: PathBuf,

    /// Memory file path
    #[arg(short, long, default_value = "memory.jsonl")]
    pub memory: PathBuf,

    /// Override LLM model
    #[arg(long)]
    pub model: Option<String>,

    /// Override API key
    #[arg(short, long)]
    pub api_key: Option<String>,

    /// Disable memory persistence
    #[arg(long)]
    pub no_memory: bool,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Quiet mode (only output results)
    #[arg(short, long)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Interactive mode (default)
    Run {
        /// Override system persona
        #[arg(long)]
        system: Option<String>,
    },

    /// Execute single command
    Exec {
        /// Command to execute
        command: String,

        /// Stream output
        #[arg(short, long)]
        stream: bool,
    },

    /// Chat mode
    Chat {
        /// Continue previous conversation
        #[arg(long)]
        continue_: bool,

        /// Clear conversation history
        #[arg(long)]
        clear: bool,
    },

    /// Atom management
    Atom {
        #[command(subcommand)]
        subcommand: AtomSubcommand,
    },

    /// Memory management
    Memory {
        #[command(subcommand)]
        subcommand: MemorySubcommand,
    },
}

#[derive(Subcommand)]
pub enum AtomSubcommand {
    /// List all available atoms
    List,

    /// Show atom details
    Info { name: String },
}

#[derive(Subcommand)]
pub enum MemorySubcommand {
    /// Show memory content
    Show,

    /// Clear all memory
    Clear,

    /// Export memory to file
    Export { path: PathBuf },

    /// Import memory from file
    Import { path: PathBuf },
}

/// Run the CLI
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    // Load config
    let config = match Config::load(&cli.config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    // Handle commands that don't need full initialization
    if let Some(Command::Atom { subcommand }) = &cli.command {
        return handle_atom_command(subcommand);
    }

    if let Some(Command::Memory { subcommand }) = &cli.command {
        let rt = tokio::runtime::Runtime::new()?;
        return rt.block_on(handle_memory_command(subcommand, &cli.memory));
    }

    // Initialize components
    let registry = create_default_registry();

    let llm_config = LlmConfig {
        model: cli.model.clone().unwrap_or(config.llm.model.clone()),
        api_key: cli.api_key.clone().unwrap_or(config.llm.api_key.clone()),
        base_url: config.llm.base_url.clone(),
        timeout_secs: config.llm.timeout_secs,
    };

    let llm = match LlmClient::new(llm_config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error initializing LLM client: {}", e);
            std::process::exit(1);
        }
    };

    let memory = Arc::new(Mutex::new(Memory::new(cli.memory.clone())));

    let executor = Executor::new(llm, registry, memory);

    // Handle commands
    let rt = tokio::runtime::Runtime::new()?;

    match cli.command {
        Some(Command::Exec { command, .. }) => {
            rt.block_on(handle_exec(&executor, &command, cli.quiet))?;
        }
        Some(Command::Run { system }) => {
            rt.block_on(handle_run(&executor, system.as_deref(), cli.quiet))?;
        }
        Some(Command::Chat { continue_, clear }) => {
            rt.block_on(handle_chat(&executor, continue_, clear, cli.quiet))?;
        }
        Some(Command::Atom { .. }) | Some(Command::Memory { .. }) => {
            // Already handled above - this should never be reached
        }
        None => {
            // Default: interactive mode
            rt.block_on(handle_run(
                &executor,
                Some(&config.system.persona),
                cli.quiet,
            ))?;
        }
    }

    Ok(())
}

fn handle_atom_command(subcommand: &AtomSubcommand) -> Result<()> {
    let registry = create_default_registry();

    match subcommand {
        AtomSubcommand::List => {
            println!("Available atoms:");
            for (name, desc) in registry.list() {
                println!("  {} - {}", name, desc);
            }
        }
        AtomSubcommand::Info { name } => {
            if let Some(atom) = registry.get(name) {
                println!("Atom: {}", atom.name());
                println!("Description: {}", atom.description());
            } else {
                eprintln!("Atom not found: {}", name);
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

async fn handle_memory_command(subcommand: &MemorySubcommand, memory_path: &Path) -> Result<()> {
    let mut memory = Memory::new(memory_path.to_path_buf());

    match subcommand {
        MemorySubcommand::Show => {
            memory.load().await?;
            println!("Memory contents:");
            for msg in memory.get_messages() {
                println!("[{}] {}", msg.role, msg.content.as_str().unwrap_or(""));
            }
        }
        MemorySubcommand::Clear => {
            memory.clear().await?;
            println!("Memory cleared.");
        }
        MemorySubcommand::Export { path } => {
            memory.load().await?;
            memory.export(path).await?;
            println!("Memory exported.");
        }
        MemorySubcommand::Import { path } => {
            memory.import(path).await?;
            println!("Memory imported.");
        }
    }

    Ok(())
}

async fn handle_exec(executor: &Executor, command: &str, quiet: bool) -> Result<()> {
    if !quiet {
        println!("Executing: {}", command);
    }

    match executor.execute_once(command).await {
        Ok(response) => {
            println!("{}", response);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn handle_run(executor: &Executor, system: Option<&str>, quiet: bool) -> Result<()> {
    if !quiet {
        println!("Axon - Interactive Mode (Ctrl+C to exit)");
        if let Some(s) = system {
            println!("System: {}", &s[..s.len().min(80)]);
        }
    }

    // Inject system prompt and load history on first call
    {
        let mut memory = executor.memory().lock().await;
        memory.load().await?;

        if let Some(persona) = system {
            // Only inject if no system message already exists
            let has_system = memory.get_messages().iter().any(|m| m.role == "system");
            if !has_system {
                memory.add_message(Message::system(persona));
                memory
                    .append(&Message::system(persona))
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to save system prompt: {}", e))?;
            }
        }
    }

    loop {
        print!("> ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "quit" {
            break;
        }

        match executor.execute_once(input).await {
            Ok(response) => {
                println!("{}", response);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }

    Ok(())
}

async fn handle_chat(
    executor: &Executor,
    continue_: bool,
    clear: bool,
    quiet: bool,
) -> Result<()> {
    if !quiet {
        println!("Axon - Chat Mode");
    }

    if clear {
        let memory = executor.memory().lock().await;
        memory.clear().await?;
        if !quiet {
            println!("Chat history cleared.");
        }
    }

    if !continue_ {
        // Load and clear history for new chat
        {
            let mut memory = executor.memory().lock().await;
            memory.load().await?;
            memory.clear().await?;
        }
    }

    handle_run(executor, None, quiet).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let cli = Cli::parse_from(&["axon", "exec", "hello"]);
        assert!(matches!(cli.command, Some(Command::Exec { .. })));
    }
}
