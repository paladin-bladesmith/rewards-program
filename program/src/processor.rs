//! Program processor.

use {
    crate::{
        error::PaladinRewardsError,
        instruction::PaladinRewardsInstruction,
        state::{
            collect_holder_rewards_pool_signer_seeds, collect_holder_rewards_signer_seeds,
            find_duna_document_pda, get_holder_rewards_address,
            get_holder_rewards_address_and_bump_seed, get_holder_rewards_pool_address,
            get_holder_rewards_pool_address_and_bump_seed, HolderRewards, HolderRewardsPool,
        },
    },
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        msg,
        program::{invoke, invoke_signed},
        program_error::ProgramError,
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
        system_instruction, system_program,
        sysvar::Sysvar,
    },
    spl_token::{
        instruction::transfer,
        state::{Account as TokenAccount, AccountState, Mint},
    },
};

pub const REWARDS_PER_TOKEN_SCALING_FACTOR: u128 = 1_000_000_000_000_000_000; // 1e18

fn get_token_account_balance_checked(
    mint: &Pubkey,
    token_account_info: &AccountInfo,
) -> Result<u64, ProgramError> {
    assert_eq!(token_account_info.owner, &spl_token::ID);
    let token_account_data = token_account_info.try_borrow_data()?;
    let token_account = TokenAccount::unpack(&token_account_data)?;

    // Ensure the provided token account is for the mint.
    if !token_account.mint.eq(mint) {
        return Err(PaladinRewardsError::TokenAccountMintMismatch.into());
    }

    Ok(token_account.amount)
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
    owner_address: &Pubkey,
    holder_rewards_info: &AccountInfo,
) -> ProgramResult {
    // Ensure the holder rewards account is owned by the Paladin Rewards
    // program.
    if holder_rewards_info.owner != program_id {
        return Err(ProgramError::InvalidAccountOwner);
    }

    // Ensure the provided holder rewards address is the correct address
    // derived from the token account.
    if holder_rewards_info.key != &get_holder_rewards_address(owner_address, program_id) {
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
fn calculate_rewards_per_token(rewards: u64, total_deposited: u64) -> Result<u128, ProgramError> {
    if total_deposited == 0 {
        return Ok(0);
    }
    (rewards as u128)
        .checked_mul(REWARDS_PER_TOKEN_SCALING_FACTOR)
        .and_then(|product| product.checked_div(total_deposited as u128))
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
    deposited_amount: u64,
) -> Result<u64, ProgramError> {
    let marginal_rate =
        current_accumulated_rewards_per_token.wrapping_sub(last_accumulated_rewards_per_token);

    if marginal_rate == 0 {
        return Ok(0);
    }
    marginal_rate
        .checked_mul(deposited_amount as u128)
        .and_then(|product| product.checked_div(REWARDS_PER_TOKEN_SCALING_FACTOR))
        .and_then(|product| product.try_into().ok())
        .ok_or(ProgramError::ArithmeticOverflow)
}

fn update_accumulated_rewards_per_token(
    mint_info: &AccountInfo,
    holder_rewards_pool_info: &AccountInfo,
    pool_token_account: &AccountInfo,
    pool_state: &mut HolderRewardsPool,
) -> ProgramResult {
    let total_deposited = get_token_account_balance_checked(mint_info.key, pool_token_account)?;
    let latest_lamports = holder_rewards_pool_info.lamports();

    let additional_lamports = latest_lamports
        .checked_sub(pool_state.lamports_last)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    let marginal_rate = calculate_rewards_per_token(additional_lamports, total_deposited)?;

    pool_state.accumulated_rewards_per_token = pool_state
        .accumulated_rewards_per_token
        .wrapping_add(marginal_rate);
    pool_state.lamports_last = latest_lamports;

    Ok(())
}

fn assert_rent_exempt(account: &AccountInfo) {
    assert!(account.lamports() >= Rent::get().unwrap().minimum_balance(account.data_len()));
}

fn validate_token_account(
    token_account_info: &AccountInfo,
    expected_owner: &Pubkey,
    expected_mint: &Pubkey,
) -> ProgramResult {
    // Check if account is owned by SPL Token program
    if token_account_info.owner != &spl_token::id() {
        msg!("Token account not owned by SPL Token program");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Check rent exemption
    let rent = Rent::get()?;
    if !rent.is_exempt(token_account_info.lamports(), token_account_info.data_len()) {
        msg!("Token account is not rent exempt");
        return Err(ProgramError::InsufficientFunds);
    }

    // Deserialize token account data
    let token_account = TokenAccount::unpack(&token_account_info.data.borrow())?;

    // Check if account is initialized
    if token_account.state != AccountState::Initialized {
        msg!("Token account is not initialized");
        return Err(ProgramError::UninitializedAccount);
    }

    // Verify owner
    if token_account.owner != *expected_owner {
        msg!("Invalid token account owner");
        return Err(PaladinRewardsError::TokenAccountOwnerMissmatch.into());
    }

    // Verify mint
    if token_account.mint != *expected_mint {
        msg!("Invalid token account mint");
        return Err(PaladinRewardsError::TokenAccountMintMismatch.into());
    }

    // Check if account is not frozen
    if token_account.state == AccountState::Frozen {
        msg!("Token account is frozen");
        return Err(PaladinRewardsError::TokenAccountFrozen.into());
    }

    Ok(())
}

/// Calculate the amount of rewards that can be harvested by the holder
///
/// This is done by subtracting the `last_accumulated_rewards_per_token`
/// rate from the pool's current rate, then multiplying by the token account
/// balance.
///
/// The holder should also be able to harvest any unharvested rewards.
fn calculate_rewards_to_harvest(
    holder_rewards_state: &mut HolderRewards,
    pool_state: &HolderRewardsPool,
    pool_lamports: u64,
) -> Result<u64, ProgramError> {
    // Calculate the eligible rewards from the marginal rate.
    let eligible_rewards = calculate_eligible_rewards(
        pool_state.accumulated_rewards_per_token,
        holder_rewards_state.last_accumulated_rewards_per_token,
        holder_rewards_state.deposited,
    )?;

    // Error if the pool doesn't have enough lamports to cover the rewards,
    // This should never happen, but the check is a failsafe.
    let pool_excess_lamports = {
        let rent = <Rent as Sysvar>::get()?;
        let rent_exempt_lamports = rent.minimum_balance(HolderRewardsPool::LEN);
        pool_lamports.saturating_sub(rent_exempt_lamports)
    };

    if eligible_rewards > pool_excess_lamports {
        return Err(PaladinRewardsError::RewardsExcessPoolBalance.into());
    }

    // Update the holder rewards state with last rewards per token
    holder_rewards_state.last_accumulated_rewards_per_token =
        pool_state.accumulated_rewards_per_token;

    Ok(eligible_rewards)
}

// Send the rewards to the holder's token account.
fn send_rewards(
    holder_rewards_pool_info: AccountInfo,
    owner: AccountInfo,
    pool_state: &mut HolderRewardsPool,
    rewards_to_harvest: u64,
) -> ProgramResult {
    // Move the amount from the holder rewards pool to the token account.
    let new_holder_rewards_pool_lamports = holder_rewards_pool_info
        .lamports()
        .checked_sub(rewards_to_harvest)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    let new_token_account_lamports = owner
        .lamports()
        .checked_add(rewards_to_harvest)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    **holder_rewards_pool_info.try_borrow_mut_lamports()? = new_holder_rewards_pool_lamports;
    **owner.try_borrow_mut_lamports()? = new_token_account_lamports;
    pool_state.lamports_last = new_holder_rewards_pool_lamports;

    Ok(())
}

// Check that duna document is signed
pub(crate) fn check_duna_document_signed(
    signer: &Pubkey,
    doc_pda: &AccountInfo,
    doc_hash: &[u8; 32],
) -> ProgramResult {
    let (duna_document_pda, _) = find_duna_document_pda(signer, doc_hash);

    // Check the duna document PDA is correct.
    if doc_pda.key != &duna_document_pda {
        return Err(PaladinRewardsError::InvalidDunaPdaSeeds.into());
    }
    // Ensure the duna document PDA is initialized.
    let duna_document_data = doc_pda.try_borrow_data()?;

    if duna_document_data.first() != Some(&1) {
        return Err(PaladinRewardsError::DunaDocumentNotInitialized.into());
    }

    Ok(())
}

/// Processes an
/// [InitializeHolderRewardsPool](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_initialize_holder_rewards_pool(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    stake_program_vault_pda: Pubkey,
    duna_document_hash: [u8; 32],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let holder_rewards_pool_info = next_account_info(accounts_iter)?;
    let holder_rewards_pool_token_account_info = next_account_info(accounts_iter)?;
    let mint_info = next_account_info(accounts_iter)?;
    let _system_program_info = next_account_info(accounts_iter)?;

    // Run checks on the mint.
    assert_eq!(mint_info.owner, &spl_token::ID);
    let mint_data = mint_info.try_borrow_data()?;
    Mint::unpack(&mint_data)?;

    // Validate pool token account
    validate_token_account(
        holder_rewards_pool_token_account_info,
        holder_rewards_pool_info.key,
        mint_info.key,
    )?;

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
                HolderRewardsPool::LEN as u64,
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
                stake_program_vault_pda,
                duna_document_hash,
                _padding: 0,
            };
    }

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
    let holder_rewards_pool_token_account_info = next_account_info(accounts_iter)?;
    let owner = next_account_info(accounts_iter)?;
    let holder_rewards_info = next_account_info(accounts_iter)?;
    let mint_info = next_account_info(accounts_iter)?;
    let duna_document_info = next_account_info(accounts_iter)?;
    let _system_program = next_account_info(accounts_iter)?;

    validate_token_account(
        holder_rewards_pool_token_account_info,
        holder_rewards_pool_info.key,
        mint_info.key,
    )?;

    // Confirm owner is the signer
    if !owner.is_signer {
        return Err(PaladinRewardsError::OwnerNotSigner.into());
    }

    // Run checks on the pool account.
    check_pool(program_id, mint_info.key, holder_rewards_pool_info)?;
    let mut pool_data = holder_rewards_pool_info.try_borrow_mut_data()?;
    let pool_state = bytemuck::try_from_bytes_mut::<HolderRewardsPool>(&mut pool_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // If owner is not stake program vault pda, owner must have duna signed
    if owner.key != &pool_state.stake_program_vault_pda {
        // Check duna is signed
        check_duna_document_signed(
            owner.key,
            duna_document_info,
            &pool_state.duna_document_hash,
        )?;
    }

    // Process any received lamports.
    update_accumulated_rewards_per_token(
        mint_info,
        holder_rewards_pool_info,
        holder_rewards_pool_token_account_info,
        pool_state,
    )?;

    // Initialize the holder rewards account.
    {
        let (holder_rewards_address, bump_seed) =
            get_holder_rewards_address_and_bump_seed(owner.key, program_id);
        let bump_seed = [bump_seed];
        let holder_rewards_signer_seeds =
            collect_holder_rewards_signer_seeds(owner.key, &bump_seed);

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
            &system_instruction::allocate(&holder_rewards_address, HolderRewards::LEN as u64),
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
                deposited: 0,
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
    let holder_rewards_pool_token_account_info = next_account_info(accounts_iter)?;
    let holder_rewards_info = next_account_info(accounts_iter)?;
    let mint_info = next_account_info(accounts_iter)?;
    let owner = next_account_info(accounts_iter)?;

    // Ensure signer is the owner and can close this account
    if !owner.is_signer {
        return Err(PaladinRewardsError::OwnerNotSigner.into());
    }

    validate_token_account(
        holder_rewards_pool_token_account_info,
        holder_rewards_pool_info.key,
        mint_info.key,
    )?;

    // Check & load the pool
    check_pool(program_id, mint_info.key, holder_rewards_pool_info)?;
    let mut pool_data = holder_rewards_pool_info.try_borrow_mut_data()?;
    let pool_state = bytemuck::try_from_bytes_mut::<HolderRewardsPool>(&mut pool_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Check & load the holder rewards.
    check_holder_rewards(program_id, owner.key, holder_rewards_info)?;
    let mut holder_rewards_data = holder_rewards_info.try_borrow_mut_data()?;
    let holder_rewards_state =
        bytemuck::try_from_bytes_mut::<HolderRewards>(&mut holder_rewards_data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

    // Handle any lamports received since last harvest.
    update_accumulated_rewards_per_token(
        mint_info,
        holder_rewards_pool_info,
        holder_rewards_pool_token_account_info,
        pool_state,
    )?;

    // Determine the amount the holder can harvest.
    let rewards_to_harvest = calculate_rewards_to_harvest(
        holder_rewards_state,
        pool_state,
        holder_rewards_pool_info.lamports(),
    )?;

    if rewards_to_harvest > 0 {
        send_rewards(
            holder_rewards_pool_info.clone(),
            owner.clone(),
            pool_state,
            rewards_to_harvest,
        )?;
    }

    Ok(())
}

/// Processes a
/// [CloseHolderRewards](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_close_holder_rewards(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let holder_rewards_pool_info = next_account_info(accounts_iter)?;
    let holder_rewards_pool_token_account_info = next_account_info(accounts_iter)?;
    let holder_rewards_info = next_account_info(accounts_iter)?;
    let mint_info = next_account_info(accounts_iter)?;
    let owner = next_account_info(accounts_iter)?;

    validate_token_account(
        holder_rewards_pool_token_account_info,
        holder_rewards_pool_info.key,
        mint_info.key,
    )?;

    // Ensure signer is the owner and can close this account
    if !owner.is_signer {
        return Err(PaladinRewardsError::OwnerNotSigner.into());
    }

    // Load pool & holder rewards.
    check_pool(program_id, mint_info.key, holder_rewards_pool_info)?;
    let mut pool_data = holder_rewards_pool_info.try_borrow_mut_data()?;
    let pool_state = bytemuck::try_from_bytes_mut::<HolderRewardsPool>(&mut pool_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    check_holder_rewards(program_id, owner.key, holder_rewards_info)?;
    let holder_rewards_data = holder_rewards_info.try_borrow_data()?;
    let holder_rewards_state = bytemuck::try_from_bytes::<HolderRewards>(&holder_rewards_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Handle any lamports received since last harvest.
    update_accumulated_rewards_per_token(
        mint_info,
        holder_rewards_pool_info,
        holder_rewards_pool_token_account_info,
        pool_state,
    )?;

    // Ensure holder has no unclaimed rewards.
    if holder_rewards_state.last_accumulated_rewards_per_token
        < pool_state.accumulated_rewards_per_token
    {
        return Err(PaladinRewardsError::CloseWithUnclaimedRewards.into());
    }

    // Ensure holder withdrew all tokens
    if holder_rewards_state.deposited > 0 {
        return Err(PaladinRewardsError::CloseWithDepositedTokens.into());
    }

    drop(holder_rewards_data);

    // NB: If this overflows then the runtime will catch it.
    #[allow(clippy::arithmetic_side_effects)]
    {
        **owner.lamports.borrow_mut() += holder_rewards_info.lamports();
    }

    // Close the account.
    **holder_rewards_info.lamports.borrow_mut() = 0;
    holder_rewards_info.realloc(0, true)?;
    holder_rewards_info.assign(&system_program::ID);

    Ok(())
}

/// Processes a [Deposit](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_deposit(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let holder_rewards_pool_info = next_account_info(accounts_iter)?;
    let holder_rewards_pool_token_account_info = next_account_info(accounts_iter)?;
    let holder_rewards_info = next_account_info(accounts_iter)?;
    let token_account_info = next_account_info(accounts_iter)?;
    let mint_info = next_account_info(accounts_iter)?;
    let owner = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Ensure signer is the owner and can close this account.
    if !owner.is_signer {
        return Err(PaladinRewardsError::OwnerNotSigner.into());
    }

    // Validate pool token account.
    validate_token_account(
        holder_rewards_pool_token_account_info,
        holder_rewards_pool_info.key,
        mint_info.key,
    )?;

    // Validate the owner token account.
    validate_token_account(token_account_info, owner.key, mint_info.key)?;

    // Validate user has enough tokens to deposit.
    let owner_balance = get_token_account_balance_checked(mint_info.key, token_account_info)?;
    if owner_balance < amount {
        return Err(PaladinRewardsError::NotEnoughTokenToDeposit.into());
    }

    // Load pool & holder rewards.
    check_pool(program_id, mint_info.key, holder_rewards_pool_info)?;
    let mut pool_data = holder_rewards_pool_info.try_borrow_mut_data()?;
    let pool_state = bytemuck::try_from_bytes_mut::<HolderRewardsPool>(&mut pool_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    check_holder_rewards(program_id, owner.key, holder_rewards_info)?;
    let mut holder_rewards_data = holder_rewards_info.try_borrow_mut_data()?;
    let holder_rewards_state =
        bytemuck::try_from_bytes_mut::<HolderRewards>(&mut holder_rewards_data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

    // Handle any lamports received since last harvest.
    update_accumulated_rewards_per_token(
        mint_info,
        holder_rewards_pool_info,
        holder_rewards_pool_token_account_info,
        pool_state,
    )?;

    // Calculate rewards to harvest before new deposit
    let rewards_to_harvest = calculate_rewards_to_harvest(
        holder_rewards_state,
        pool_state,
        holder_rewards_pool_info.lamports(),
    )?;

    // Update total deposited tokens
    holder_rewards_state.deposited = holder_rewards_state
        .deposited
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Transfer tokens from the owner to the holder rewards pool.
    let transfer_ix = transfer(
        &spl_token::ID,
        token_account_info.key,
        holder_rewards_pool_token_account_info.key,
        owner.key,
        &[owner.key],
        amount,
    )?;

    invoke(
        &transfer_ix,
        &[
            token_account_info.clone(),
            holder_rewards_pool_token_account_info.clone(),
            owner.clone(),
            token_program.clone(),
        ],
    )?;

    // Send rewards to the owner
    if rewards_to_harvest > 0 {
        send_rewards(
            holder_rewards_pool_info.clone(),
            owner.clone(),
            pool_state,
            rewards_to_harvest,
        )?;
    }

    Ok(())
}

/// Processes a [Withdraw](enum.PaladinRewardsInstruction.html)
/// instruction.
fn process_withdraw(program_id: &Pubkey, accounts: &[AccountInfo], amount: u64) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let holder_rewards_pool_info = next_account_info(accounts_iter)?;
    let holder_rewards_pool_token_account_info = next_account_info(accounts_iter)?;
    let holder_rewards_info = next_account_info(accounts_iter)?;
    let token_account_info = next_account_info(accounts_iter)?;
    let mint_info = next_account_info(accounts_iter)?;
    let owner = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Ensure signer is the owner and can close this account
    if !owner.is_signer {
        return Err(PaladinRewardsError::OwnerNotSigner.into());
    }

    // Validate pool token account
    validate_token_account(
        holder_rewards_pool_token_account_info,
        holder_rewards_pool_info.key,
        mint_info.key,
    )?;

    // Validate the owner token account
    validate_token_account(token_account_info, owner.key, mint_info.key)?;

    // Load pool & holder rewards.
    check_pool(program_id, mint_info.key, holder_rewards_pool_info)?;
    let mut pool_data = holder_rewards_pool_info.try_borrow_mut_data()?;
    let pool_state = bytemuck::try_from_bytes_mut::<HolderRewardsPool>(&mut pool_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;
    check_holder_rewards(program_id, owner.key, holder_rewards_info)?;
    let mut holder_rewards_data = holder_rewards_info.try_borrow_mut_data()?;
    let holder_rewards_state =
        bytemuck::try_from_bytes_mut::<HolderRewards>(&mut holder_rewards_data)
            .map_err(|_| ProgramError::InvalidAccountData)?;

    // Validate that we have enough deposited tokens to withdraw
    let pool_balance =
        get_token_account_balance_checked(mint_info.key, holder_rewards_pool_token_account_info)?;
    let to_withdraw = if amount == u64::MAX {
        holder_rewards_state.deposited
    } else {
        amount
    };

    if holder_rewards_state.deposited == 0 {
        return Err(PaladinRewardsError::NoDepositedTokensToWithdraw.into());
    } else if holder_rewards_state.deposited > pool_balance {
        return Err(PaladinRewardsError::WithdrawExceedsPoolBalance.into());
    } else if to_withdraw > holder_rewards_state.deposited {
        return Err(PaladinRewardsError::WithdrawExceedsDeposited.into());
    }

    // Handle any lamports received since last harvest.
    update_accumulated_rewards_per_token(
        mint_info,
        holder_rewards_pool_info,
        holder_rewards_pool_token_account_info,
        pool_state,
    )?;

    // Calculate rewards to harvest before withdrawal
    let rewards_to_harvest = match calculate_rewards_to_harvest(
        holder_rewards_state,
        pool_state,
        holder_rewards_pool_info.lamports(),
    ) {
        Ok(rewards) => Ok(rewards),
        Err(ProgramError::Custom(err)) => {
            // If the pool does not have enough lamports to cover the rewards,
            // we set the amount to 0
            if err == PaladinRewardsError::RewardsExcessPoolBalance as u32 {
                Ok(0)
            } else {
                return Err(ProgramError::Custom(err));
            }
        }
        Err(err) => Err(err),
    }?;

    // Update total deposited tokens
    holder_rewards_state.deposited = holder_rewards_state
        .deposited
        .checked_sub(to_withdraw)
        .ok_or(ProgramError::ArithmeticOverflow)?;

    // Get pool token account signer seeds.
    let (_, bump_seed) = get_holder_rewards_pool_address_and_bump_seed(mint_info.key, program_id);
    let bump_seed = [bump_seed];
    let holder_rewards_pool_signer_seeds =
        collect_holder_rewards_pool_signer_seeds(mint_info.key, &bump_seed);

    // Transfer tokens from the pool to the owner.
    let transfer_ix = transfer(
        &spl_token::ID,
        holder_rewards_pool_token_account_info.key,
        token_account_info.key,
        holder_rewards_pool_info.key,
        &[holder_rewards_pool_info.key],
        to_withdraw,
    )?;

    drop(pool_data);
    invoke_signed(
        &transfer_ix,
        &[
            holder_rewards_pool_token_account_info.clone(),
            token_account_info.clone(),
            holder_rewards_pool_info.clone(),
            token_program.clone(),
        ],
        &[&holder_rewards_pool_signer_seeds],
    )?;

    // re-borrow the pool data to use in `send_rewards`
    let mut pool_data = holder_rewards_pool_info.try_borrow_mut_data()?;
    let pool_state = bytemuck::try_from_bytes_mut::<HolderRewardsPool>(&mut pool_data)
        .map_err(|_| ProgramError::InvalidAccountData)?;

    // Send rewards to the owner
    if rewards_to_harvest > 0 {
        send_rewards(
            holder_rewards_pool_info.clone(),
            owner.clone(),
            pool_state,
            rewards_to_harvest,
        )?;
    }

    Ok(())
}

/// Processes a
/// [PaladinRewardsInstruction](enum.PaladinRewardsInstruction.html).
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = PaladinRewardsInstruction::unpack(input)?;
    match instruction {
        PaladinRewardsInstruction::InitializeHolderRewardsPool {
            stake_program_vault_pda,
            duna_document_hash,
        } => {
            msg!("Instruction: InitializeHolderRewardsPool");
            process_initialize_holder_rewards_pool(
                program_id,
                accounts,
                stake_program_vault_pda,
                duna_document_hash,
            )
        }
        PaladinRewardsInstruction::InitializeHolderRewards => {
            msg!("Instruction: InitializeHolderRewards");
            process_initialize_holder_rewards(program_id, accounts)
        }
        PaladinRewardsInstruction::HarvestRewards => {
            msg!("Instruction: HarvestRewards");
            process_harvest_rewards(program_id, accounts)
        }
        PaladinRewardsInstruction::CloseHolderRewards => {
            msg!("Instruction: CloseHolderRewards");
            process_close_holder_rewards(program_id, accounts)
        }
        PaladinRewardsInstruction::Deposit { amount } => {
            msg!("Instruction: Deposit");
            process_deposit(program_id, accounts, amount)
        }
        PaladinRewardsInstruction::Withdraw { amount } => {
            msg!("Instruction: Withdraw");
            process_withdraw(program_id, accounts, amount)
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
