//! Program error types.

use spl_program_error::*;

/// Errors that can be returned by the Paladin Rewards program.
#[spl_program_error]
pub enum PaladinRewardsError {
    /// Incorrect mint authority.
    #[error("Incorrect mint authority")]
    IncorrectMintAuthority,
    /// Incorrect transfer hook program ID.
    #[error("Incorrect transfer hook program ID")]
    IncorrectTransferHookProgramId,
    /// Incorrect holder rewards pool address.
    #[error("Incorrect holder rewards pool address")]
    IncorrectHolderRewardsPoolAddress,
    /// Incorrect extra metas address.
    #[error("Incorrect extra metas address")]
    IncorrectExtraMetasAddress,
    /// Incorrect holder rewards address.
    #[error("Incorrect holder rewards address")]
    IncorrectHolderRewardsAddress,
    /// Token account mint mismatch.
    #[error("Token account mint mismatch")]
    TokenAccountMintMismatch,
}
