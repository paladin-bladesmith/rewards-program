#![cfg(feature = "test-sbf")]

mod execute_utils;
mod setup;

use {
    crate::{
        execute_utils::{execute_with_payer, execute_with_payer_err},
        setup::{setup_holder_rewards_account, setup_token_account},
    },
    paladin_rewards_program::{
        error::PaladinRewardsError,
        state::{
            get_holder_rewards_address, get_holder_rewards_pool_address, HolderRewards,
            HolderRewardsPool,
        },
    },
    paladin_rewards_program_client::instructions::InitializeHolderRewardsPoolBuilder,
    setup::{setup, setup_mint},
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        instruction::InstructionError,
        pubkey::Pubkey,
        system_program,
        transaction::TransactionError,
    },
    spl_associated_token_account::get_associated_token_address,
};

#[tokio::test]
async fn fail_mint_invalid_data() {
    let mint = Pubkey::new_unique();

    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let pool_token_account = get_associated_token_address(&holder_rewards_pool, &mint);

    let mut context = setup().start_with_context().await;

    // Set up a mint with invalid data.
    {
        context.set_account(
            &mint,
            &AccountSharedData::new_data(100_000_000, &vec![5; 165], &spl_token::id()).unwrap(),
        );
    }

    let vault_pda = Pubkey::default();
    let vault_holder_rewards =
        get_holder_rewards_address(&vault_pda, &paladin_rewards_program::id());
    // Fund the holder rewards account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<HolderRewards>());
        context.set_account(
            &vault_holder_rewards,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }

    let instruction = InitializeHolderRewardsPoolBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token_account)
        .mint(mint)
        .stake_vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .duna_document_hash([1; 32])
        .instruction();
    let err = execute_with_payer_err(&mut context, instruction, None).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidAccountData)
    );
}

#[tokio::test]
async fn fail_holder_rewards_pool_incorrect_address() {
    let mint = Pubkey::new_unique();

    let holder_rewards_pool = Pubkey::new_unique(); // Incorrect holder rewards pool address.
    let pool_token_account = get_associated_token_address(&holder_rewards_pool, &mint);

    let mut context = setup().start_with_context().await;
    setup_mint(&mut context, &mint, 0, None).await;
    setup_token_account(
        &mut context,
        &pool_token_account,
        &holder_rewards_pool,
        &mint,
        0,
    )
    .await;

    let vault_pda = Pubkey::default();
    let vault_holder_rewards =
        get_holder_rewards_address(&vault_pda, &paladin_rewards_program::id());
    // Fund the holder rewards account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<HolderRewards>());
        context.set_account(
            &vault_holder_rewards,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }
    let instruction = InitializeHolderRewardsPoolBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token_account)
        .mint(mint)
        .stake_vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .duna_document_hash([1; 32])
        .instruction();
    let err = execute_with_payer_err(&mut context, instruction, None).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinRewardsError::IncorrectHolderRewardsPoolAddress as u32)
        )
    );
}

#[tokio::test]
async fn fail_holder_rewards_pool_incorrect_token_address() {
    let mint = Pubkey::new_unique();
    let rand = Pubkey::new_unique();

    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let pool_token_account = get_associated_token_address(&rand, &mint); // Incorrect token account address.

    let mut context = setup().start_with_context().await;
    setup_mint(&mut context, &mint, 0, None).await;
    setup_token_account(&mut context, &pool_token_account, &rand, &mint, 0).await;

    let vault_pda = Pubkey::default();
    let vault_holder_rewards =
        get_holder_rewards_address(&vault_pda, &paladin_rewards_program::id());
    // Fund the holder rewards account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<HolderRewards>());
        context.set_account(
            &vault_holder_rewards,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }
    let instruction = InitializeHolderRewardsPoolBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token_account)
        .mint(mint)
        .stake_vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .duna_document_hash([1; 32])
        .instruction();
    let err = execute_with_payer_err(&mut context, instruction, None).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinRewardsError::TokenAccountOwnerMissmatch as u32)
        )
    );
}

#[tokio::test]
async fn fail_holder_rewards_pool_account_initialized() {
    let mint = Pubkey::new_unique();

    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let pool_token_account = get_associated_token_address(&holder_rewards_pool, &mint);

    let mut context = setup().start_with_context().await;
    setup_mint(&mut context, &mint, 0, None).await;
    setup_token_account(
        &mut context,
        &pool_token_account,
        &holder_rewards_pool,
        &mint,
        0,
    )
    .await;

    // Set up an already (arbitrarily) initialized holder rewards pool account.
    {
        context.set_account(
            &holder_rewards_pool,
            &AccountSharedData::from(Account {
                lamports: 1_000_000_000,
                data: vec![2; 45],
                owner: paladin_rewards_program::id(),
                ..Account::default()
            }),
        );
    }

    let vault_pda = Pubkey::default();
    let vault_holder_rewards =
        get_holder_rewards_address(&vault_pda, &paladin_rewards_program::id());
    // Fund the holder rewards account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<HolderRewards>());
        context.set_account(
            &vault_holder_rewards,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }

    let instruction = InitializeHolderRewardsPoolBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token_account)
        .mint(mint)
        .stake_vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .duna_document_hash([1; 32])
        .instruction();
    let err = execute_with_payer_err(&mut context, instruction, None).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::AccountAlreadyInitialized)
    );
}

#[tokio::test]
async fn fail_incorrect_vault_holder_rewards_address() {
    let mint = Pubkey::new_unique();

    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let pool_token_account = get_associated_token_address(&holder_rewards_pool, &mint);

    let mut context = setup().start_with_context().await;
    setup_mint(&mut context, &mint, 0, None).await;
    setup_token_account(
        &mut context,
        &pool_token_account,
        &holder_rewards_pool,
        &mint,
        0,
    )
    .await;
    // Fund the holder rewards pool account.
    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(std::mem::size_of::<HolderRewardsPool>());
    context.set_account(
        &holder_rewards_pool,
        &AccountSharedData::new(lamports, 0, &system_program::id()),
    );

    let vault_pda = Pubkey::default();
    let vault_holder_rewards =
        get_holder_rewards_address(&Pubkey::new_unique(), &paladin_rewards_program::id());
    // Fund the holder rewards account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<HolderRewards>());
        context.set_account(
            &vault_holder_rewards,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }

    let instruction = InitializeHolderRewardsPoolBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token_account)
        .mint(mint)
        .stake_vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .duna_document_hash([1; 32])
        .instruction();
    let err = execute_with_payer_err(&mut context, instruction, None).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinRewardsError::IncorrectHolderRewardsAddress as u32)
        )
    );
}

#[tokio::test]
async fn fail_initialized_vault_holder_rewards_address() {
    let mint = Pubkey::new_unique();

    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let pool_token_account = get_associated_token_address(&holder_rewards_pool, &mint);

    let mut context = setup().start_with_context().await;
    setup_mint(&mut context, &mint, 0, None).await;
    setup_token_account(
        &mut context,
        &pool_token_account,
        &holder_rewards_pool,
        &mint,
        0,
    )
    .await;
    // Fund the holder rewards pool account.
    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(std::mem::size_of::<HolderRewardsPool>());
    context.set_account(
        &holder_rewards_pool,
        &AccountSharedData::new(lamports, 0, &system_program::id()),
    );

    let vault_pda = Pubkey::default();
    let vault_holder_rewards =
        get_holder_rewards_address(&vault_pda, &paladin_rewards_program::id());
    // Fund the holder rewards account.
    // Set vault holer rewards account
    setup_holder_rewards_account(&mut context, &vault_holder_rewards, 0, 0).await;

    let instruction = InitializeHolderRewardsPoolBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token_account)
        .mint(mint)
        .stake_vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .duna_document_hash([1; 32])
        .instruction();
    let err = execute_with_payer_err(&mut context, instruction, None).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::AccountAlreadyInitialized)
    );
}

#[tokio::test]
async fn success() {
    let mint = Pubkey::new_unique();

    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let pool_token_account = get_associated_token_address(&holder_rewards_pool, &mint);

    let mut context = setup().start_with_context().await;
    let rent = context.banks_client.get_rent().await.unwrap();
    setup_mint(&mut context, &mint, 0, None).await;
    setup_token_account(
        &mut context,
        &pool_token_account,
        &holder_rewards_pool,
        &mint,
        0,
    )
    .await;

    // Fund the holder rewards pool account.
    let lamports = rent.minimum_balance(std::mem::size_of::<HolderRewardsPool>());
    context.set_account(
        &holder_rewards_pool,
        &AccountSharedData::new(lamports, 0, &system_program::id()),
    );

    let vault_pda = Pubkey::default();
    let vault_holder_rewards =
        get_holder_rewards_address(&vault_pda, &paladin_rewards_program::id());
    // Fund the holder rewards account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<HolderRewards>());
        context.set_account(
            &vault_holder_rewards,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }

    let instruction = InitializeHolderRewardsPoolBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token_account)
        .mint(mint)
        .stake_vault_pda(vault_pda)
        .vault_holder_rewards(vault_holder_rewards)
        .duna_document_hash([1; 32])
        .instruction();
    execute_with_payer(&mut context, instruction, None).await;

    // Check the holder rewards pool account.
    let holder_rewards_pool_account = context
        .banks_client
        .get_account(holder_rewards_pool)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        bytemuck::from_bytes::<HolderRewardsPool>(&holder_rewards_pool_account.data),
        &HolderRewardsPool {
            accumulated_rewards_per_token: 0,
            lamports_last: rent.minimum_balance(HolderRewardsPool::LEN),
            duna_document_hash: [1; 32],
            _padding: 0,
        }
    );

    // Confirm stkae program vault holder rewards was initialized
    let vault_holder_rewards_account = context
        .banks_client
        .get_account(vault_holder_rewards)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        bytemuck::from_bytes::<HolderRewards>(&vault_holder_rewards_account.data),
        &HolderRewards {
            last_accumulated_rewards_per_token: 0,
            deposited: 0,
            _padding: 0,
        }
    );
}
