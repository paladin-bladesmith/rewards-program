//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

use {num_derive::FromPrimitive, thiserror::Error};

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum PaladinRewardsError {
    /// 0 - Incorrect mint authority
    #[error("Incorrect mint authority")]
    IncorrectMintAuthority = 0x0,
    /// 1 - Incorrect transfer hook program ID
    #[error("Incorrect transfer hook program ID")]
    IncorrectTransferHookProgramId = 0x1,
    /// 2 - Incorrect holder rewards pool address
    #[error("Incorrect holder rewards pool address")]
    IncorrectHolderRewardsPoolAddress = 0x2,
    /// 3 - Incorrect extra metas address
    #[error("Incorrect extra metas address")]
    IncorrectExtraMetasAddress = 0x3,
    /// 4 - Incorrect holder rewards address
    #[error("Incorrect holder rewards address")]
    IncorrectHolderRewardsAddress = 0x4,
    /// 5 - Token account mint mismatch
    #[error("Token account mint mismatch")]
    TokenAccountMintMismatch = 0x5,
    /// 6 - Holder rewards sponsor account mismatch
    #[error("Holder rewards sponsor account mismatch")]
    IncorrectSponsorAddress = 0x6,
    /// 7 - Holder rewards has unclaimed rewards
    #[error("Holder rewards has unclaimed rewards")]
    CloseWithUnclaimedRewards = 0x7,
    /// 8 - Holder rewards token account balance must be zero
    #[error("Holder rewards token account balance must be zero")]
    InvalidClosingBalance = 0x8,
    /// 9 - Incorrect sweep address
    #[error("Incorrect sweep address")]
    IncorrectSweepAddress = 0x9,
}

impl solana_program::program_error::PrintProgramError for PaladinRewardsError {
    fn print<E>(&self) {
        solana_program::msg!(&self.to_string());
    }
}
