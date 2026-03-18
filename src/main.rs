//! Axon - Biological-inspired CLI Agent
//!
//! A high-performance, memory-safe CLI agent written in Rust.

pub mod atoms;
pub mod cli;
pub mod config;
pub mod executor;
pub mod llm;
pub mod memory;
pub mod skills;

use anyhow::Result;

fn main() -> Result<()> {
    cli::run()
}
