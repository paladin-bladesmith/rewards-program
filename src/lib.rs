//! Paladin Rewards program.

#[cfg(all(target_os = "solana", feature = "bpf-entrypoint"))]
mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

solana_program::declare_id!("6wsKX77nJ8CzjR2CUfbosKd1C42HfkQu9AwtoZtaLx9q");
