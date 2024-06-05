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
    /// Configures staker rewards for a mint that has been configured with the
    /// rewards program as a transfer hook program.
    ///
    /// This instruction will:
    ///
    /// - Initialize a staker rewards account.
    /// - Initialize the required accounts for the transfer hook.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w]` Staker rewards account.
    /// 1. `[w]` Transfer hook extra account metas account.
    /// 2. `[ ]` Token mint.
    /// 3. `[ ]` Token mint.
    /// 4. `[s]` Mint authority.
    /// 5. `[ ]` System program.
    InitializeStakerRewards,
    /// Moves SOL rewards to the following parties:
    ///
    /// - 1%  Piggy bank.
    /// - 4%  Staker rewards.
    /// - 5%  Holder rewards.
    /// - 90% Leader who produces the block.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w]` Staker rewards account.
    /// 1. `[w]` Holder rewards account.
    /// 2. `[w]` Piggy bank account.
    /// 3. `[w]` Leader account.
    DistributeRewards,
    /// Initializes a holder rewards account for a token account.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w]` Holder rewards account.
    /// 1. `[ ]` Token account.
    /// 3. `[ ]` System program.
    InitializeHolderRewards,
    /// Moves accrued SOL rewards into the provided PAL token account.
    ///
    /// Accounts expected by this instruction:
    ///
    /// 0. `[w]` Staker rewards account.
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
            PaladinRewardsInstruction::InitializeStakerRewards => vec![0],
            PaladinRewardsInstruction::DistributeRewards => vec![1],
            PaladinRewardsInstruction::InitializeHolderRewards => vec![2],
            PaladinRewardsInstruction::HarvestRewards => vec![3],
        }
    }

    /// Unpacks a byte buffer into a
    /// [PaladinRewardsInstruction](enum.PaladinRewardsInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        match input.first() {
            Some(&0) => Ok(PaladinRewardsInstruction::InitializeStakerRewards),
            Some(&1) => Ok(PaladinRewardsInstruction::DistributeRewards),
            Some(&2) => Ok(PaladinRewardsInstruction::InitializeHolderRewards),
            Some(&3) => Ok(PaladinRewardsInstruction::HarvestRewards),
            _ => Err(ProgramError::InvalidInstructionData),
        }
    }
}

/// Creates an [InitializeStakerRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
pub fn initialize_staker_rewards(
    staker_rewards_address: &Pubkey,
    extra_account_metas_address: &Pubkey,
    piggy_bank_address: &Pubkey,
    mint_address: &Pubkey,
    mint_authority_address: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*staker_rewards_address, false),
        AccountMeta::new(*extra_account_metas_address, false),
        AccountMeta::new_readonly(*piggy_bank_address, false),
        AccountMeta::new_readonly(*mint_address, false),
        AccountMeta::new_readonly(*mint_authority_address, true),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let data = PaladinRewardsInstruction::InitializeStakerRewards.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a [DistributeRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
pub fn distribute_rewards(
    staker_rewards_address: &Pubkey,
    holder_rewards_address: &Pubkey,
    piggy_bank_address: &Pubkey,
    leader_address: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*staker_rewards_address, false),
        AccountMeta::new(*holder_rewards_address, false),
        AccountMeta::new(*piggy_bank_address, false),
        AccountMeta::new(*leader_address, false),
    ];
    let data = PaladinRewardsInstruction::DistributeRewards.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates an [InitializeHolderRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
pub fn initialize_holder_rewards(
    holder_rewards_address: &Pubkey,
    token_account_address: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*holder_rewards_address, false),
        AccountMeta::new_readonly(*token_account_address, false),
        AccountMeta::new_readonly(system_program::id(), false),
    ];
    let data = PaladinRewardsInstruction::InitializeHolderRewards.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

/// Creates a [HarvestRewards](enum.PaladinRewardsInstruction.html) instruction.
pub fn harvest_rewards(
    staker_rewards_address: &Pubkey,
    holder_rewards_address: &Pubkey,
    token_account_address: &Pubkey,
    mint_address: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*staker_rewards_address, false),
        AccountMeta::new(*holder_rewards_address, false),
        AccountMeta::new_readonly(*token_account_address, false),
        AccountMeta::new_readonly(*mint_address, false),
    ];
    let data = PaladinRewardsInstruction::HarvestRewards.pack();
    Instruction::new_with_bytes(crate::id(), &data, accounts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack_initialize_staker_rewards() {
        let original = PaladinRewardsInstruction::InitializeStakerRewards;
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
}
