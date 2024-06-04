#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_rewards_program::{
        error::PaladinRewardsError,
        instruction::harvest_rewards,
        state::{get_holder_rewards_address, get_holder_rewards_pool_address, HolderRewards},
    },
    setup::{
        setup, setup_holder_rewards_account, setup_holder_rewards_pool_account, setup_token_account,
    },
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        instruction::InstructionError,
        pubkey::Pubkey,
        signer::Signer,
        system_program,
        transaction::{Transaction, TransactionError},
    },
    spl_associated_token_account::get_associated_token_address,
};

#[tokio::test]
async fn fail_token_account_incorrect_owner() {
    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;

    // Setup token account with incorrect owner.
    {
        context.set_account(
            &token_account,
            &AccountSharedData::new_data(100_000_000, &vec![5; 165], &system_program::id())
                .unwrap(),
        );
    }

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();

    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidAccountOwner)
    );
}

#[tokio::test]
async fn fail_token_account_invalid_data() {
    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;

    // Setup token account with invalid data.
    {
        context.set_account(
            &token_account,
            &AccountSharedData::new_data(100_000_000, &vec![5; 165], &spl_token_2022::id())
                .unwrap(),
        );
    }

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();

    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidAccountData)
    );
}

#[tokio::test]
async fn fail_holder_rewards_incorrect_owner() {
    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;

    // Setup holder rewards account with incorrect owner.
    {
        context.set_account(
            &holder_rewards,
            &AccountSharedData::new_data(100_000_000, &vec![5; 165], &system_program::id())
                .unwrap(),
        );
    }

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();

    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidAccountOwner)
    );
}

#[tokio::test]
async fn fail_holder_rewards_incorrect_address() {
    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = Pubkey::new_unique(); // Incorrect holder rewards address.
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_account(&mut context, &holder_rewards, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinRewardsError::IncorrectHolderRewardsAddress as u32)
        )
    );
}

#[tokio::test]
async fn fail_holder_rewards_invalid_data() {
    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;

    // Setup holder rewards account with invalid data.
    {
        context.set_account(
            &holder_rewards,
            &AccountSharedData::new_data(
                100_000_000,
                &vec![5; 165],
                &paladin_rewards_program::id(),
            )
            .unwrap(),
        );
    }

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();

    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidAccountData)
    );
}

#[tokio::test]
async fn fail_holder_rewards_pool_incorrect_owner() {
    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_account(&mut context, &holder_rewards, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;

    // Setup holder rewards pool account with incorrect owner.
    {
        context.set_account(
            &holder_rewards_pool,
            &AccountSharedData::new_data(100_000_000, &vec![5; 165], &system_program::id())
                .unwrap(),
        );
    }

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();

    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidAccountOwner)
    );
}

#[tokio::test]
async fn fail_holder_rewards_pool_incorrect_address() {
    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = Pubkey::new_unique(); // Incorrect holder rewards pool address.

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0).await;
    setup_holder_rewards_account(&mut context, &holder_rewards, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinRewardsError::IncorrectHolderRewardsPoolAddress as u32)
        )
    );
}

#[tokio::test]
async fn fail_holder_rewards_pool_invalid_data() {
    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_account(&mut context, &holder_rewards, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;

    // Setup holder rewards pool account with invalid data.
    {
        context.set_account(
            &holder_rewards_pool,
            &AccountSharedData::new_data(
                100_000_000,
                &vec![5; 165],
                &paladin_rewards_program::id(),
            )
            .unwrap(),
        );
    }

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();

    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidAccountData)
    );
}

#[tokio::test]
async fn fail_holder_rewards_pool_insufficient_lamports() {
    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(
        &mut context,
        &holder_rewards_pool,
        0, // No excess lamports.
    )
    .await;
    setup_holder_rewards_account(
        &mut context,
        &holder_rewards,
        100_000_000, // Unharvested rewards.
        0,
    )
    .await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    let err = context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap_err()
        .unwrap();

    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InsufficientFunds)
    );
}

#[tokio::test]
async fn success() {
    let total_rewards = 100_000_000_000;
    let unharvested_rewards = 100_000_000;

    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, total_rewards).await;
    setup_holder_rewards_account(
        &mut context,
        &holder_rewards,
        unharvested_rewards,
        total_rewards,
    )
    .await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;

    // For checks later.
    let pool_beginning_lamports = context
        .banks_client
        .get_account(holder_rewards_pool)
        .await
        .unwrap()
        .unwrap()
        .lamports;
    let token_account_beginning_lamports = context
        .banks_client
        .get_account(token_account)
        .await
        .unwrap()
        .unwrap()
        .lamports;

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Assert the holder rewards account's unharvested rewards was updated.
    let holder_rewards_account = context
        .banks_client
        .get_account(holder_rewards)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        bytemuck::from_bytes::<HolderRewards>(&holder_rewards_account.data),
        &HolderRewards {
            last_seen_total_rewards: total_rewards,
            unharvested_rewards: 0,
        }
    );

    // Assert the holder rewards pool's balance was debited.
    let pool_resulting_lamports = context
        .banks_client
        .get_account(holder_rewards_pool)
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(
        pool_resulting_lamports,
        pool_beginning_lamports - unharvested_rewards,
    );

    // Assert the token account's balance was credited.
    let token_account_resulting_lamports = context
        .banks_client
        .get_account(token_account)
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(
        token_account_resulting_lamports,
        token_account_beginning_lamports + unharvested_rewards,
    );
}
