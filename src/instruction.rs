//! Program instruction types.

use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};

/// Instructions supported by the Paladin Rewards program.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PaladinRewardsInstruction {
    /// Configures rewards for a mint that has been configured with the rewards
    /// program as a transfer hook program.
    ///
    /// This instruction will:
    ///
    /// - Initialize a rewards account for the mint (distribution account),
    ///   configured with the distribution addresses:
    ///   - Piggy bank
    ///   - Staked PAL rewards
    /// - Initialize an active rewards account for the mint.
    /// - Initialize the required accounts for the transfer hook.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w]` Distribution account.
    /// 1. `[w]` Transfer hook extra account metas account.
    /// 2. `[ ]` Token mint.
    /// 3. `[s]` Mint authority.
    /// 4. `[ ]` System program.
    InitializeMintRewardInfo {
        piggy_bank_address: Pubkey,
        staked_rewards_address: Pubkey,
    },
    /// Moves SOL rewards to the following parties:
    ///
    /// - 1%  Piggy bank
    /// - 4%  Staked PAL (validators)
    /// - 5%  All PAL holders
    /// - 90% Leader who produces the block
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w]` Distribution account.
    /// 1. `[w]` Piggy bank account.
    /// 2. `[w]` Staked PAL rewards account.
    /// 3. `[w]` Leader account.
    DistributeRewards,
    /// Initializes holder reward info by storing the last seen total rewards
    /// in the distribution account.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w]` Holder rewards account.
    /// 1. `[ ]` PAL token account.
    /// 2. `[ ]` PAL token mint.
    /// 3. `[ ]` System program.
    InitializeHolderRewardInfo,
    /// Moves accrued SOL rewards into the provided PAL token account.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w]` Distribution account.
    /// 1. `[w]` Holder rewards account.
    /// 2. `[ ]` PAL token account.
    /// 3. `[ ]` PAL token mint.
    HarvestRewards,
}

impl PaladinRewardsInstruction {
    /// Packs a
    /// [PaladinRewardsInstruction](enum.PaladinRewardsInstruction.html)
    /// into a byte buffer.
    pub fn pack(&self) -> Vec<u8> {
        match self {
            PaladinRewardsInstruction::InitializeMintRewardInfo {
                piggy_bank_address,
                staked_rewards_address,
            } => {
                let mut data = Vec::with_capacity(65);
                data.push(0);
                data.extend_from_slice(piggy_bank_address.as_ref());
                data.extend_from_slice(staked_rewards_address.as_ref());
                data
            }
            PaladinRewardsInstruction::DistributeRewards => vec![1],
            PaladinRewardsInstruction::InitializeHolderRewardInfo => vec![2],
            PaladinRewardsInstruction::HarvestRewards => vec![3],
        }
    }

    /// Unpacks a byte buffer into a
    /// [PaladinRewardsInstruction](enum.PaladinRewardsInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        match input.split_first() {
            Some((&0, rest)) if rest.len() == 64 => {
                Ok(PaladinRewardsInstruction::InitializeMintRewardInfo {
                    piggy_bank_address: *bytemuck::from_bytes(&rest[0..32]),
                    staked_rewards_address: *bytemuck::from_bytes(&rest[32..64]),
                })
            }
            Some((&1, _)) => Ok(PaladinRewardsInstruction::DistributeRewards),
            Some((&2, _)) => Ok(PaladinRewardsInstruction::InitializeHolderRewardInfo),
            Some((&3, _)) => Ok(PaladinRewardsInstruction::HarvestRewards),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

/// Creates an [InitializeMintRewardInfo](enum.PaladinRewardsInstruction.html)
/// instruction.
pub fn initialize_mint_reward_info(
    distribution_account: &Pubkey,
    transfer_hook_extra_account_metas_account: &Pubkey,
    token_mint: &Pubkey,
    mint_authority: &Pubkey,
    piggy_bank_address: &Pubkey,
    staked_rewards_address: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*distribution_account, false),
        AccountMeta::new(*transfer_hook_extra_account_metas_account, false),
        AccountMeta::new_readonly(*token_mint, false),
        AccountMeta::new_readonly(*mint_authority, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let data = PaladinRewardsInstruction::InitializeMintRewardInfo {
        piggy_bank_address: *piggy_bank_address,
        staked_rewards_address: *staked_rewards_address,
    }
    .pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a [DistributeRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
pub fn distribute_rewards(
    distribution_account: &Pubkey,
    piggy_bank_account: &Pubkey,
    staked_pal_rewards_account: &Pubkey,
    leader_account: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*distribution_account, false),
        AccountMeta::new(*piggy_bank_account, false),
        AccountMeta::new(*staked_pal_rewards_account, false),
        AccountMeta::new(*leader_account, false),
    ];
    let data = PaladinRewardsInstruction::DistributeRewards.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates an [InitializeHolderRewardInfo](enum.PaladinRewardsInstruction.html)
/// instruction.
pub fn initialize_holder_reward_info(
    holder_rewards_account: &Pubkey,
    pal_token_account: &Pubkey,
    pal_token_mint: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*holder_rewards_account, false),
        AccountMeta::new_readonly(*pal_token_account, false),
        AccountMeta::new_readonly(*pal_token_mint, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let data = PaladinRewardsInstruction::InitializeHolderRewardInfo.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a [HarvestRewards](enum.PaladinRewardsInstruction.html) instruction.
pub fn harvest_rewards(
    distribution_account: &Pubkey,
    holder_rewards_account: &Pubkey,
    pal_token_account: &Pubkey,
    pal_token_mint: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*distribution_account, false),
        AccountMeta::new(*holder_rewards_account, false),
        AccountMeta::new_readonly(*pal_token_account, false),
        AccountMeta::new_readonly(*pal_token_mint, false),
    ];
    let data = PaladinRewardsInstruction::HarvestRewards.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack_initialize_mint_reward_info() {
        let piggy_bank_address = Pubkey::new_unique();
        let staked_rewards_address = Pubkey::new_unique();
        let original = PaladinRewardsInstruction::InitializeMintRewardInfo {
            piggy_bank_address,
            staked_rewards_address,
        };
        let packed = original.pack();
        let unpacked = PaladinRewardsInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_distribute_rewards() {
        let original = PaladinRewardsInstruction::DistributeRewards;
        let packed = original.pack();
        let unpacked = PaladinRewardsInstruction::unpack(&packed).unwrap();
        assert_eq!(original, unpacked);
    }

    #[test]
    fn test_pack_unpack_initialize_holder_reward_info() {
        let original = PaladinRewardsInstruction::InitializeHolderRewardInfo;
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
