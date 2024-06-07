//! Program processor.

use {
    crate::instruction::PaladinRewardsInstruction,
    solana_program::{account_info::AccountInfo, entrypoint::ProgramResult, msg, pubkey::Pubkey},
    spl_transfer_hook_interface::instruction::TransferHookInstruction,
};

/// Processes an
/// [InitializeHolderRewardsPool](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_initialize_holder_rewards_pool(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
) -> ProgramResult {
    Ok(())
}

/// Processes a [DistributeRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_distribute_rewards(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _amount: u64,
) -> ProgramResult {
    Ok(())
}

/// Processes an
/// [InitializeHolderRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_initialize_holder_rewards(
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

/// Processes an SPL Transfer Hook Interface
/// [ExecuteInstruction](https://docs.rs/spl-transfer-hook-interface/latest/spl_transfer_hook_interface/instruction/struct.ExecuteInstruction.html).
pub fn process_spl_transfer_hook_execute(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _amount: u64,
) -> ProgramResult {
    Ok(())
}

/// Processes a
/// [PaladinRewardsInstruction](enum.PaladinRewardsInstruction.html).
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    if let Ok(TransferHookInstruction::Execute { amount }) = TransferHookInstruction::unpack(input)
    {
        process_spl_transfer_hook_execute(program_id, accounts, amount)
    } else {
        let instruction = PaladinRewardsInstruction::unpack(input)?;
        match instruction {
            PaladinRewardsInstruction::InitializeHolderRewardsPool => {
                msg!("Instruction: InitializeHolderRewardsPool");
                process_initialize_holder_rewards_pool(program_id, accounts)
            }
            PaladinRewardsInstruction::DistributeRewards(amount) => {
                msg!("Instruction: DistributeRewards");
                process_distribute_rewards(program_id, accounts, amount)
            }
            PaladinRewardsInstruction::InitializeHolderRewards => {
                msg!("Instruction: InitializeHolderRewards");
                process_initialize_holder_rewards(program_id, accounts)
            }
            PaladinRewardsInstruction::HarvestRewards => {
                msg!("Instruction: HarvestRewards");
                process_harvest_rewards(program_id, accounts)
            }
        }
    }
}
