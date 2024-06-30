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
        extension::{
            transfer_hook::{TransferHook, TransferHookAccount},
            BaseStateWithExtensions, StateWithExtensions,
        },
        state::{Account, Mint},
    },
    spl_transfer_hook_interface::{
        collect_extra_account_metas_signer_seeds,
        error::TransferHookError,
        get_extra_account_metas_address_and_bump_seed,
        instruction::{ExecuteInstruction, TransferHookInstruction},
    },
};

const REWARDS_PER_TOKEN_SCALING_FACTOR: u128 = 1_000_000_000; // 1e9

fn get_token_supply(mint_info: &AccountInfo) -> Result<u64, ProgramError> {
    let mint_data = mint_info.try_borrow_data()?;
    let mint = StateWithExtensions::<Mint>::unpack(&mint_data)?;
    Ok(mint.base.supply)
}

fn get_token_account_balance_checked(
    mint: &Pubkey,
    token_account_info: &AccountInfo,
    check_is_transferring: bool,
) -> Result<u64, ProgramError> {
    let token_account_data = token_account_info.try_borrow_data()?;
    let token_account = StateWithExtensions::<Account>::unpack(&token_account_data)?;

    // Ensure the provided token account is for the mint.
    if !token_account.base.mint.eq(mint) {
        return Err(PaladinRewardsError::TokenAccountMintMismatch.into());
    }

    if check_is_transferring {
        // Ensure the provided token account is transferring.
        let extension = token_account.get_extension::<TransferHookAccount>()?;
        if !bool::from(extension.transferring) {
            return Err(TransferHookError::ProgramCalledOutsideOfTransfer.into());
        }
    }

    Ok(token_account.base.amount)
}

fn check_pool(
    program_id: &Pubkey,
    mint: &Pubkey,
    holder_rewards_pool_info: &AccountInfo,
) -> ProgramResult {
    // Ensure the holder rewards pool is owned by the Paladin Rewards
    // program.
    if !holder_rewards_pool_info.owner.eq(program_id) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Ensure the provided holder rewards pool address is the correct
    // address derived from the mint.
    if !holder_rewards_pool_info
        .key
        .eq(&get_holder_rewards_pool_address(mint, program_id))
    {
        return Err(PaladinRewardsError::IncorrectHolderRewardsPoolAddress.into());
    }

    Ok(())
}

fn check_holder_rewards(
    program_id: &Pubkey,
    token_account_key: &Pubkey,
    holder_rewards_info: &AccountInfo,
) -> ProgramResult {
    // Ensure the holder rewards account is owned by the Paladin Rewards
    // program.
    if !holder_rewards_info.owner.eq(program_id) {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Ensure the provided holder rewards address is the correct address
    // derived from the token account.
    if !holder_rewards_info
        .key
        .eq(&get_holder_rewards_address(token_account_key, program_id))
    {
        return Err(PaladinRewardsError::IncorrectHolderRewardsAddress.into());
    }

    Ok(())
}

fn calculate_rewards_per_token(rewards: u64, token_supply: u64) -> Result<u128, ProgramError> {
    if token_supply == 0 {
        return Ok(0);
    }
    // Calculation: rewards / token_supply
    //
    // Scaled by 1e9 to store 9 decimal places of precision.
    (rewards as u128)
        .checked_mul(REWARDS_PER_TOKEN_SCALING_FACTOR)
        .and_then(|product| product.checked_div(token_supply as u128))
        .ok_or(ProgramError::ArithmeticOverflow)
}

fn calculate_eligible_rewards(
    current_accumulated_rewards_per_token: u128,
    last_accumulated_rewards_per_token: u128,
    token_account_balance: u64,
) -> Result<u64, ProgramError> {
    // Calculation: (current_accumulated_rewards_per_token
    //   - last_accumulated_rewards_per_token) * token_account_balance
    let marginal_rate = current_accumulated_rewards_per_token
        .checked_sub(last_accumulated_rewards_per_token)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    if marginal_rate == 0 {
        return Ok(0);
    }
    // Scaled by 1e9 to store 9 decimal places of precision.
    marginal_rate
        .checked_mul(token_account_balance as u128)
        .and_then(|product| product.checked_div(REWARDS_PER_TOKEN_SCALING_FACTOR))
        .and_then(|product| product.try_into().ok())
        .ok_or(ProgramError::ArithmeticOverflow)
}

fn update_holder_rewards_for_transfer_hook(
    program_id: &Pubkey,
    mint: &Pubkey,
    token_account_info: &AccountInfo,
    holder_rewards_info: &AccountInfo,
    current_accumulated_rewards_per_token: u128,
    adjust_token_balance_fn: impl FnOnce(u64) -> Result<u64, ProgramError>,
) -> ProgramResult {
    // Calculate the token account's updated share of the pool rewards.
    //
    // Since the holder rewards account may already have unharvested
    // rewards, calculate the share of rewards that have not been seen
    // by the holder rewards account.
    //
    // Then, adjust the unharvested rewards with the additional share.
    //
    // Token account balances are updated before transfer hooks are called,
    // so this token account balance is the balance _after_ the transfer.
    // See: https://github.com/solana-labs/solana-program-library/blob/3c60545668eafa2294365e2edfb5799c657971c3/token/program-2022/src/processor.rs#L479-L487.
    check_holder_rewards(program_id, token_account_info.key, holder_rewards_info)?;
    let mut holder_rewards_data = holder_rewards_info.try_borrow_mut_data()?;
    let holder_rewards_state =
        bytemuck::try_from_bytes_mut::<HolderRewards>(&mut holder_rewards_data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

    let token_account_balance = {
        // At this point, the token account balance is the balance _after_ the
        // transfer.
        //
        // For the source - since it was just debited - the transfer amount
        // will be added back to calculate the rewards share before the
        // transfer.
        //
        // For the destination - since it was just credited - the transfer
        // amount will be subtracted to calculate the rewards share before
        // the transfer.
        let current_balance = get_token_account_balance_checked(mint, token_account_info, true)?;
        adjust_token_balance_fn(current_balance)?
    };

    let eligible_rewards = calculate_eligible_rewards(
        current_accumulated_rewards_per_token,
        holder_rewards_state.last_accumulated_rewards_per_token,
        token_account_balance,
    )?;

    // Update the holder rewards state.
    holder_rewards_state.last_accumulated_rewards_per_token = current_accumulated_rewards_per_token;
    holder_rewards_state.unharvested_rewards = holder_rewards_state
        .unharvested_rewards
        .checked_add(eligible_rewards)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    Ok(())
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
            get_holder_rewards_pool_address_and_bump_seed(mint_info.key, program_id);
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
    let mint_info = next_account_info(accounts_iter)?;
    let _system_program_info = next_account_info(accounts_iter)?;

    // Ensure the payer account is a signer.
    if !payer_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let token_supply = get_token_supply(mint_info)?;

    check_pool(program_id, mint_info.key, holder_rewards_pool_info)?;

    // Update the total rewards in the holder rewards pool.
    {
        let mut pool_data = holder_rewards_pool_info.try_borrow_mut_data()?;
        let pool_state = bytemuck::try_from_bytes_mut::<HolderRewardsPool>(&mut pool_data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

        // Calculate the new rewards per token by first calculating the rewards
        // per token on the provided rewards amount, then adding that rate to
        // the old rate.
        let marginal_rate = calculate_rewards_per_token(amount, token_supply)?;
        let new_accumulated_rewards_per_token = pool_state
            .accumulated_rewards_per_token
            .checked_add(marginal_rate)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        pool_state.accumulated_rewards_per_token = new_accumulated_rewards_per_token;
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

    // Run checks on the token account.
    {
        let token_account_data = token_account_info.try_borrow_data()?;
        let token_account = StateWithExtensions::<Account>::unpack(&token_account_data)?;

        // Ensure the provided token account is for the mint.
        if !token_account.base.mint.eq(mint_info.key) {
            return Err(PaladinRewardsError::TokenAccountMintMismatch.into());
        }
    }

    check_pool(program_id, mint_info.key, holder_rewards_pool_info)?;
    let pool_data = holder_rewards_pool_info.try_borrow_data()?;
    let pool_state = bytemuck::try_from_bytes::<HolderRewardsPool>(&pool_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Initialize the holder rewards account.
    {
        let (holder_rewards_address, bump_seed) =
            get_holder_rewards_address_and_bump_seed(token_account_info.key, program_id);
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
            HolderRewards::new(pool_state.accumulated_rewards_per_token, 0);
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

    // Run checks on the token account.
    let token_account_balance =
        get_token_account_balance_checked(mint_info.key, token_account_info, false)?;

    check_pool(program_id, mint_info.key, holder_rewards_pool_info)?;
    let pool_data = holder_rewards_pool_info.try_borrow_data()?;
    let pool_state = bytemuck::try_from_bytes::<HolderRewardsPool>(&pool_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    check_holder_rewards(program_id, token_account_info.key, holder_rewards_info)?;
    let mut holder_rewards_data = holder_rewards_info.try_borrow_mut_data()?;
    let holder_rewards_state =
        bytemuck::try_from_bytes_mut::<HolderRewards>(&mut holder_rewards_data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

    // Determine the amount the holder can harvest.
    //
    // This is done by subtracting the `last_accumulated_rewards_per_token`
    // rate from the pool's current rate, then multiplying by the token account
    // balance.
    //
    // The holder should also be able to harvest any unharvested rewards.
    let rewards_to_harvest = {
        // Calculate the eligible rewards from the marginal rate.
        let eligible_rewards = calculate_eligible_rewards(
            pool_state.accumulated_rewards_per_token,
            holder_rewards_state.last_accumulated_rewards_per_token,
            token_account_balance,
        )?;

        // Update the holder rewards state.
        //
        // Temporarily update `unharvested_rewards` with the eligible rewards.
        holder_rewards_state.last_accumulated_rewards_per_token =
            pool_state.accumulated_rewards_per_token;
        holder_rewards_state.unharvested_rewards = holder_rewards_state
            .unharvested_rewards
            .checked_add(eligible_rewards)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        // If the pool doesn't have enough lamports to cover the rewards, only
        // harvest the available lamports. This should never happen, but the check
        // is a failsafe.
        let pool_excess_lamports = {
            let rent = <Rent as Sysvar>::get()?;
            let rent_exempt_lamports =
                rent.minimum_balance(std::mem::size_of::<HolderRewardsPool>());
            holder_rewards_pool_info
                .lamports()
                .saturating_sub(rent_exempt_lamports)
        };

        holder_rewards_state
            .unharvested_rewards
            .min(pool_excess_lamports)
    };

    if rewards_to_harvest != 0 {
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

        // Update the holder's unharvested rewards.
        holder_rewards_state.unharvested_rewards = holder_rewards_state
            .unharvested_rewards
            .checked_sub(rewards_to_harvest)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    }

    Ok(())
}

/// Processes an SPL Transfer Hook Interface
/// [ExecuteInstruction](https://docs.rs/spl-transfer-hook-interface/latest/spl_transfer_hook_interface/instruction/struct.ExecuteInstruction.html).
pub fn process_spl_transfer_hook_execute(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    transfer_amount: u64,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let source_token_account_info = next_account_info(accounts_iter)?;
    let mint_info = next_account_info(accounts_iter)?;
    let destination_token_account_info = next_account_info(accounts_iter)?;
    let _source_owner_info = next_account_info(accounts_iter)?;
    let _extra_metas_info = next_account_info(accounts_iter)?;
    let holder_rewards_pool_info = next_account_info(accounts_iter)?;
    let source_holder_rewards_info = next_account_info(accounts_iter)?;
    let destination_holder_rewards_info = next_account_info(accounts_iter)?;

    let current_accumulated_rewards_per_token = {
        check_pool(program_id, mint_info.key, holder_rewards_pool_info)?;
        let pool_data = holder_rewards_pool_info.try_borrow_data()?;
        let pool_state = bytemuck::try_from_bytes::<HolderRewardsPool>(&pool_data)
            .map_err(|_| ProgramError::InvalidAccountData)?;
        pool_state.accumulated_rewards_per_token
    };

    // Update the source holder rewards account.
    //
    // For the source - since it was just debited - the transfer amount
    // will be added back to calculate the rewards share before the
    // transfer.
    update_holder_rewards_for_transfer_hook(
        program_id,
        mint_info.key,
        source_token_account_info,
        source_holder_rewards_info,
        current_accumulated_rewards_per_token,
        |amount| {
            amount
                .checked_add(transfer_amount)
                .ok_or(ProgramError::ArithmeticOverflow)
        },
    )?;

    // Update the destination holder rewards account.
    //
    // For the destination - since it was just credited - the transfer
    // amount will be subtracted to calculate the rewards share before
    // the transfer.
    update_holder_rewards_for_transfer_hook(
        program_id,
        mint_info.key,
        destination_token_account_info,
        destination_holder_rewards_info,
        current_accumulated_rewards_per_token,
        |amount| {
            amount
                .checked_sub(transfer_amount)
                .ok_or(ProgramError::ArithmeticOverflow)
        },
    )?;

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
