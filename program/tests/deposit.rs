#![cfg(feature = "test-sbf")]

mod execute_utils;
mod setup;

use {
    crate::{
        execute_utils::{execute_with_payer, execute_with_payer_err},
        setup::{
            send_rewards_to_pool, setup_holder_rewards_account_with_token_account,
            setup_holder_rewards_pool_account_with_token_account, DEPOSIT_AMOUNT,
            INITIAL_OWNER_BALANCE,
        },
    },
    paladin_rewards_program::{
        error::PaladinRewardsError,
        processor::REWARDS_PER_TOKEN_SCALING_FACTOR,
        state::{
            get_holder_rewards_address, get_holder_rewards_pool_address, HolderRewards,
            HolderRewardsPool,
        },
    },
    paladin_rewards_program_client::instructions::DepositBuilder,
    setup::setup,
    solana_program_test::*,
    solana_sdk::{
        instruction::InstructionError, program_pack::Pack, pubkey::Pubkey, signature::Keypair,
        signer::Signer, transaction::TransactionError,
    },
    spl_associated_token_account::get_associated_token_address,
    spl_token::state::Account as TokenAccount,
};

#[tokio::test]
async fn fail_pool_doesnt_have_enough_rewards() {
    let owner = Keypair::new();
    let mint = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

    // Setup pool
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let pool_token = get_associated_token_address(&holder_rewards_pool, &mint);

    setup_holder_rewards_pool_account_with_token_account(
        &mut context,
        &mint,
        &holder_rewards_pool,
        &pool_token,
        0,                                                           // no rewards
        REWARDS_PER_TOKEN_SCALING_FACTOR.checked_mul(1000).unwrap(), // accumalated per token
        100_000_000_000, // pool balance (total deposited by all users)
    )
    .await;

    // Setup token account for the owner.
    let holder_rewards =
        get_holder_rewards_address(&owner.pubkey(), &paladin_rewards_program::id());
    let owner_token = get_associated_token_address(&owner.pubkey(), &mint);
    setup_holder_rewards_account_with_token_account(
        &mut context,
        &mint,
        &owner.pubkey(),
        &holder_rewards,
        &owner_token,
        100_000_000_000,       // total deposited for holder
        10_000,                // last rewards per token
        INITIAL_OWNER_BALANCE, // token balance
    )
    .await;

    let instruction = DepositBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token)
        .holder_rewards(holder_rewards)
        .token_account(owner_token)
        .mint(mint)
        .owner(owner.pubkey())
        .amount(DEPOSIT_AMOUNT)
        .instruction();
    let err = execute_with_payer_err(&mut context, instruction, Some(&owner)).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinRewardsError::RewardsExcessPoolBalance as u32)
        )
    );
}

#[tokio::test]
async fn fail_not_enough_tokens_to_deposit() {
    let owner = Keypair::new();
    let mint = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

    // Setup pool
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let pool_token = get_associated_token_address(&holder_rewards_pool, &mint);

    setup_holder_rewards_pool_account_with_token_account(
        &mut context,
        &mint,
        &holder_rewards_pool,
        &pool_token,
        0,
        0,
        0,
    )
    .await;

    // Setup token account for the owner.
    let holder_rewards =
        get_holder_rewards_address(&owner.pubkey(), &paladin_rewards_program::id());
    let owner_token = get_associated_token_address(&owner.pubkey(), &mint);
    setup_holder_rewards_account_with_token_account(
        &mut context,
        &mint,
        &owner.pubkey(),
        &holder_rewards,
        &owner_token,
        0,
        0,
        INITIAL_OWNER_BALANCE - 1,
    )
    .await;

    let instruction = DepositBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token)
        .holder_rewards(holder_rewards)
        .token_account(owner_token)
        .mint(mint)
        .owner(owner.pubkey())
        .amount(INITIAL_OWNER_BALANCE)
        .instruction();
    let err = execute_with_payer_err(&mut context, instruction, Some(&owner)).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinRewardsError::NotEnoughTokenToDeposit as u32)
        )
    );
}

#[tokio::test]
async fn success() {
    let owner = Keypair::new();
    let mint = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

    // Setup pool
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let pool_token = get_associated_token_address(&holder_rewards_pool, &mint);

    setup_holder_rewards_pool_account_with_token_account(
        &mut context,
        &mint,
        &holder_rewards_pool,
        &pool_token,
        0,
        0,
        0,
    )
    .await;

    // Setup token account for the owner.
    let holder_rewards =
        get_holder_rewards_address(&owner.pubkey(), &paladin_rewards_program::id());
    let owner_token = get_associated_token_address(&owner.pubkey(), &mint);
    setup_holder_rewards_account_with_token_account(
        &mut context,
        &mint,
        &owner.pubkey(),
        &holder_rewards,
        &owner_token,
        0,
        0,
        INITIAL_OWNER_BALANCE,
    )
    .await;

    let instruction = DepositBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token)
        .holder_rewards(holder_rewards)
        .token_account(owner_token)
        .mint(mint)
        .owner(owner.pubkey())
        .amount(DEPOSIT_AMOUNT)
        .instruction();
    execute_with_payer(&mut context, instruction, Some(&owner)).await;

    // Assert pool balance is DEPOSIT_AMOUNT.
    let pool_token_account = context
        .banks_client
        .get_account(pool_token)
        .await
        .unwrap()
        .unwrap();
    let pool_token_account_balance = TokenAccount::unpack(&pool_token_account.data)
        .unwrap()
        .amount;
    assert_eq!(pool_token_account_balance, DEPOSIT_AMOUNT);

    // Assert owner balance is INITIAL_OWNER_BALANCE - DEPOSIT_AMOUNT.
    let owner_token_account = context
        .banks_client
        .get_account(owner_token)
        .await
        .unwrap()
        .unwrap();
    let owner_token_account_balance = TokenAccount::unpack(&owner_token_account.data)
        .unwrap()
        .amount;
    assert_eq!(
        owner_token_account_balance,
        INITIAL_OWNER_BALANCE - DEPOSIT_AMOUNT
    );

    let holder_rewards_account = context
        .banks_client
        .get_account(holder_rewards)
        .await
        .unwrap()
        .unwrap();
    let holder_deposited =
        bytemuck::from_bytes::<HolderRewards>(&holder_rewards_account.data).deposited;
    assert_eq!(holder_deposited, DEPOSIT_AMOUNT);

    // Confirm that rewards are being sent on 2nd deposit.
    let rewards_amount = 1_000_000_000;
    let previous_pool_lamports = context
        .banks_client
        .get_account(holder_rewards_pool)
        .await
        .unwrap()
        .unwrap()
        .lamports;
    send_rewards_to_pool(&mut context, &holder_rewards_pool, rewards_amount).await;
    // Assert rewards were sent
    let check_pool_lamports = context
        .banks_client
        .get_account(holder_rewards_pool)
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(check_pool_lamports, previous_pool_lamports + rewards_amount);

    // Deposit again to check if rewards are being sent
    let instruction = DepositBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token)
        .holder_rewards(holder_rewards)
        .token_account(owner_token)
        .mint(mint)
        .owner(owner.pubkey())
        .amount(DEPOSIT_AMOUNT / 2)
        .instruction();
    execute_with_payer(&mut context, instruction, Some(&owner)).await;

    // Assert pool balance is DEPOSIT_AMOUNT * 2.
    let pool_token_account = context
        .banks_client
        .get_account(pool_token)
        .await
        .unwrap()
        .unwrap();
    let pool_token_account_balance = TokenAccount::unpack(&pool_token_account.data)
        .unwrap()
        .amount;
    assert_eq!(
        pool_token_account_balance,
        DEPOSIT_AMOUNT + DEPOSIT_AMOUNT / 2
    );

    // Assert owner balance is INITIAL_OWNER_BALANCE - DEPOSIT_AMOUNT * 2.
    let owner_token_account = context
        .banks_client
        .get_account(owner_token)
        .await
        .unwrap()
        .unwrap();
    let owner_token_account_balance = TokenAccount::unpack(&owner_token_account.data)
        .unwrap()
        .amount;
    assert_eq!(
        owner_token_account_balance,
        INITIAL_OWNER_BALANCE - (DEPOSIT_AMOUNT + DEPOSIT_AMOUNT / 2)
    );

    let holder_rewards_pool_account = context
        .banks_client
        .get_account(holder_rewards_pool)
        .await
        .unwrap()
        .unwrap();
    let pool_state = bytemuck::from_bytes::<HolderRewardsPool>(&holder_rewards_pool_account.data);
    let holder_reward_account = context
        .banks_client
        .get_account(holder_rewards)
        .await
        .unwrap()
        .unwrap();
    let holder_rewards_state = bytemuck::from_bytes::<HolderRewards>(&holder_reward_account.data);
    assert_eq!(
        holder_rewards_state.last_accumulated_rewards_per_token,
        pool_state.accumulated_rewards_per_token
    );

    // Assert pool sent all rewards to holder (single holder)
    let current_pool_lamports = context
        .banks_client
        .get_account(holder_rewards_pool)
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(current_pool_lamports, previous_pool_lamports);

    // Assert rewards were sent to owner
    let current_owner_lamports = context
        .banks_client
        .get_account(owner.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(current_owner_lamports, rewards_amount);
}
