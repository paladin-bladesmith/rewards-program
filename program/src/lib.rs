#![allow(unexpected_cfgs)]
//! The Paladin Rewards program.
//!
//! Manages the distribution of rewards to token holders based on their share
//! of the total token supply. Holders earn shares of rewards proportional to
//! their share of token supply.

#[cfg(all(target_os = "solana", feature = "bpf-entrypoint"))]
mod entrypoint;
pub mod error;
pub mod extra_metas;
pub mod instruction;
pub mod processor;
pub mod state;

solana_program::declare_id!("7LdHk6jnrY4kJW79mVXshTzduvgn3yz4hZzHpzTbt7Ph");
