//! Program processor.

use {
    crate::{
        error::PaladinRewardsError,
        extra_metas::get_extra_account_metas,
        instruction::PaladinRewardsInstruction,
        state::{
            collect_holder_rewards_pool_signer_seeds, collect_holder_rewards_signer_seeds,
            get_holder_rewards_address, get_holder_rewards_address_and_bump_seed,
            get_holder_rewards_pool_address, get_holder_rewards_pool_address_and_bump_seed,
            HolderRewards, HolderRewardsPool,
        },
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        program_option::COption,
        pubkey::Pubkey,
        rent::Rent,
        system_instruction,
        sysvar::Sysvar,
    },
    spl_tlv_account_resolution::state::ExtraAccountMetaList,
    spl_token_2022::{
        extension::{transfer_hook::TransferHook, BaseStateWithExtensions, StateWithExtensions},
        state::{Account, Mint},
    },
    spl_transfer_hook_interface::{
        collect_extra_account_metas_signer_seeds, get_extra_account_metas_address_and_bump_seed,
        instruction::{ExecuteInstruction, TransferHookInstruction},
    },
};

fn calculate_reward_share(
    token_supply: u64,
    token_account_balance: u64,
    total_rewards: u64,
) -> Result<u64, ProgramError> {
    if token_supply == 0 {
        return Ok(0);
    }
    // Calculation: (token_amount / total_token_supply) * pool_rewards
    //
    // However, multiplication is done first to avoid truncation, ie:
    // (token_amount * pool_rewards) / total_token_supply
    (token_account_balance as u128)
        .checked_mul(total_rewards as u128)
        .and_then(|product| product.checked_div(token_supply as u128))
        .and_then(|share| u64::try_from(share).ok())
        .ok_or(ProgramError::ArithmeticOverflow)
}

/// Processes an
/// [InitializeHolderRewardsPool](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_initialize_holder_rewards_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let holder_rewards_pool_info = next_account_info(accounts_iter)?;
    let extra_metas_info = next_account_info(accounts_iter)?;
    let mint_info = next_account_info(accounts_iter)?;
    let mint_authority_info = next_account_info(accounts_iter)?;
    let _system_program_info = next_account_info(accounts_iter)?;

    // Run checks on the mint.
    {
        let mint_data = mint_info.try_borrow_data()?;
        let mint = StateWithExtensions::<Mint>::unpack(&mint_data)?;

        // Ensure the mint is configured with the `TransferHook` extension,
        // and the program ID is the Paladin Rewards program.
        let transfer_hook = mint.get_extension::<TransferHook>()?;
        let hook_program_id: Option<Pubkey> = transfer_hook.program_id.into();
        if !hook_program_id.eq(&Some(*program_id)) {
            return Err(PaladinRewardsError::IncorrectTransferHookProgramId.into());
        }

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
    }

    // Initialize the holder rewards pool account.
    {
        let (holder_rewards_pool_address, bump_seed) =
            get_holder_rewards_pool_address_and_bump_seed(mint_info.key);
        let bump_seed = [bump_seed];
        let holder_rewards_pool_signer_seeds =
            collect_holder_rewards_pool_signer_seeds(mint_info.key, &bump_seed);

        // Ensure the provided holder rewards pool address is the correct
        // address derived from the mint.
        if !holder_rewards_pool_info
            .key
            .eq(&holder_rewards_pool_address)
        {
            return Err(PaladinRewardsError::IncorrectHolderRewardsPoolAddress.into());
        }

        // Ensure the holder rewards pool account has not already been
        // initialized.
        if holder_rewards_pool_info.data_len() != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Allocate & assign.
        invoke_signed(
            &system_instruction::allocate(
                &holder_rewards_pool_address,
                std::mem::size_of::<HolderRewardsPool>() as u64,
            ),
            &[holder_rewards_pool_info.clone()],
            &[&holder_rewards_pool_signer_seeds],
        )?;
        invoke_signed(
            &system_instruction::assign(&holder_rewards_pool_address, program_id),
            &[holder_rewards_pool_info.clone()],
            &[&holder_rewards_pool_signer_seeds],
        )?;

        // Write the data.
        let mut data = holder_rewards_pool_info.try_borrow_mut_data()?;
        *bytemuck::try_from_bytes_mut(&mut data).map_err(|_| ProgramError::InvalidAccountData)? =
            HolderRewardsPool::default();
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
            return Err(PaladinRewardsError::IncorrectExtraMetasAddress.into());
        }

        // Ensure the extra metas account has not already been initialized.
        if extra_metas_info.data_len() != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        let extra_metas = get_extra_account_metas();
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
        ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &extra_metas)?;
    }

    Ok(())
}

/// Processes a [DistributeRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_distribute_rewards(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let payer_info = next_account_info(accounts_iter)?;
    let holder_rewards_pool_info = next_account_info(accounts_iter)?;
    let _system_program_info = next_account_info(accounts_iter)?;

    // Ensure the payer account is a signer.
    if !payer_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Ensure the holder rewards pool account is owned by the Paladin Rewards
    // program.
    if !holder_rewards_pool_info.owner.eq(program_id) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Update the total rewards in the holder rewards pool.
    {
        let mut holder_rewards_pool_data = holder_rewards_pool_info.try_borrow_mut_data()?;
        let holder_rewards_pool_state =
            bytemuck::try_from_bytes_mut::<HolderRewardsPool>(&mut holder_rewards_pool_data)
                .map_err(|_| ProgramError::InvalidAccountData)?;

        holder_rewards_pool_state.total_rewards = holder_rewards_pool_state
            .total_rewards
            .checked_add(amount)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    }

    // Move the amount from the payer to the holder rewards pool.
    invoke(
        &system_instruction::transfer(payer_info.key, holder_rewards_pool_info.key, amount),
        &[payer_info.clone(), holder_rewards_pool_info.clone()],
    )?;

    Ok(())
}

/// Processes an
/// [InitializeHolderRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_initialize_holder_rewards(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let holder_rewards_pool_info = next_account_info(accounts_iter)?;
    let holder_rewards_info = next_account_info(accounts_iter)?;
    let token_account_info = next_account_info(accounts_iter)?;
    let mint_info = next_account_info(accounts_iter)?;
    let _system_program = next_account_info(accounts_iter)?;

    let token_supply = {
        let mint_data = mint_info.try_borrow_data()?;
        let mint = StateWithExtensions::<Mint>::unpack(&mint_data)?;
        mint.base.supply
    };

    let token_account_balance = {
        let token_account_data = token_account_info.try_borrow_data()?;
        let token_account = StateWithExtensions::<Account>::unpack(&token_account_data)?;

        // Ensure the provided token account is for the mint.
        if !token_account.base.mint.eq(mint_info.key) {
            return Err(PaladinRewardsError::TokenAccountMintMismatch.into());
        }

        token_account.base.amount
    };

    let last_seen_total_rewards = {
        // Ensure the holder rewards pool is owned by the Paladin Rewards
        // program.
        if !holder_rewards_pool_info.owner.eq(program_id) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Ensure the provided holder rewards pool address is the correct
        // address derived from the mint.
        if !holder_rewards_pool_info
            .key
            .eq(&get_holder_rewards_pool_address(mint_info.key))
        {
            return Err(PaladinRewardsError::IncorrectHolderRewardsPoolAddress.into());
        }

        let holder_rewards_pool_data = holder_rewards_pool_info.try_borrow_data()?;
        let holder_rewards_pool_state =
            bytemuck::try_from_bytes::<HolderRewardsPool>(&holder_rewards_pool_data)
                .map_err(|_| ProgramError::InvalidAccountData)?;

        holder_rewards_pool_state.total_rewards
    };

    // Calculate unharvested rewards for the token account.
    //
    // Since the holder rewards account is being initialized, the
    // `unharvested_rewards` is calculated from the _available_ rewards in the
    // pool, ie. `pool.lamports - rent_exempt_minimum`.
    //
    // If the program used total rewards for this calculation, new holders
    // would be able to claim rewards that were already distributed to other
    // holders.
    //
    // If the program used zero rewards for this calculation, new holders
    // would not be able to claim rewards until the next distribution, which
    // could result in some lamports left unclaimable in the pool.
    let unharvested_rewards = {
        let rent = <Rent as Sysvar>::get()?;
        let rent_exempt_lamports = rent.minimum_balance(std::mem::size_of::<HolderRewardsPool>());
        let available_rewards = holder_rewards_pool_info
            .lamports()
            .saturating_sub(rent_exempt_lamports);
        calculate_reward_share(token_supply, token_account_balance, available_rewards)?
    };

    // Initialize the holder rewards account.
    {
        let (holder_rewards_address, bump_seed) =
            get_holder_rewards_address_and_bump_seed(token_account_info.key);
        let bump_seed = [bump_seed];
        let holder_rewards_signer_seeds =
            collect_holder_rewards_signer_seeds(token_account_info.key, &bump_seed);

        // Ensure the provided holder rewards address is the correct address
        // derived from the token account.
        if !holder_rewards_info.key.eq(&holder_rewards_address) {
            return Err(PaladinRewardsError::IncorrectHolderRewardsAddress.into());
        }

        // Ensure the holder rewards account has not already been initialized.
        if holder_rewards_info.data.borrow().len() != 0 {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        // Allocate & assign.
        invoke_signed(
            &system_instruction::allocate(
                &holder_rewards_address,
                std::mem::size_of::<HolderRewards>() as u64,
            ),
            &[holder_rewards_info.clone()],
            &[&holder_rewards_signer_seeds],
        )?;
        invoke_signed(
            &system_instruction::assign(&holder_rewards_address, program_id),
            &[holder_rewards_info.clone()],
            &[&holder_rewards_signer_seeds],
        )?;

        // Write the data.
        let mut data = holder_rewards_info.try_borrow_mut_data()?;
        *bytemuck::try_from_bytes_mut(&mut data).map_err(|_| ProgramError::InvalidAccountData)? =
            HolderRewards {
                last_seen_total_rewards,
                unharvested_rewards,
            };
    }

    Ok(())
}

/// Processes a [HarvestRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_harvest_rewards(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let holder_rewards_pool_info = next_account_info(accounts_iter)?;
    let holder_rewards_info = next_account_info(accounts_iter)?;
    let token_account_info = next_account_info(accounts_iter)?;
    let mint_info = next_account_info(accounts_iter)?;

    let token_supply = {
        let mint_data = mint_info.try_borrow_data()?;
        let mint = StateWithExtensions::<Mint>::unpack(&mint_data)?;
        mint.base.supply
    };

    let token_account_balance = {
        let token_account_data = token_account_info.try_borrow_data()?;
        let token_account = StateWithExtensions::<Account>::unpack(&token_account_data)?;

        // Ensure the provided token account is for the mint.
        if !token_account.base.mint.eq(mint_info.key) {
            return Err(PaladinRewardsError::TokenAccountMintMismatch.into());
        }

        token_account.base.amount
    };

    let current_total_rewards = {
        // Ensure the holder rewards pool is owned by the Paladin Rewards
        // program.
        if !holder_rewards_pool_info.owner.eq(program_id) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Ensure the provided holder rewards pool address is the correct
        // address derived from the mint.
        if !holder_rewards_pool_info
            .key
            .eq(&get_holder_rewards_pool_address(mint_info.key))
        {
            return Err(PaladinRewardsError::IncorrectHolderRewardsPoolAddress.into());
        }

        let holder_rewards_pool_data = holder_rewards_pool_info.try_borrow_data()?;
        let holder_rewards_pool_state =
            bytemuck::try_from_bytes::<HolderRewardsPool>(&holder_rewards_pool_data)
                .map_err(|_| ProgramError::InvalidAccountData)?;

        holder_rewards_pool_state.total_rewards
    };

    let (last_seen_total_rewards, unharvested_rewards) = {
        // Ensure the holder rewards account is owned by the Paladin Rewards
        // program.
        if !holder_rewards_info.owner.eq(program_id) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // Ensure the provided holder rewards address is the correct address
        // derived from the token account.
        if !holder_rewards_info
            .key
            .eq(&get_holder_rewards_address(token_account_info.key))
        {
            return Err(PaladinRewardsError::IncorrectHolderRewardsAddress.into());
        }

        let mut holder_rewards_data = holder_rewards_info.try_borrow_mut_data()?;
        let holder_rewards_state =
            bytemuck::try_from_bytes_mut::<HolderRewards>(&mut holder_rewards_data)
                .map_err(|_| ProgramError::InvalidAccountData)?;

        // Update the holder rewards state with the new "last seen" total
        // rewards and zero out the unharvested rewards.
        (
            std::mem::replace(
                &mut holder_rewards_state.last_seen_total_rewards,
                current_total_rewards,
            ),
            std::mem::take(&mut holder_rewards_state.unharvested_rewards),
        )
    };

    // Calculate unharvested rewards for the token account.
    // Since the holder rewards account may already have unharvested rewards,
    // calculate the share of rewards that have not been seen by the holder
    // rewards account.
    let unseen_total_rewards = current_total_rewards
        .checked_sub(last_seen_total_rewards)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let unseen_unharvested_rewards =
        calculate_reward_share(token_supply, token_account_balance, unseen_total_rewards)?;
    let rewards_to_harvest = unharvested_rewards
        .checked_add(unseen_unharvested_rewards)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Move the amount from the holder rewards pool to the token account.
    let new_holder_rewards_pool_lamports = holder_rewards_pool_info
        .lamports()
        .checked_sub(rewards_to_harvest)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let new_token_account_lamports = token_account_info
        .lamports()
        .checked_add(rewards_to_harvest)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    **holder_rewards_pool_info.try_borrow_mut_lamports()? = new_holder_rewards_pool_lamports;
    **token_account_info.try_borrow_mut_lamports()? = new_token_account_lamports;

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
