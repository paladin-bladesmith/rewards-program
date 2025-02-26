//! Program processor.

use {
    crate::{
        constants::rent_debt,
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
        program::invoke_signed,
        program_error::ProgramError,
        pubkey::Pubkey,
        rent::Rent,
        system_instruction, system_program,
        sysvar::Sysvar,
    },
    spl_tlv_account_resolution::state::ExtraAccountMetaList,
    spl_token_2022::{
        extension::{
            transfer_hook::{TransferHook, TransferHookAccount},
            BaseStateWithExtensions, ExtensionType, StateWithExtensions,
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

const REWARDS_PER_TOKEN_SCALING_FACTOR: u128 = 1_000_000_000_000_000_000; // 1e18

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
    assert_eq!(token_account_info.owner, &spl_token_2022::ID);
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
    if holder_rewards_pool_info.owner != program_id {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Ensure the provided holder rewards pool address is the correct
    // address derived from the mint.
    if holder_rewards_pool_info.key != &get_holder_rewards_pool_address(mint, program_id) {
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
    if holder_rewards_info.owner != program_id {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Ensure the provided holder rewards address is the correct address
    // derived from the token account.
    if holder_rewards_info.key != &get_holder_rewards_address(token_account_key, program_id) {
        return Err(PaladinRewardsError::IncorrectHolderRewardsAddress.into());
    }

    Ok(())
}

// Calculate the rewards per token.
//
// Calculation: rewards / token_supply
// Scaled by 1e18 to store 18 decimal places of precision.
//
// This calculation is valid for all possible values of rewards and token
// supply, since the scaling to `u128` prevents multiplication from breaking
// the `u64::MAX` ceiling, and the `token_supply == 0` check prevents
// `checked_div` returning `None` from a zero denominator.
fn calculate_rewards_per_token(rewards: u64, token_supply: u64) -> Result<u128, ProgramError> {
    if token_supply == 0 {
        return Ok(0);
    }
    (rewards as u128)
        .checked_mul(REWARDS_PER_TOKEN_SCALING_FACTOR)
        .and_then(|product| product.checked_div(token_supply as u128))
        .ok_or(ProgramError::ArithmeticOverflow)
}

// Calculate the eligible rewards for a token account.
//
// Calculation: (current - last) * balance
// The result is descaled by a factor of 1e18 since both rewards per token
// values are scaled by 1e18 for precision.
//
// This calculation is valid as long as the total rewards accumulated by the
// system does not exceed `u64::MAX`.
//
// This condition is a reasonable upper bound, considering `u64::MAX` is
// approximately 386_266 % of the current circulating supply of SOL.
//
// For more information, see this function's prop tests.
fn calculate_eligible_rewards(
    current_accumulated_rewards_per_token: u128,
    last_accumulated_rewards_per_token: u128,
    token_account_balance: u64,
) -> Result<u64, ProgramError> {
    let marginal_rate =
        current_accumulated_rewards_per_token.wrapping_sub(last_accumulated_rewards_per_token);
    if marginal_rate == 0 {
        return Ok(0);
    }
    marginal_rate
        .checked_mul(token_account_balance as u128)
        .and_then(|product| product.checked_div(REWARDS_PER_TOKEN_SCALING_FACTOR))
        .and_then(|product| product.try_into().ok())
        .ok_or(ProgramError::ArithmeticOverflow)
}

fn update_accumulated_rewards_per_token(
    mint_info: &AccountInfo,
    holder_rewards_pool_info: &AccountInfo,
    pool_state: &mut HolderRewardsPool,
) -> ProgramResult {
    let latest_lamports = holder_rewards_pool_info.lamports();
    let additional_lamports = latest_lamports
        .checked_sub(pool_state.lamports_last)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let marginal_rate =
        calculate_rewards_per_token(additional_lamports, get_token_supply(mint_info)?)?;
    pool_state.accumulated_rewards_per_token = pool_state
        .accumulated_rewards_per_token
        .wrapping_add(marginal_rate);
    pool_state.lamports_last = latest_lamports;

    Ok(())
}

fn update_holder_rewards_for_transfer_hook(
    program_id: &Pubkey,
    mint: &Pubkey,
    token_account_info: &AccountInfo,
    holder_rewards_info: &AccountInfo,
    current_accumulated_rewards_per_token: u128,
    adjust_token_balance_fn: impl FnOnce(u64) -> Result<u64, ProgramError>,
) -> ProgramResult {
    let mut holder_rewards_data = holder_rewards_info.try_borrow_mut_data()?;
    if holder_rewards_data.is_empty() {
        return Ok(());
    }

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

fn assert_rent_exempt(account: &AccountInfo) {
    assert!(account.lamports() >= Rent::get().unwrap().minimum_balance(account.data_len()));
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
    let _system_program_info = next_account_info(accounts_iter)?;

    // Run checks on the mint.
    {
        assert_eq!(mint_info.owner, &spl_token_2022::ID);
        let mint_data = mint_info.try_borrow_data()?;
        let mint = StateWithExtensions::<Mint>::unpack(&mint_data)?;

        // Check only allowed extensions.
        let extensions = mint.get_extension_types()?;
        if !extensions
            .iter()
            .all(|extension| matches!(extension, ExtensionType::TransferHook))
        {
            return Err(PaladinRewardsError::InvalidExtension.into());
        }

        // Ensure the mint is configured with the `TransferHook` extension,
        // and the program ID is the Paladin Rewards program.
        let transfer_hook = mint.get_extension::<TransferHook>()?;
        let hook_program_id: Option<Pubkey> = transfer_hook.program_id.into();
        if hook_program_id != Some(*program_id) {
            return Err(PaladinRewardsError::IncorrectTransferHookProgramId.into());
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
        if holder_rewards_pool_info.key != &holder_rewards_pool_address {
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
        assert_rent_exempt(holder_rewards_pool_info);

        // Write the data.
        let mut data = holder_rewards_pool_info.try_borrow_mut_data()?;
        *bytemuck::try_from_bytes_mut(&mut data).map_err(|_| ProgramError::InvalidAccountData)? =
            HolderRewardsPool {
                accumulated_rewards_per_token: 0,
                lamports_last: holder_rewards_pool_info.lamports(),
                _padding: 0,
            };
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
        if extra_metas_info.key != &extra_metas_address {
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
        assert_rent_exempt(extra_metas_info);

        // Write the data.
        let mut data = extra_metas_info.try_borrow_mut_data()?;
        ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &extra_metas)?;
    }

    Ok(())
}

/// Processes an
/// [InitializeHolderRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_initialize_holder_rewards(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    rent_sponsor: Pubkey,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let holder_rewards_pool_info = next_account_info(accounts_iter)?;
    let holder_rewards_info = next_account_info(accounts_iter)?;
    let token_account_info = next_account_info(accounts_iter)?;
    let mint_info = next_account_info(accounts_iter)?;
    let _system_program = next_account_info(accounts_iter)?;

    // Run checks on the token account.
    let initial_balance =
        get_token_account_balance_checked(mint_info.key, token_account_info, false)?;

    check_pool(program_id, mint_info.key, holder_rewards_pool_info)?;
    let mut pool_data = holder_rewards_pool_info.try_borrow_mut_data()?;
    let pool_state = bytemuck::try_from_bytes_mut::<HolderRewardsPool>(&mut pool_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Process any received lamports.
    update_accumulated_rewards_per_token(mint_info, holder_rewards_pool_info, pool_state)?;

    // Initialize the holder rewards account.
    {
        let (holder_rewards_address, bump_seed) =
            get_holder_rewards_address_and_bump_seed(token_account_info.key, program_id);
        let bump_seed = [bump_seed];
        let holder_rewards_signer_seeds =
            collect_holder_rewards_signer_seeds(token_account_info.key, &bump_seed);

        // Ensure the provided holder rewards address is the correct address
        // derived from the token account.
        if holder_rewards_info.key != &holder_rewards_address {
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
        assert_rent_exempt(holder_rewards_info);

        // Write the data.
        let mut data = holder_rewards_info.try_borrow_mut_data()?;
        *bytemuck::try_from_bytes_mut(&mut data).map_err(|_| ProgramError::InvalidAccountData)? =
            HolderRewards {
                last_accumulated_rewards_per_token: pool_state.accumulated_rewards_per_token,
                unharvested_rewards: 0,
                rent_sponsor,
                rent_debt: match rent_sponsor == Pubkey::default() {
                    true => 0,
                    // NB: Sponsor is paid back at a 10% premium as an incentive to sponsor the
                    // account.
                    #[allow(clippy::arithmetic_side_effects)]
                    false => rent_debt(Rent::get()?.minimum_balance(HolderRewards::LEN)),
                },
                minimum_balance: match rent_sponsor == Pubkey::default() {
                    true => 0,
                    false => initial_balance,
                },
                _padding: 0,
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

    // NB: Checks program owner & mint correctness.
    let token_account_balance =
        get_token_account_balance_checked(mint_info.key, token_account_info, false)?;

    // Check & load the pool
    check_pool(program_id, mint_info.key, holder_rewards_pool_info)?;
    let mut pool_data = holder_rewards_pool_info.try_borrow_mut_data()?;
    let pool_state = bytemuck::try_from_bytes_mut::<HolderRewardsPool>(&mut pool_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Check & load the holder rewards.
    check_holder_rewards(program_id, token_account_info.key, holder_rewards_info)?;
    let mut holder_rewards_data = holder_rewards_info.try_borrow_mut_data()?;
    let holder_rewards_state =
        bytemuck::try_from_bytes_mut::<HolderRewards>(&mut holder_rewards_data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

    // Handle any lamports received since last harvest.
    update_accumulated_rewards_per_token(mint_info, holder_rewards_pool_info, pool_state)?;

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

        if eligible_rewards != 0 {
            // Update the holder rewards state.
            //
            // Temporarily update `unharvested_rewards` with the eligible rewards.
            holder_rewards_state.last_accumulated_rewards_per_token =
                pool_state.accumulated_rewards_per_token;
            holder_rewards_state.unharvested_rewards = holder_rewards_state
                .unharvested_rewards
                .checked_add(eligible_rewards)
                .ok_or(ProgramError::ArithmeticOverflow)?;
        }

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

        std::cmp::min(
            holder_rewards_state.unharvested_rewards,
            pool_excess_lamports,
        )
    };

    if rewards_to_harvest > 0 {
        // If there is still payment owing to the rental sponsor, then pay up to 50% of
        // the pending rewards.
        let user_rewards = if holder_rewards_state.rent_debt > 0 {
            let repayment = std::cmp::min(rewards_to_harvest / 2, holder_rewards_state.rent_debt);

            // Get the rent sponsor.
            let rent_sponsor = next_account_info(accounts_iter)?;
            if rent_sponsor.key != &holder_rewards_state.rent_sponsor {
                return Err(PaladinRewardsError::IncorrectSponsorAddress.into());
            }

            // NB: The following operations cannot over/underflow, or if they can they will
            // be caught by the runtime (unbalanced SOL transfer).
            #[allow(clippy::arithmetic_side_effects)]
            {
                // Pay the rent sponsor.
                **rent_sponsor.try_borrow_mut_lamports()? += repayment;

                // Decrease the rent debt.
                holder_rewards_state.rent_debt -= repayment;
            }

            // Remove the sponsor related fields if debt is fully repaid.
            if holder_rewards_state.rent_debt == 0 {
                holder_rewards_state.rent_sponsor = Pubkey::default();
                holder_rewards_state.minimum_balance = 0;
            }

            // NB: Cannot underflow as repayment is `min(rewards_to_harvest / 2, other)`.
            #[allow(clippy::arithmetic_side_effects)]
            {
                rewards_to_harvest - repayment
            }
        } else {
            rewards_to_harvest
        };

        // Move the amount from the holder rewards pool to the token account.
        let new_holder_rewards_pool_lamports = holder_rewards_pool_info
            .lamports()
            .checked_sub(rewards_to_harvest)
            .ok_or(ProgramError::ArithmeticOverflow)?;
        let new_token_account_lamports = token_account_info
            .lamports()
            .checked_add(user_rewards)
            .ok_or(ProgramError::ArithmeticOverflow)?;

        **holder_rewards_pool_info.try_borrow_mut_lamports()? = new_holder_rewards_pool_lamports;
        **token_account_info.try_borrow_mut_lamports()? = new_token_account_lamports;
        pool_state.lamports_last = new_holder_rewards_pool_lamports;

        // Update the holder's unharvested rewards.
        holder_rewards_state.unharvested_rewards = holder_rewards_state
            .unharvested_rewards
            .checked_sub(rewards_to_harvest)
            .ok_or(ProgramError::ArithmeticOverflow)?;
    }

    Ok(())
}

/// Processes a
/// [CloseHolderRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_close_holder_rewards(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let holder_rewards_pool_info = next_account_info(accounts_iter)?;
    let holder_rewards_info = next_account_info(accounts_iter)?;
    let token_account_info = next_account_info(accounts_iter)?;
    let mint_info = next_account_info(accounts_iter)?;
    let close_authority = next_account_info(accounts_iter)?;
    let owner = next_account_info(accounts_iter)?;

    // Load pool & holder rewards.
    check_pool(program_id, mint_info.key, holder_rewards_pool_info)?;
    let pool_data = holder_rewards_pool_info.try_borrow_data()?;
    let pool_state = bytemuck::try_from_bytes::<HolderRewardsPool>(&pool_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    check_holder_rewards(program_id, token_account_info.key, holder_rewards_info)?;
    let holder_rewards_data = holder_rewards_info.try_borrow_data()?;
    let holder_rewards_state = bytemuck::try_from_bytes::<HolderRewards>(&holder_rewards_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Ensure holder has no unclaimed rewards.
    if holder_rewards_state.last_accumulated_rewards_per_token
        < pool_state.accumulated_rewards_per_token
        || holder_rewards_state.unharvested_rewards > 0
    {
        return Err(PaladinRewardsError::CloseWithUnclaimedRewards.into());
    }

    // Load token account info (if it's not been closed).
    let (token_amount, token_owner) = (!token_account_info.data_is_empty())
        .then(|| {
            assert_eq!(token_account_info.owner, &spl_token_2022::ID);
            let token_account_data = token_account_info.data.borrow();
            let token_account_state = StateWithExtensions::<Account>::unpack(&token_account_data)
                .unwrap()
                .base;
            assert_eq!(&token_account_state.mint, mint_info.key);
            assert_eq!(mint_info.owner, &spl_token_2022::ID);

            (token_account_state.amount, token_account_state.owner)
        })
        .unwrap_or_default();

    // Ensure close authority is either:
    //
    // - The owner.
    // - OR; The sponsor AND the token balance has dropped below the initial level.
    if !close_authority.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }
    match close_authority.key {
        // NB: If the account is closed, this will be the default pubkey which is the system program
        // and cannot be a signer.
        key if key == &token_owner && holder_rewards_state.rent_debt == 0 => {
            if token_amount > 0 {
                return Err(PaladinRewardsError::InvalidClosingBalance.into());
            }
        }
        key if key == &holder_rewards_state.rent_sponsor => {
            if token_amount >= holder_rewards_state.minimum_balance {
                return Err(PaladinRewardsError::InvalidClosingBalance.into());
            }
        }
        _ => return Err(ProgramError::IncorrectAuthority),
    }

    // Grab rent debt and then drop the borrow.
    let rent_debt = holder_rewards_state.rent_debt;
    drop(holder_rewards_data);

    // Repay the closer (either sponsor or owner) and the residual to the owner.
    let close_authority_repayment = std::cmp::min(holder_rewards_info.lamports(), rent_debt);
    let owner_repayment = holder_rewards_info
        .lamports()
        .saturating_sub(close_authority_repayment);
    // NB: If this overflows then the runtime will catch it.
    #[allow(clippy::arithmetic_side_effects)]
    {
        **close_authority.lamports.borrow_mut() += close_authority_repayment;
        **owner.lamports.borrow_mut() += owner_repayment;
    }

    // Close the account.
    **holder_rewards_info.lamports.borrow_mut() = 0;
    holder_rewards_info.realloc(0, true)?;
    holder_rewards_info.assign(&system_program::ID);

    Ok(())
}

/// Processes an SPL Transfer Hook Interface
/// [ExecuteInstruction](https://docs.rs/spl-transfer-hook-interface/latest/spl_transfer_hook_interface/instruction/struct.ExecuteInstruction.html).
fn process_spl_transfer_hook_execute(
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

    // Don't accrue rewards on self transfer as there is no effective balance
    // change.
    if source_token_account_info.key == destination_token_account_info.key {
        return Ok(());
    }

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
            PaladinRewardsInstruction::InitializeHolderRewards { sponsor } => {
                msg!("Instruction: InitializeHolderRewards");
                process_initialize_holder_rewards(program_id, accounts, sponsor)
            }
            PaladinRewardsInstruction::HarvestRewards => {
                msg!("Instruction: HarvestRewards");
                process_harvest_rewards(program_id, accounts)
            }
            PaladinRewardsInstruction::CloseHolderRewards => {
                msg!("Instruction: CloseHolderRewards");
                process_close_holder_rewards(program_id, accounts)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use {super::*, proptest::prelude::*};

    const BENCH_TOKEN_SUPPLY: u64 = 1_000_000_000 * 1_000_000_000; // 1 billion with 9 decimals

    #[test]
    fn minimum_rewards_per_token() {
        // 1 lamport (arithmetic minimum)
        let minimum_reward = 1;
        let result = calculate_rewards_per_token(minimum_reward, BENCH_TOKEN_SUPPLY).unwrap();
        assert_ne!(result, 0);

        // Anything below the minimum should return zero.
        let result = calculate_rewards_per_token(minimum_reward - 1, BENCH_TOKEN_SUPPLY).unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn maximum_rewards_per_token() {
        // u64::MAX (not really practical, but shows that we're ok)
        let maximum_reward = u64::MAX;
        let _ = calculate_rewards_per_token(maximum_reward, BENCH_TOKEN_SUPPLY).unwrap();
    }

    #[test]
    fn minimum_eligible_rewards() {
        // 1 / 1e18 lamports per token
        let minimum_marginal_rewards_per_token = 1;
        let result = calculate_eligible_rewards(
            minimum_marginal_rewards_per_token,
            0,
            BENCH_TOKEN_SUPPLY, // 100% of the supply.
        )
        .unwrap();
        assert_ne!(result, 0);

        // Anything below the minimum should return zero.
        let result = calculate_eligible_rewards(
            minimum_marginal_rewards_per_token - 1,
            0,
            BENCH_TOKEN_SUPPLY, // 100% of the supply.
        )
        .unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn minimum_eligible_rewards_with_one_token() {
        // 1 / 1e9 lamports per token
        let minimum_marginal_rewards_per_token = 1_000_000_000;
        let result = calculate_eligible_rewards(
            minimum_marginal_rewards_per_token,
            0,
            BENCH_TOKEN_SUPPLY / 1_000_000_000, // 1 with 9 decimals.
        )
        .unwrap();
        assert_ne!(result, 0);

        // Anything below the minimum should return zero.
        let result = calculate_eligible_rewards(
            minimum_marginal_rewards_per_token - 1,
            0,
            BENCH_TOKEN_SUPPLY / 1_000_000_000, // 1 with 9 decimals.
        )
        .unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn minimum_eligible_rewards_with_smallest_fractional_token() {
        // 1 lamport per token
        let minimum_marginal_rewards_per_token = 1_000_000_000_000_000_000;
        let result = calculate_eligible_rewards(
            minimum_marginal_rewards_per_token,
            0,
            BENCH_TOKEN_SUPPLY / 1_000_000_000_000_000_000, // .000_000_001 with 9 decimals.
        )
        .unwrap();
        assert_ne!(result, 0);

        // Anything below the minimum should return zero.
        let result = calculate_eligible_rewards(
            minimum_marginal_rewards_per_token - 1,
            0,
            BENCH_TOKEN_SUPPLY / 1_000_000_000_000_000_000, // .000_000_001 with 9 decimals.
        )
        .unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn maximum_eligible_rewards() {
        // 1 lamport per token (not really practical, but shows that we're ok)
        let maximum_marginal_rewards_per_token = REWARDS_PER_TOKEN_SCALING_FACTOR;
        let _ = calculate_eligible_rewards(
            maximum_marginal_rewards_per_token,
            0,
            BENCH_TOKEN_SUPPLY, // 100% of the supply.
        )
        .unwrap();
    }

    #[test]
    fn wrapping_eligible_rewards() {
        // Set up current to be less than rate, simulating a scenario where the
        // current reward has wrapped around `u128::MAX`.
        let current_accumulated_rewards_per_token = 0;
        let last_accumulated_rewards_per_token = u128::MAX - 1_000_000_000_000_000_000;
        let result = calculate_eligible_rewards(
            current_accumulated_rewards_per_token,
            last_accumulated_rewards_per_token,
            BENCH_TOKEN_SUPPLY,
        )
        .unwrap();
        assert_eq!(result, 1_000_000_000_000_000_001);

        // Try it again at the very edge. Result should be one.
        let current_accumulated_rewards_per_token = 0;
        let last_accumulated_rewards_per_token = u128::MAX;
        let result = calculate_eligible_rewards(
            current_accumulated_rewards_per_token,
            last_accumulated_rewards_per_token,
            BENCH_TOKEN_SUPPLY,
        )
        .unwrap();
        assert_eq!(result, 1);
    }

    proptest! {
        #[test]
        fn test_calculate_rewards_per_token(
            rewards in 0u64..,
            token_supply in 0u64..,
        ) {
            // Calculate.
            //
            // For all possible values of rewards and token_supply, the
            // calculation should never return an error, hence the
            // `unwrap` here.
            //
            // The scaling to `u128` prevents multiplication from breaking
            // the `u64::MAX` ceiling, and the `token_supply == 0` check
            // prevents `checked_div` returning `None` from a zero
            // denominator.
            let result = calculate_rewards_per_token(rewards, token_supply).unwrap();
            // Evaluate.
            if token_supply == 0 {
                prop_assert_eq!(result, 0);
            } else {
                let expected = (rewards as u128)
                    .checked_mul(REWARDS_PER_TOKEN_SCALING_FACTOR)
                    .and_then(|product| product.checked_div(token_supply as u128))
                    .unwrap();
                prop_assert_eq!(result, expected);
            }
        }
    }

    // The marginal reward per token (current - last) within the
    // `calculate_eligible_rewards` function (tested below) is expressed in
    // terms of rewards _per token_, which is stored as a `u128` and calculated
    // by the `calculate_rewards_per_token` function (tested above).
    //
    // The return type of `calculate_eligible_rewards` is limited to
    // `u64::MAX`, but in order to determine the function's upper bounds for
    // each input parameter, we must consider the maximum marginal reward per token
    // (`current_accumulated_rewards_per_token`
    //             - `last_accumulated_rewards_per_token`)
    // and token account balance that this function can support.
    //
    // Since the marginal reward per token is at its maximum anytime a holder
    // has a "last seen rate" (`last_accumulated_rewards_per_token`) of zero,
    // we can evaluate in terms of `current_accumulated_rewards_per_token`,
    // assuming the "last seen rate" to be zero. We will stick to this
    // assumption in all references to `marginal_rewards_per_token` below.
    //
    // On its face, the maximum marginal reward per token is bound by
    // `u128::MAX` - since both `current_accumulated_rewards_per_token` and
    // `last_accumulated_rewards_per_token` are represented as `u128` integers.
    // However, since the return value is capped at `u64::MAX`, we can perform
    // the following arithmetic.
    //
    // Consider the original function:
    //
    //     eligible_rewards = (marginal_rewards_per_token * balance) / 1e18
    //
    // We can plug in `u64::MAX` for both `eligible_rewards` and `balance`
    // to calculate the input `marginal_rewards_per_token` upper bound.
    //
    //     u64::MAX = (marginal_rewards_per_token * u64::MAX) / 1e18
    //
    // And evaluate to:
    //
    //     marginal_rewards_per_token = 1e18
    //
    // This means `calculate_eligible_rewards` can handle a maximum marginal
    // reward per token of 1e18, or 1 lamport per token.
    //
    // But what does this mean as a constraint on the system as a whole? In
    // other words, if a holder had 100% of the token supply and their last
    // seen rate was zero, what's the maximum number of rewards the entire
    // system can accumulate (in lamports) before this function would break?
    //
    // We can compute this value from the formula for
    // `calculate_rewards_per_token`, which is represented below.
    //
    //     rewards_per_token = (reward * 1e18) / mint_supply
    //
    // Plugging in the values for maximum marginal reward per token and
    // `u64::MAX` for token supply...
    //
    //     1e18 = (reward * 1e18) / u64::MAX
    //
    // ... we get:
    //
    //     reward = u64::MAX
    //
    // This means that the maximum lamports that can be paid into the system
    // when a holder has 100% of the token supply and has never claimed is
    // u64::MAX.
    //
    // This also means that the two functions - `calculate_rewards_per_token`
    // and `calculate_eligible_rewards` share the same upper bound, since
    // `calculate_rewards_per_token` expects a `u64` for rewards.
    //
    // However, since rewards can be paid into the system incrementally, and
    // are stored as _rewards per token_ in a `u128`, it's mathematically
    // possible for the system to receive more than `u64::MAX` over time.
    //
    // It's worth noting that `u64::MAX` exceeds the current circulating supply
    // of SOL (`4.66e15`) by 386_266 %.
    //
    // That being said, we can pipe `u64::MAX` into `calculate_rewards_per_token`
    // as the function's upper bound for proptesting. This will also max-out at
    // 1e18.
    prop_compose! {
        fn current_last_and_balance(max_accumulated_rewards: u64)
        (mint_supply in 0..u64::MAX)
        (
            current in 0..=calculate_rewards_per_token(
                max_accumulated_rewards,
                mint_supply,
            ).unwrap(),
            balance in 0..=mint_supply,
        ) -> (u128, u128, u64) {
            (
                current, // Current accumulated rewards per token.
                0,       // Last accumulated rewards per token (always 0 here for maximum margin).
                balance, // Token account balance (up to 100% of mint supply).
            )
        }
    }
    proptest! {
        #[test]
        fn test_calculate_eligible_rewards(
            (
                current_accumulated_rewards_per_token,
                last_accumulated_rewards_per_token,
                token_account_balance,
            ) in current_last_and_balance(u64::MAX),
        ) {
            // Calculate.
            let result = calculate_eligible_rewards(
                current_accumulated_rewards_per_token,
                last_accumulated_rewards_per_token,
                token_account_balance,
            )
            .unwrap();
            // Evaluate.
            //
            // Since we've configured the inputs so that last never exceeds
            // current, this subtraction never overflows, so it's safe to
            // unwrap here.
            let marginal_rate = current_accumulated_rewards_per_token
                .checked_sub(last_accumulated_rewards_per_token)
                .unwrap();
            if marginal_rate == 0 {
                // If the marginal rate resolves to zero, the
                // calculation should short-circuit and return zero.
                prop_assert_eq!(result, 0);
            } else {
                // The rest of the calculation consists of three steps,
                // so evaluate each step one at a time.
                //
                // 1. marginal rate x token account balance
                // 2. product / REWARDS_PER_TOKEN_SCALING_FACTOR
                // 3. product.try_into (u64)

                // Step 1.
                //
                // Since we've restricted the inputs within the bounds
                // of the system, the multiplication should never exceed
                // `u128::MAX`.
                let marginal_rewards = marginal_rate
                    .checked_mul(token_account_balance as u128)
                    .unwrap();

                // Step 2.
                //
                // Since we're always dividing by a non-zero constant,
                // the division should never return `None`, so we can
                // unwrap here.
                let descaled_marginal_rewards = marginal_rewards
                    .checked_div(REWARDS_PER_TOKEN_SCALING_FACTOR)
                    .unwrap();

                // Step 3.
                //
                // Since we've restricted the inputs within the bounds
                // of the system, the conversion to `u64` should always
                // succeed.
                let expected_result: u64 = descaled_marginal_rewards
                    .try_into()
                    .unwrap();

                // The calculation should return the expected value.
                prop_assert_eq!(result, expected_result);
            }
        }
    }
}
