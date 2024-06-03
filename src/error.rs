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
    /// Incorrect distribution account address.
    #[error("Incorrect distribution account address")]
    IncorrectDistributionAccountAddress,
    /// Incorrect extra metas account address.
    #[error("Incorrect extra metas account address")]
    IncorrectExtraMetasAccountAddress,
}
