//! Program processor.

use {
    crate::instruction::PaladinRewardsInstruction,
    solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey},
};

/// Processes an [InitializeMintRewardInfo](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_initialize_mint_reward_info(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _piggy_bank_address: Pubkey,
    _staked_rewards_address: Pubkey,
) -> ProgramResult {
    Ok(())
}

/// Processes an [SweepActiveRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_sweep_active_rewards(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}

/// Processes a [DistributeRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_distribute_rewards(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}

/// Processes an
/// [InitializeHolderRewardInfo](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_initialize_holder_reward_info(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
) -> ProgramResult {
    Ok(())
}

/// Processes a [HarvestRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_harvest_rewards(_program_id: &Pubkey, _accounts: &[AccountInfo]) -> ProgramResult {
    Ok(())
}

/// Processes a
/// [PaladinRewardsInstruction](enum.PaladinRewardsInstruction.html).
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = PaladinRewardsInstruction::unpack(input)?;
    match instruction {
        PaladinRewardsInstruction::InitializeMintRewardInfo {
            piggy_bank_address,
            staked_rewards_address,
        } => {
            msg!("Instruction: InitializeMintRewardInfo");
            process_initialize_mint_reward_info(
                program_id,
                accounts,
                piggy_bank_address,
                staked_rewards_address,
            )
        }
        PaladinRewardsInstruction::SweepActiveRewards => {
            msg!("Instruction: SweepActiveRewards");
            process_sweep_active_rewards(program_id, accounts)
        }
        PaladinRewardsInstruction::DistributeRewards => {
            msg!("Instruction: DistributeRewards");
            process_distribute_rewards(program_id, accounts)
        }
        PaladinRewardsInstruction::InitializeHolderRewardInfo => {
            msg!("Instruction: InitializeHolderRewardInfo");
            process_initialize_holder_reward_info(program_id, accounts)
        }
        PaladinRewardsInstruction::HarvestRewards => {
            msg!("Instruction: HarvestRewards");
            process_harvest_rewards(program_id, accounts)
        }
    }
}
