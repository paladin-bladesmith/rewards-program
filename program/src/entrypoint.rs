//! Program entrypoint.

use {
    crate::{error::PaladinRewardsError, processor},
    solana_program::{
        account_info::AccountInfo, entrypoint::ProgramResult, program_error::PrintProgramError,
        pubkey::Pubkey,
    },
};

solana_program::entrypoint!(process_instruction);

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    input: &[u8],
) -> ProgramResult {
    if let Err(error) = processor::process(program_id, accounts, input) {
        error.print::<PaladinRewardsError>();
        return Err(error);
    }
    Ok(())
}
