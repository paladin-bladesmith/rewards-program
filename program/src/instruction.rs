//! Program instruction types.

use {
    shank::ShankInstruction,
    solana_program::{
        instruction::{AccountMeta, Instruction},
        program_error::ProgramError,
        pubkey::Pubkey,
        system_program,
    },
};

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
        writable,
        name = "extra_account_metas",
        desc = "Transfer hook extra account metas account."
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
        writable,
        name = "owner",
        desc = "Token account owner.",
    )]
    #[account(
        2,
        writable,
        name = "holder_rewards",
        desc = "Holder rewards account.",
    )]
    #[account(
        3,
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
        writable,
        name = "holder_rewards",
        desc = "Holder rewards account.",
    )]
    #[account(
        2,
        writable,
        name = "token_account",
        desc = "Token account.",
    )]
    #[account(
        3,
        name = "mint",
        desc = "Token mint.",
    )]
    #[account(
        4,
        optional,
        writable,
        name = "sponsor",
        desc = "Sponsor of this account, required if rent_debt is non zero",
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
        name = "holder_rewards",
        desc = "Holder rewards account.",
    )]
    #[account(
        2,
        name = "token_account",
        desc = "Token account.",
    )]
    #[account(
        3,
        name = "mint",
        desc = "Token mint.",
    )]
    #[account(
        4,
        writable,
        name = "owner",
        desc = "Owner of the account.",
    )]
    CloseHolderRewards,
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
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

/// Creates an
/// [InitializeHolderRewardsPool](enum.PaladinRewardsInstruction.html)
/// instruction.
pub fn initialize_holder_rewards_pool(
    holder_rewards_pool_address: &Pubkey,
    extra_account_metas_address: &Pubkey,
    mint_address: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*holder_rewards_pool_address, false),
        AccountMeta::new(*extra_account_metas_address, false),
        AccountMeta::new_readonly(*mint_address, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let data = PaladinRewardsInstruction::InitializeHolderRewardsPool.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates an [InitializeHolderRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
pub fn initialize_holder_rewards(
    holder_rewards_pool_address: &Pubkey,
    holder_rewards_address: &Pubkey,
    token_account_address: &Pubkey,
    mint_address: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*holder_rewards_pool_address, false),
        AccountMeta::new(*holder_rewards_address, false),
        AccountMeta::new_readonly(*token_account_address, false),
        AccountMeta::new_readonly(*mint_address, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let data = PaladinRewardsInstruction::InitializeHolderRewards.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a [HarvestRewards](enum.PaladinRewardsInstruction.html) instruction.
pub fn harvest_rewards(
    holder_rewards_pool_address: &Pubkey,
    holder_rewards_address: &Pubkey,
    token_account_address: &Pubkey,
    mint_address: &Pubkey,
) -> Instruction {
    let accounts: Vec<_> = [
        AccountMeta::new(*holder_rewards_pool_address, false),
        AccountMeta::new(*holder_rewards_address, false),
        AccountMeta::new(*token_account_address, false),
        AccountMeta::new_readonly(*mint_address, false),
    ]
    .into_iter()
    .collect();
    let data = PaladinRewardsInstruction::HarvestRewards.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a [CloseHolderRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
pub fn close_holder_rewards(
    holder_rewards_pool_address: Pubkey,
    holder_rewards_address: Pubkey,
    token_account_address: Pubkey,
    mint_address: Pubkey,
    close_authority: Pubkey,
    owner: Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(holder_rewards_pool_address, false),
        AccountMeta::new(holder_rewards_address, false),
        AccountMeta::new_readonly(token_account_address, false),
        AccountMeta::new_readonly(mint_address, false),
        AccountMeta::new(close_authority, true),
        AccountMeta::new(owner, false),
    ];
    let data = PaladinRewardsInstruction::CloseHolderRewards.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
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
}
