//! Program instruction types.

use {arrayref::array_ref, shank::ShankInstruction, solana_program::program_error::ProgramError};

/// Instructions supported by the Paladin Rewards program.
#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, ShankInstruction)]
pub enum PaladinRewardsInstruction {
    /// Configures a holder rewards pool for a mint that has been configured
    /// with the rewards program as a transfer hook program.
    ///
    /// This instruction will:
    ///
    /// - Initialize a holder rewards pool account.
    /// - Initialize the required accounts for the transfer hook.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w]` Holder rewards pool account.
    /// 1. `[w]` Transfer hook extra account metas account.
    /// 2. `[ ]` Token mint.
    /// 3. `[s]` Mint authority.
    /// 4. `[ ]` System program.
    #[account(
        0,
        writable,
        name = "holder_rewards_pool",
        desc = "Holder rewards pool account."
    )]
    #[account(
        1,
        name = "holder_rewards_pool_token_account",
        desc = "Holder rewards pool token account."
    )]
    #[account(
        2,
        name = "mint",
        desc = "Token mint.",
    )]
    #[account(
        3,
        name = "system_program",
        desc = "System program.",
    )]
    InitializeHolderRewardsPool,
    /// Initializes a holder rewards account for a token account.
    ///
    /// This instruction will evaluate the token account's share of the total
    /// supply of the mint and use that to calculate the holder rewards
    /// account's share of the total rewards pool.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[ ]` Holder rewards pool account.
    /// 1. `[w]` Holder rewards account.
    /// 2. `[ ]` Token account.
    /// 3. `[ ]` Token mint.
    /// 4. `[ ]` System program.
    #[account(
        0,
        writable,
        name = "holder_rewards_pool",
        desc = "Holder rewards pool account.",
    )]
    #[account(
        1,
        name = "holder_rewards_pool_token_account",
        desc = "Holder rewards pool token account."
    )]
    #[account(
        2,
        signer,
        writable,
        name = "owner",
        desc = "Token account owner.",
    )]
    #[account(
        3,
        writable,
        name = "holder_rewards",
        desc = "Holder rewards account.",
    )]
    #[account(
        4,
        name = "mint",
        desc = "Token mint.",
    )]
    #[account(
        5,
        name = "system_program",
        desc = "System program.",
    )]
    InitializeHolderRewards,
    /// Moves accrued SOL rewards into the provided token account based on the
    /// share of the total rewards pool represented in the holder rewards
    /// account.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w]` Holder rewards pool account.
    /// 1. `[w]` Holder rewards account.
    /// 2. `[w]` Token account.
    /// 3. `[ ]` Token mint.
    /// 4. `[w]?` Sponsor account if rent_debt is non zero.
    #[account(
        0,
        writable,
        name = "holder_rewards_pool",
        desc = "Holder rewards pool account."
    )]
    #[account(
        1,
        name = "holder_rewards_pool_token_account",
        desc = "Holder rewards pool token account."
    )]
    #[account(
        2,
        writable,
        name = "holder_rewards",
        desc = "Holder rewards account.",
    )]
    #[account(
        3,
        name = "mint",
        desc = "Token mint.",
    )]
    #[account(
        4,
        signer,
        writable,
        name = "owner",
        desc = "owner of token account",
    )]
    HarvestRewards,
    /// Closes the provided holder rewards account.
    #[account(
        0,
        writable,
        name = "holder_rewards_pool",
        desc = "Holder rewards pool account."
    )]
    #[account(
        1,
        writable,
        name = "holder_rewards_pool_token_account",
        desc = "Holder rewards pool token account."
    )]
    #[account(
        2,
        writable,
        name = "holder_rewards",
        desc = "Holder rewards account.",
    )]
    #[account(
        3,
        name = "mint",
        desc = "Token mint.",
    )]
    #[account(
        4,
        signer,
        writable,
        name = "owner",
        desc = "Owner of the account.",
    )]
    CloseHolderRewards,
    #[account(
        0,
        writable,
        name = "holder_rewards_pool",
        desc = "Holder rewards pool account."
    )]
    #[account(
        1,
        writable,
        name = "holder_rewards_pool_token_account",
        desc = "Holder rewards pool token account."
    )]
    #[account(
        2,
        writable,
        name = "holder_rewards",
        desc = "Holder rewards account.",
    )]
    #[account(
        3,
        writable,
        name = "token_account",
        desc = "Token account.",
    )]
    #[account(
        4,
        name = "mint",
        desc = "Token mint.",
    )]
    #[account(
        5,
        writable,
        signer,
        name = "owner",
        desc = "Owner of the account.",
    )]
    #[account(
        6,
        name = "token program",
        desc = "token program",
    )]
    Deposit { amount: u64},
     #[account(
        0,
        writable,
        name = "holder_rewards_pool",
        desc = "Holder rewards pool account."
    )]
    #[account(
        1,
        writable,
        name = "holder_rewards_pool_token_account",
        desc = "Holder rewards pool token account."
    )]
    #[account(
        2,
        writable,
        name = "holder_rewards",
        desc = "Holder rewards account.",
    )]
    #[account(
        3,
        writable,
        name = "token_account",
        desc = "Token account.",
    )]
    #[account(
        4,
        name = "mint",
        desc = "Token mint.",
    )]
    #[account(
        5,
        writable,
        signer,
        name = "owner",
        desc = "Owner of the account.",
    )]
    #[account(
        6,
        name = "token program",
        desc = "token program",
    )]
    Withdraw { amount: u64},
}

impl PaladinRewardsInstruction {
    /// Packs a
    /// [PaladinRewardsInstruction](enum.PaladinRewardsInstruction.html)
    /// into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        match self {
            PaladinRewardsInstruction::InitializeHolderRewardsPool => vec![0],
            PaladinRewardsInstruction::InitializeHolderRewards => vec![1],
            PaladinRewardsInstruction::HarvestRewards => vec![2],
            PaladinRewardsInstruction::CloseHolderRewards => vec![3],
            PaladinRewardsInstruction::Deposit { amount } => {
                let mut data = Vec::with_capacity(9);
                data.push(4);
                data.extend_from_slice(&amount.to_le_bytes());
                data
            }
            PaladinRewardsInstruction::Withdraw { amount } => {
                let mut data = Vec::with_capacity(9);
                data.push(5);
                data.extend_from_slice(&amount.to_le_bytes());
                data
            }
        }
    }

    /// Unpacks a byte buffer into a
    /// [PaladinRewardsInstruction](enum.PaladinRewardsInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        match input.split_first() {
            Some((&0, _)) => Ok(PaladinRewardsInstruction::InitializeHolderRewardsPool),
            Some((&1, _)) => Ok(PaladinRewardsInstruction::InitializeHolderRewards),
            Some((&2, _)) => Ok(PaladinRewardsInstruction::HarvestRewards),
            Some((&3, _)) => Ok(PaladinRewardsInstruction::CloseHolderRewards),
            Some((&4, rest)) if rest.len() == 8 => {
                let amount = u64::from_le_bytes(*array_ref![rest, 0, 8]);

                Ok(PaladinRewardsInstruction::Deposit { amount })
            }
            Some((&5, rest)) if rest.len() == 8 => {
                let amount = u64::from_le_bytes(*array_ref![rest, 0, 8]);

                Ok(PaladinRewardsInstruction::Withdraw { amount })
            }
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack_initialize_holder_rewards_pool() {
        let original = PaladinRewardsInstruction::InitializeHolderRewardsPool;
        let packed = original.pack();
        let unpacked = PaladinRewardsInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_initialize_holder_rewards() {
        let original = PaladinRewardsInstruction::InitializeHolderRewards;
        let packed = original.pack();
        let unpacked = PaladinRewardsInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_harvest_rewards() {
        let original = PaladinRewardsInstruction::HarvestRewards;
        let packed = original.pack();
        let unpacked = PaladinRewardsInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_close_holder_rewards() {
        let original = PaladinRewardsInstruction::CloseHolderRewards;
        let packed = original.pack();
        let unpacked = PaladinRewardsInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_deposit() {
        let original = PaladinRewardsInstruction::Deposit { amount: 1000 };
        let packed = original.pack();
        let unpacked = PaladinRewardsInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_withdraw() {
        let original = PaladinRewardsInstruction::Withdraw { amount: 1000 };
        let packed = original.pack();
        let unpacked = PaladinRewardsInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }
}
