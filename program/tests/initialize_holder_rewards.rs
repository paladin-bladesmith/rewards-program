#![cfg(feature = "test-sbf")]

mod execute_utils;
mod setup;

use {
    crate::execute_utils::{execute_with_payer, execute_with_payer_err},
    paladin_rewards_program::{
        error::PaladinRewardsError,
        state::{
            get_holder_rewards_address, get_holder_rewards_pool_address, HolderRewards,
            HolderRewardsPool,
        },
    },
    paladin_rewards_program_client::instructions::InitializeHolderRewardsBuilder,
    setup::{setup, setup_holder_rewards_pool_account, setup_mint, setup_token_account},
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        instruction::InstructionError,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        system_program,
        transaction::TransactionError,
    },
    spl_associated_token_account::get_associated_token_address,
};

#[tokio::test]
async fn fail_holder_rewards_pool_incorrect_owner() {
    let owner = Keypair::new();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner.pubkey(), &mint);
    let holder_rewards =
        get_holder_rewards_address(&owner.pubkey(), &paladin_rewards_program::id());
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let pool_token_account = get_associated_token_address(&holder_rewards_pool, &mint);

    let mut context = setup().start_with_context().await;
    setup_token_account(
        &mut context,
        &pool_token_account,
        &holder_rewards_pool,
        &mint,
        0,
    )
    .await;
    setup_token_account(&mut context, &token_account, &owner.pubkey(), &mint, 0).await;
    setup_mint(&mut context, &mint, 0, None).await;

    // Set up a holder rewards pool account with incorrect owner.
    {
        context.set_account(
            &holder_rewards_pool,
            &AccountSharedData::new_data(100_000_000, &vec![5; 8], &Pubkey::new_unique()).unwrap(),
        );
    }

    let instruction = InitializeHolderRewardsBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token_account)
        .holder_rewards(holder_rewards)
        .owner(owner.pubkey())
        .mint(mint)
        .instruction();

    let err = execute_with_payer_err(&mut context, instruction, Some(&owner)).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidAccountOwner)
    );
}

#[tokio::test]
async fn fail_holder_rewards_pool_incorrect_address() {
    let owner = Keypair::new();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner.pubkey(), &mint);
    let holder_rewards =
        get_holder_rewards_address(&owner.pubkey(), &paladin_rewards_program::id());
    let holder_rewards_pool = Pubkey::new_unique(); // Incorrect holder rewards pool address.
    let pool_token_account = get_associated_token_address(&holder_rewards_pool, &mint);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner.pubkey(), &mint, 0).await;
    setup_token_account(
        &mut context,
        &pool_token_account,
        &holder_rewards_pool,
        &mint,
        0,
    )
    .await;
    setup_mint(&mut context, &mint, 0, None).await;

    let instruction = InitializeHolderRewardsBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token_account)
        .holder_rewards(holder_rewards)
        .owner(owner.pubkey())
        .mint(mint)
        .instruction();

    let err = execute_with_payer_err(&mut context, instruction, Some(&owner)).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinRewardsError::IncorrectHolderRewardsPoolAddress as u32)
        )
    );
}

#[tokio::test]
async fn fail_holder_rewards_pool_token_incorrect_address() {
    let owner = Keypair::new();
    let mint = Pubkey::new_unique();
    let rand = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner.pubkey(), &mint);
    let holder_rewards =
        get_holder_rewards_address(&owner.pubkey(), &paladin_rewards_program::id());
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let pool_token_account = get_associated_token_address(&rand, &mint); // Incorrect token account address.

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner.pubkey(), &mint, 0).await;
    setup_token_account(&mut context, &pool_token_account, &rand, &mint, 0).await;
    setup_mint(&mut context, &mint, 0, None).await;

    let instruction = InitializeHolderRewardsBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token_account)
        .holder_rewards(holder_rewards)
        .owner(owner.pubkey())
        .mint(mint)
        .instruction();

    let err = execute_with_payer_err(&mut context, instruction, Some(&owner)).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinRewardsError::TokenAccountOwnerMissmatch as u32)
        )
    );
}

#[tokio::test]
async fn fail_holder_rewards_pool_invalid_data() {
    let owner = Keypair::new();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner.pubkey(), &mint);
    let holder_rewards =
        get_holder_rewards_address(&owner.pubkey(), &paladin_rewards_program::id());
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let pool_token_account = get_associated_token_address(&holder_rewards_pool, &mint);

    let mut context = setup().start_with_context().await;
    setup_token_account(
        &mut context,
        &pool_token_account,
        &holder_rewards_pool,
        &mint,
        0,
    )
    .await;
    setup_token_account(&mut context, &token_account, &owner.pubkey(), &mint, 0).await;
    setup_mint(&mut context, &mint, 0, None).await;

    // Set up a holder rewards pool account with invalid data.
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

    let instruction = InitializeHolderRewardsBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token_account)
        .holder_rewards(holder_rewards)
        .owner(owner.pubkey())
        .mint(mint)
        .instruction();

    let err = execute_with_payer_err(&mut context, instruction, Some(&owner)).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(0, InstructionError::InvalidAccountData)
    );
}

#[tokio::test]
async fn fail_holder_rewards_incorrect_address() {
    let owner = Keypair::new();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner.pubkey(), &mint);
    let holder_rewards = Pubkey::new_unique(); // Incorrect holder reward address.
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let pool_token_account = get_associated_token_address(&holder_rewards_pool, &mint);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_token_account(
        &mut context,
        &pool_token_account,
        &holder_rewards_pool,
        &mint,
        0,
    )
    .await;
    setup_token_account(&mut context, &token_account, &owner.pubkey(), &mint, 0).await;
    setup_mint(&mut context, &mint, 0, None).await;

    let instruction = InitializeHolderRewardsBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token_account)
        .holder_rewards(holder_rewards)
        .owner(owner.pubkey())
        .mint(mint)
        .instruction();

    let err = execute_with_payer_err(&mut context, instruction, Some(&owner)).await;

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
    let owner = Keypair::new();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner.pubkey(), &mint);
    let holder_rewards =
        get_holder_rewards_address(&owner.pubkey(), &paladin_rewards_program::id());
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let pool_token_account = get_associated_token_address(&holder_rewards_pool, &mint);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_token_account(
        &mut context,
        &pool_token_account,
        &holder_rewards_pool,
        &mint,
        0,
    )
    .await;
    setup_token_account(&mut context, &token_account, &owner.pubkey(), &mint, 0).await;
    setup_mint(&mut context, &mint, 0, None).await;

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

    let instruction = InitializeHolderRewardsBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token_account)
        .holder_rewards(holder_rewards)
        .owner(owner.pubkey())
        .mint(mint)
        .instruction();

    let err = execute_with_payer_err(&mut context, instruction, Some(&owner)).await;

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

    let owner = Keypair::new();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner.pubkey(), &mint);
    let holder_rewards =
        get_holder_rewards_address(&owner.pubkey(), &paladin_rewards_program::id());
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let pool_token_account = get_associated_token_address(&holder_rewards_pool, &mint);

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
        &pool_token_account,
        &holder_rewards_pool,
        &mint,
        0,
    )
    .await;

    setup_token_account(
        &mut context,
        &token_account,
        &owner.pubkey(),
        &mint,
        100, // Token account balance (not used here).
    )
    .await;
    setup_mint(
        &mut context,
        &mint,
        100_000, // Token supply (not used here).
        None,
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

    let instruction = InitializeHolderRewardsBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token_account)
        .holder_rewards(holder_rewards)
        .owner(owner.pubkey())
        .mint(mint)
        .instruction();

    execute_with_payer(&mut context, instruction, Some(&owner)).await;

    // Assert - Check the holder rewards account.
    let holder_rewards_account = context
        .banks_client
        .get_account(holder_rewards)
        .await
        .unwrap()
        .unwrap();
    let holder_rewards_state = bytemuck::from_bytes::<HolderRewards>(&holder_rewards_account.data);
    assert_eq!(
        holder_rewards_state,
        &HolderRewards {
            last_accumulated_rewards_per_token: accumulated_rewards_per_token,
            deposited: 0,
            _padding: 0,
        }
    );

    // Assert - Eligible tokens is updated.
    let holder_rewards_pool_account = context
        .banks_client
        .get_account(holder_rewards_pool)
        .await
        .unwrap()
        .unwrap();
    let holder_rewards_pool_state =
        bytemuck::from_bytes::<HolderRewardsPool>(&holder_rewards_pool_account.data);
    assert_eq!(
        holder_rewards_pool_state,
        &HolderRewardsPool {
            accumulated_rewards_per_token,
            lamports_last: holder_rewards_pool_account.lamports,
            _padding: 0,
        }
    );
}
