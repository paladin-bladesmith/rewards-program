//! Program error types.

use spl_program_error::*;

/// Errors that can be returned by the Paladin Rewards program.
#[spl_program_error]
pub enum PaladinRewardsError {
    /// Placeholder.
    #[error("This is a placeholder error")]
    Placeholder,
}
