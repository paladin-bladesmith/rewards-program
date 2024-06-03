//! Program processor.

use {
    crate::{
        error::PaladinRewardsError,
        instruction::PaladinRewardsInstruction,
        state::{
            get_mint_rewards_address_and_bump_seed, MintRewards, SEED_PREFIX_HOLDER_REWARDS,
            SEED_PREFIX_MINT_REWARDS,
        },
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::invoke_signed,
        program_error::ProgramError,
        program_option::COption,
        pubkey::Pubkey,
        system_instruction,
    },
    spl_tlv_account_resolution::{
        account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
    },
    spl_token_2022::{
        extension::{transfer_hook::TransferHook, BaseStateWithExtensions, StateWithExtensions},
        state::Mint,
    },
    spl_transfer_hook_interface::{
        collect_extra_account_metas_signer_seeds, get_extra_account_metas_address_and_bump_seed,
        instruction::ExecuteInstruction,
    },
};

/// Processes an [InitializeMintRewardInfo](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_initialize_mint_reward_info(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    piggy_bank_address: Pubkey,
    staked_rewards_address: Pubkey,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let distribution_info = next_account_info(accounts_iter)?;
    let extra_metas_info = next_account_info(accounts_iter)?;
    let mint_info = next_account_info(accounts_iter)?;
    let mint_authority_info = next_account_info(accounts_iter)?;
    let _system_program_info = next_account_info(accounts_iter)?;

    // Run checks on the mint.
    {
        let mint_data = mint_info.try_borrow_data()?;
        let mint = StateWithExtensions::<Mint>::unpack(&mint_data)?;

        // Ensure the provided mint authority is the correct mint authority.
        if !mint
            .base
            .mint_authority
            .eq(&COption::Some(*mint_authority_info.key))
        {
            return Err(PaladinRewardsError::IncorrectMintAuthority.into());
        }

        // Ensure the mint authority is a signer.
        if !mint_authority_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // Ensure the mint is configured with the `TransferHook` extension,
        // and the program ID is the Paladin Rewards program.
        let transfer_hook = mint.get_extension::<TransferHook>()?;
        let hook_program_id: Option<Pubkey> = transfer_hook.program_id.into();
        if !hook_program_id.eq(&Some(*program_id)) {
            return Err(PaladinRewardsError::IncorrectTransferHookProgramId.into());
        }
    }

    // Initialize the distribution account.
    {
        let (distribution_address, distribution_bump) =
            get_mint_rewards_address_and_bump_seed(mint_info.key);
        let distribution_signer_seeds = &[
            SEED_PREFIX_MINT_REWARDS,
            mint_info.key.as_ref(),
            &[distribution_bump],
        ];

        // Ensure the provided distribution address is the correct address
        // derived from the mint.
        if !distribution_info.key.eq(&distribution_address) {
            return Err(PaladinRewardsError::IncorrectDistributionAccountAddress.into());
        }

        // Ensure the distribution account has not already been initialized.
        if distribution_info.data.borrow().len() != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Allocate & assign.
        invoke_signed(
            &system_instruction::allocate(
                &distribution_address,
                std::mem::size_of::<MintRewards>() as u64,
            ),
            &[distribution_info.clone()],
            &[distribution_signer_seeds],
        )?;
        invoke_signed(
            &system_instruction::assign(&distribution_address, program_id),
            &[distribution_info.clone()],
            &[distribution_signer_seeds],
        )?;

        // Write the data.
        let distribution_state = MintRewards::new(&piggy_bank_address, &staked_rewards_address);
        let distribution_data = bytemuck::bytes_of(&distribution_state);
        distribution_info
            .try_borrow_mut_data()?
            .copy_from_slice(distribution_data);
    }

    // Initialize the extra metas account.
    {
        let (extra_metas_address, extra_metas_bump) =
            get_extra_account_metas_address_and_bump_seed(mint_info.key, program_id);
        let extra_metas_bump = [extra_metas_bump];
        let extra_metas_signer_seeds =
            collect_extra_account_metas_signer_seeds(mint_info.key, &extra_metas_bump);

        // Ensure the provided extra metas address is the correct address
        // derived from the mint.
        if !extra_metas_info.key.eq(&extra_metas_address) {
            return Err(PaladinRewardsError::IncorrectExtraMetasAccountAddress.into());
        }

        // Ensure the extra metas account has not already been initialized.
        if extra_metas_info.data.borrow().len() != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Create the list of extra account metas.
        let extra_metas = &[
            // 5: Token-2022 program.
            ExtraAccountMeta::new_with_pubkey(&spl_token_2022::id(), false, false)?,
            // 6: Associated Token program.
            ExtraAccountMeta::new_with_pubkey(&spl_associated_token_account::id(), false, false)?,
            // 7: Holder token account.
            ExtraAccountMeta::new_external_pda_with_seeds(
                6, // Associated Token program.
                &[
                    Seed::AccountKey {
                        index: 3, // Source owner.
                    },
                    Seed::AccountKey {
                        index: 5, // Token-2022 program.
                    },
                    Seed::AccountKey {
                        index: 1, // Mint.
                    },
                ],
                false,
                false,
            )?,
            // 8: Holder rewards.
            ExtraAccountMeta::new_with_seeds(
                &[
                    Seed::Literal {
                        bytes: SEED_PREFIX_HOLDER_REWARDS.to_vec(),
                    },
                    Seed::AccountKey {
                        index: 7, // Holder token account.
                    },
                ],
                false,
                true,
            )?,
            // 9: Distribution account.
            ExtraAccountMeta::new_with_pubkey(distribution_info.key, false, true)?,
        ];
        let account_size = ExtraAccountMetaList::size_of(extra_metas.len())?;

        // Allocate & assign.
        invoke_signed(
            &system_instruction::allocate(&extra_metas_address, account_size as u64),
            &[extra_metas_info.clone()],
            &[&extra_metas_signer_seeds],
        )?;
        invoke_signed(
            &system_instruction::assign(&extra_metas_address, program_id),
            &[extra_metas_info.clone()],
            &[&extra_metas_signer_seeds],
        )?;

        // Write the data.
        let mut data = extra_metas_info.try_borrow_mut_data()?;
        ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, extra_metas)?;
    }

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
