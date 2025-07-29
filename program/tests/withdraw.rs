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
        state::{get_holder_rewards_address, get_holder_rewards_pool_address},
    },
    paladin_rewards_program_client::instructions::WithdrawBuilder,
    setup::setup,
    solana_program_test::*,
    solana_sdk::{
        instruction::InstructionError, program_pack::Pack, pubkey::Pubkey, signature::Keypair,
        signer::Signer, transaction::TransactionError,
    },
    spl_associated_token_account::get_associated_token_address,
    spl_token::state::Account as TokenAccount,
};

pub const REWARDS_AMOUNT: u64 = 100_000_000_000_000;

#[tokio::test]
async fn fail_empty_pool() {
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
        DEPOSIT_AMOUNT,
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
        INITIAL_OWNER_BALANCE,
        0,
        0,
    )
    .await;

    // Should fail as the user have more deposited tokens than the pool owns
    let instruction = WithdrawBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token)
        .holder_rewards(holder_rewards)
        .token_account(owner_token)
        .mint(mint)
        .owner(owner.pubkey())
        .amount(0)
        .instruction();
    let err = execute_with_payer_err(&mut context, instruction, Some(&owner)).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinRewardsError::WithdrawExceedsPoolBalance as u32)
        )
    );
}

#[tokio::test]
async fn fail_no_deposited_tokens() {
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
        DEPOSIT_AMOUNT,
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

    // Should fail as there are no tokens deposited in the pool by this user
    let instruction = WithdrawBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token)
        .holder_rewards(holder_rewards)
        .token_account(owner_token)
        .mint(mint)
        .owner(owner.pubkey())
        .amount(0)
        .instruction();
    let err = execute_with_payer_err(&mut context, instruction, Some(&owner)).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinRewardsError::NoDepositedTokensToWithdraw as u32)
        )
    );
}

#[tokio::test]
async fn success_with_rewards() {
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
        DEPOSIT_AMOUNT,
    )
    .await;

    // Send rewards to the pool to update rates
    send_rewards_to_pool(&mut context, &holder_rewards_pool, REWARDS_AMOUNT).await;

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
        DEPOSIT_AMOUNT,
        0,
        INITIAL_OWNER_BALANCE - DEPOSIT_AMOUNT,
    )
    .await;

    // Do withdraw
    let instruction = WithdrawBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token)
        .holder_rewards(holder_rewards)
        .token_account(owner_token)
        .mint(mint)
        .owner(owner.pubkey())
        .amount(DEPOSIT_AMOUNT)
        .instruction();
    execute_with_payer(&mut context, instruction, Some(&owner)).await;

    // Assert pool balance is 0 (single depositor withdrews all).
    let pool_token_account = context
        .banks_client
        .get_account(pool_token)
        .await
        .unwrap()
        .unwrap();
    let pool_token_account_balance = TokenAccount::unpack(&pool_token_account.data)
        .unwrap()
        .amount;
    assert_eq!(pool_token_account_balance, 0);

    // Assert owner got all the tokens back to INITIAL_OWNER_BALANCE
    let owner_token_account = context
        .banks_client
        .get_account(owner_token)
        .await
        .unwrap()
        .unwrap();
    let owner_token_account_balance = TokenAccount::unpack(&owner_token_account.data)
        .unwrap()
        .amount;
    assert_eq!(owner_token_account_balance, INITIAL_OWNER_BALANCE);

    // Get owner account lamports
    let owner_lamports = context
        .banks_client
        .get_account(owner.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(owner_lamports, REWARDS_AMOUNT);
}

#[tokio::test]
async fn success_without_rewards() {
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
        1_000_000,
        REWARDS_PER_TOKEN_SCALING_FACTOR.checked_mul(10).unwrap(), /* unaccounted rewards more
                                                                    * then current lamport */
        DEPOSIT_AMOUNT,
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
        DEPOSIT_AMOUNT,
        0,
        INITIAL_OWNER_BALANCE - DEPOSIT_AMOUNT,
    )
    .await;

    // Do wihthdraw
    let instruction = WithdrawBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token)
        .holder_rewards(holder_rewards)
        .token_account(owner_token)
        .mint(mint)
        .owner(owner.pubkey())
        .amount(DEPOSIT_AMOUNT)
        .instruction();
    execute_with_payer(&mut context, instruction, Some(&owner)).await;

    // Assert pool balance is 0 (single depositor withdrews all).
    let pool_token_account = context
        .banks_client
        .get_account(pool_token)
        .await
        .unwrap()
        .unwrap();
    let pool_token_account_balance = TokenAccount::unpack(&pool_token_account.data)
        .unwrap()
        .amount;
    assert_eq!(pool_token_account_balance, 0);

    // Assert owner got all the tokens back to INITIAL_OWNER_BALANCE
    let owner_token_account = context
        .banks_client
        .get_account(owner_token)
        .await
        .unwrap()
        .unwrap();
    let owner_token_account_balance = TokenAccount::unpack(&owner_token_account.data)
        .unwrap()
        .amount;
    assert_eq!(owner_token_account_balance, INITIAL_OWNER_BALANCE);

    // Assert that the pool still have the REWARDS_AMOUNT lamports
    let holder_rewards_pool_account = context
        .banks_client
        .get_account(holder_rewards_pool)
        .await
        .unwrap()
        .unwrap();
    assert!(holder_rewards_pool_account.lamports > 1_000_000);

    // Assert the owner didn't get any lamports (non initialized yet)
    let owner_account = context
        .banks_client
        .get_account(owner.pubkey())
        .await
        .unwrap();
    assert!(owner_account.is_none());
}

#[tokio::test]
async fn success_withdraw_half() {
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
        DEPOSIT_AMOUNT,
    )
    .await;

    // Send rewards to the pool to update rates
    send_rewards_to_pool(&mut context, &holder_rewards_pool, REWARDS_AMOUNT).await;

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
        DEPOSIT_AMOUNT,
        0,
        INITIAL_OWNER_BALANCE - DEPOSIT_AMOUNT,
    )
    .await;

    // Do withdraw
    let instruction = WithdrawBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token)
        .holder_rewards(holder_rewards)
        .token_account(owner_token)
        .mint(mint)
        .owner(owner.pubkey())
        .amount(DEPOSIT_AMOUNT / 2)
        .instruction();
    execute_with_payer(&mut context, instruction, Some(&owner)).await;

    // Assert pool balance is DEPOSIT_AMOUNT / 2 (single depositor withdrews half).
    let pool_token_account = context
        .banks_client
        .get_account(pool_token)
        .await
        .unwrap()
        .unwrap();
    let pool_token_account_balance = TokenAccount::unpack(&pool_token_account.data)
        .unwrap()
        .amount;
    assert_eq!(pool_token_account_balance, DEPOSIT_AMOUNT / 2);

    // Assert owner got half the tokens back
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
        INITIAL_OWNER_BALANCE - DEPOSIT_AMOUNT / 2
    );

    // Get owner account lamports
    let owner_lamports = context
        .banks_client
        .get_account(owner.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(owner_lamports, REWARDS_AMOUNT);
}
