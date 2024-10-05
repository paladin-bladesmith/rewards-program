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
        signer,
        name = "mint_authority",
        desc = "Mint authority.",
    )]
    #[account(
        4,
        name = "system_program",
        desc = "System program.",
    )]
    InitializeHolderRewardsPool,
    /// Moves SOL rewards to the holder rewards pool and updates the total.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w, s]` Payer account.
    /// 1. `[w]` Holder rewards pool account.
    /// 2. `[ ]` Token mint.
    /// 3. `[ ]` System program.
    #[account(
        0,
        writable,
        signer,
        name = "payer",
        desc = "Payer account.",
    )]
    #[account(
        1,
        writable,
        name = "holder_rewards_pool",
        desc = "Holder rewards pool account."
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
    DistributeRewards(u64),
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
        name = "holder_rewards_pool",
        desc = "Holder rewards pool account.",
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
        name = "system_program",
        desc = "System program.",
    )]
    InitializeHolderRewards(Pubkey),
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
        name = "sponsor",
        desc = "Sponsor of this account, required if rent_debt is non zero",
        optional
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
        name = "authority",
        desc = "Either the owner or the sponsor can close the account.",
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
            PaladinRewardsInstruction::DistributeRewards(amount) => {
                let mut data = Vec::with_capacity(9);
                data.push(1);
                data.extend_from_slice(&amount.to_le_bytes());
                data
            }
            PaladinRewardsInstruction::InitializeHolderRewards(sponsor) => {
                let mut data = Vec::with_capacity(33);
                data.push(2);
                data.extend_from_slice(bytemuck::bytes_of(sponsor));
                data
            }
            PaladinRewardsInstruction::HarvestRewards => vec![3],
            PaladinRewardsInstruction::CloseHolderRewards => vec![4],
        }
    }

    /// Unpacks a byte buffer into a
    /// [PaladinRewardsInstruction](enum.PaladinRewardsInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        match input.split_first() {
            Some((&0, _)) => Ok(PaladinRewardsInstruction::InitializeHolderRewardsPool),
            Some((&1, rest)) => {
                let amount = rest
                    .get(..8)
                    .and_then(|slice| Some(u64::from_le_bytes(slice.try_into().ok()?)))
                    .ok_or(ProgramError::InvalidInstructionData)?;
                Ok(PaladinRewardsInstruction::DistributeRewards(amount))
            }
            Some((&2, rest)) => {
                let sponsor = rest
                    .get(..32)
                    .map(bytemuck::from_bytes)
                    .ok_or(ProgramError::InvalidInstructionData)?;

                Ok(PaladinRewardsInstruction::InitializeHolderRewards(*sponsor))
            }
            Some((&3, _)) => Ok(PaladinRewardsInstruction::HarvestRewards),
            Some((&4, _)) => Ok(PaladinRewardsInstruction::CloseHolderRewards),
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
    mint_authority_address: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*holder_rewards_pool_address, false),
        AccountMeta::new(*extra_account_metas_address, false),
        AccountMeta::new_readonly(*mint_address, false),
        AccountMeta::new_readonly(*mint_authority_address, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let data = PaladinRewardsInstruction::InitializeHolderRewardsPool.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a [DistributeRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
pub fn distribute_rewards(
    payer_address: &Pubkey,
    holder_rewards_pool_address: &Pubkey,
    mint: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*payer_address, true),
        AccountMeta::new(*holder_rewards_pool_address, false),
        AccountMeta::new_readonly(*mint, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let data = PaladinRewardsInstruction::DistributeRewards(amount).pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates an [InitializeHolderRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
pub fn initialize_holder_rewards(
    holder_rewards_pool_address: &Pubkey,
    holder_rewards_address: &Pubkey,
    token_account_address: &Pubkey,
    mint_address: &Pubkey,
    sponsor: Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*holder_rewards_pool_address, false),
        AccountMeta::new(*holder_rewards_address, false),
        AccountMeta::new_readonly(*token_account_address, false),
        AccountMeta::new_readonly(*mint_address, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let data = PaladinRewardsInstruction::InitializeHolderRewards(sponsor).pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a [HarvestRewards](enum.PaladinRewardsInstruction.html) instruction.
pub fn harvest_rewards(
    holder_rewards_pool_address: &Pubkey,
    holder_rewards_address: &Pubkey,
    token_account_address: &Pubkey,
    mint_address: &Pubkey,
    sponsor: Option<Pubkey>,
) -> Instruction {
    let accounts: Vec<_> = [
        AccountMeta::new(*holder_rewards_pool_address, false),
        AccountMeta::new(*holder_rewards_address, false),
        AccountMeta::new(*token_account_address, false),
        AccountMeta::new_readonly(*mint_address, false),
    ]
    .into_iter()
    .chain(sponsor.map(|key| AccountMeta::new(key, false)))
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
    authority: Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(holder_rewards_pool_address, false),
        AccountMeta::new(holder_rewards_address, false),
        AccountMeta::new_readonly(token_account_address, false),
        AccountMeta::new_readonly(mint_address, false),
        AccountMeta::new(authority, true),
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
    fn test_pack_unpack_distribute_rewards() {
        let original = PaladinRewardsInstruction::DistributeRewards(500_000_000);
        let packed = original.pack();
        let unpacked = PaladinRewardsInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_initialize_holder_rewards_unsponsored() {
        let original = PaladinRewardsInstruction::InitializeHolderRewards(Pubkey::default());
        let packed = original.pack();
        let unpacked = PaladinRewardsInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_initialize_holder_rewards_sponsored() {
        let original =
            PaladinRewardsInstruction::InitializeHolderRewards(Pubkey::new_from_array([1; 32]));
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
}
