//! Axon - Biological-inspired CLI Agent
//! 
//! A high-performance, memory-safe CLI agent written in Rust.

pub mod config;
pub mod memory;
pub mod atoms;
pub mod llm;
pub mod executor;
pub mod cli;
pub mod skills;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
