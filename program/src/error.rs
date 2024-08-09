//! Program error types.

use {
    num_derive::FromPrimitive,
    solana_program::{
        decode_error::DecodeError,
        msg,
        program_error::{PrintProgramError, ProgramError},
    },
    thiserror::Error,
};

/// Errors that can be returned by the Paladin Rewards program.
// Note: Shank does not export the type when we use `spl_program_error`.
#[derive(Error, Clone, Debug, Eq, PartialEq, FromPrimitive)]
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

impl PrintProgramError for PaladinRewardsError {
    fn print<E>(&self) {
        msg!(&self.to_string());
    }
}

impl From<PaladinRewardsError> for ProgramError {
    fn from(e: PaladinRewardsError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for PaladinRewardsError {
    fn type_of() -> &'static str {
        "PaladinRewardsError"
    }
}
