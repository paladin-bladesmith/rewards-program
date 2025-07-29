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
    /// Incorrect holder rewards pool address.
    #[error("Incorrect holder rewards pool address")]
    IncorrectHolderRewardsPoolAddress,
    /// Incorrect holder rewards address.
    #[error("Incorrect holder rewards address")]
    IncorrectHolderRewardsAddress,
    /// Token account mint mismatch.
    #[error("Token account mint mismatch")]
    TokenAccountMintMismatch,
    /// Attempted to close a holder rewards account that had unclaimed rewards.
    #[error("Holder rewards has unclaimed rewards")]
    CloseWithUnclaimedRewards,
    /// Cannot close holder rewards with current balance.
    #[error("Cannot close holder rewards with current balance")]
    InvalidClosingBalance,
    /// Owner is not the signer.
    #[error("Owner is not the signer")]
    OwnerNotSigner,
    /// Signer not owner of token account.
    #[error("Signer not owner of token account")]
    NotOwnerTokenAccount,
    /// Rewards amount exceeds pool balance.
    #[error("Rewards amount exceeds pool balance")]
    RewardsExcessPoolBalance,
    /// Holder rewards has deposited tokens.
    #[error("Holder rewards has deposited tokens")]
    CloseWithDepositedTokens,
    /// Holder doesn't have any deposited tokens to withdraw.
    #[error("Holder doesn't have any deposited tokens to withdraw")]
    NoDepositedTokensToWithdraw,
    /// Pool doesn't have enough balance to withdraw.
    #[error("Pool doesn't have enough balance to withdraw")]
    WithdrawExceedsPoolBalance,
    /// Token account owner mismatch.
    #[error("Token account owner mismatch")]
    TokenAccountOwnerMissmatch,
    /// Token account is frozen.
    #[error("Token account is frozen")]
    TokenAccountFrozen,
    /// Owner doesn'thave enough tokens to deposit.
    #[error("Owner doesn'thave enough tokens to deposit")]
    NotEnoughTokenToDeposit,
    /// Withdraw amount exceeds deposited
    #[error("Withdraw amount exceeds deposited")]
    WithdrawExceedsDeposited,
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
