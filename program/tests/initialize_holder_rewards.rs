#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_rewards_program::{
        error::PaladinRewardsError,
        instruction::initialize_holder_rewards,
        state::{get_holder_rewards_address, get_holder_rewards_pool_address, HolderRewards},
    },
    setup::{setup, setup_holder_rewards_pool_account, setup_mint, setup_token_account},
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        instruction::InstructionError,
        pubkey::Pubkey,
        signer::Signer,
        system_program,
        transaction::{Transaction, TransactionError},
    },
    spl_associated_token_account::get_associated_token_address,
};

#[tokio::test]
async fn fail_token_account_invalid_data() {
    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), 0).await;

    // Set up a token account with invalid data.
    {
        context.set_account(
            &token_account,
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![5; 165],
                owner: spl_token_2022::id(),
                ..Account::default()
            }),
        );
    }

    let instruction =
        initialize_holder_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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
async fn fail_token_account_mint_mismatch() {
    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_token_account(
        &mut context,
        &token_account,
        &owner,
        &Pubkey::new_unique(), // Incorrect mint.
        0,
    )
    .await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), 0).await;

    let instruction =
        initialize_holder_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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
            InstructionError::Custom(PaladinRewardsError::TokenAccountMintMismatch as u32)
        )
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
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), 0).await;

    // Set up a holder rewards pool account with incorrect owner.
    {
        context.set_account(
            &holder_rewards_pool,
            &AccountSharedData::new_data(100_000_000, &vec![5; 8], &Pubkey::new_unique()).unwrap(),
        );
    }

    let instruction =
        initialize_holder_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), 0).await;

    let instruction =
        initialize_holder_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), 0).await;

    // Set up a holder rewards pool account with invalid data.
    {
        context.set_account(
            &holder_rewards_pool,
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![5; 17], /* Since this account is all integers, this will always
                                    * succeed if size is correct. */
                owner: paladin_rewards_program::id(),
                ..Account::default()
            }),
        );
    }

    let instruction =
        initialize_holder_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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
async fn fail_holder_rewards_incorrect_address() {
    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = Pubkey::new_unique(); // Incorrect holder reward address.
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), 0).await;

    let instruction =
        initialize_holder_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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
async fn fail_holder_rewards_account_initialized() {
    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), 0).await;

    // Set up an already (arbitrarily) initialized holder rewards account.
    {
        context.set_account(
            &holder_rewards,
            &AccountSharedData::from(Account {
                lamports: 1_000_000_000,
                data: vec![2; 16],
                owner: paladin_rewards_program::id(),
                ..Account::default()
            }),
        );
    }

    let instruction =
        initialize_holder_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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
        TransactionError::InstructionError(0, InstructionError::AccountAlreadyInitialized)
    );
}

#[tokio::test]
async fn success() {
    // Since there's no math involved here, we just need to assert that the
    // new holder account records the current pool values.
    let accumulated_rewards_per_token = 500_000_000;

    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(
        &mut context,
        &holder_rewards_pool,
        0, // Excess lamports (not used here).
        accumulated_rewards_per_token,
    )
    .await;
    setup_token_account(
        &mut context,
        &token_account,
        &owner,
        &mint,
        100, // Token account balance (not used here).
    )
    .await;
    setup_mint(
        &mut context,
        &mint,
        &Pubkey::new_unique(),
        100_000, // Token supply (not used here).
    )
    .await;

    // Fund the holder rewards account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<HolderRewards>());
        context.set_account(
            &holder_rewards,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }

    let instruction =
        initialize_holder_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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

    // Check the holder rewards account.
    let holder_rewards_account = context
        .banks_client
        .get_account(holder_rewards)
        .await
        .unwrap()
        .unwrap();
    let holder_rewards_state = bytemuck::from_bytes::<HolderRewards>(&holder_rewards_account.data);

    assert_eq!(
        holder_rewards_state,
        &HolderRewards::new(
            accumulated_rewards_per_token,
            /* unharvested_rewards */ 0
        ),
    );
}
