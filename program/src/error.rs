#![allow(non_local_definitions)]
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
    /// Provided sponsor account did not match expected sponsor.
    #[error("Holder rewards sponsor account mismatch")]
    IncorrectSponsorAddress,
    /// Attempted to close a holder rewards account that had unclaimed rewards.
    #[error("Holder rewards has unclaimed rewards")]
    CloseWithUnclaimedRewards,
    /// Cannot close holder rewards with current balance.
    #[error("Cannot close holder rewards with current balance")]
    InvalidClosingBalance,
    /// Invalid extension.
    #[error("Invalid extension")]
    InvalidExtension,
    /// Owner is not the signer.
    #[error("Owner is not the signer")]
    OwnerNotSigner,
    /// Signer not owner of token account.
    #[error("Signer not owner of token account")]
    SignerIsNotOwnerTokenAccount,
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
