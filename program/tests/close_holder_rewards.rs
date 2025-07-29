#![cfg(feature = "test-sbf")]

mod execute_utils;
mod setup;

use {
    crate::{
        execute_utils::{execute_with_payer, execute_with_payer_err},
        setup::{
            setup_holder_rewards_account_with_token_account,
            setup_holder_rewards_pool_account_with_token_account, DEPOSIT_AMOUNT,
            INITIAL_OWNER_BALANCE,
        },
    },
    paladin_rewards_program::{
        error::PaladinRewardsError,
        processor::REWARDS_PER_TOKEN_SCALING_FACTOR,
        state::{get_holder_rewards_address, get_holder_rewards_pool_address, HolderRewards},
    },
    paladin_rewards_program_client::instructions::CloseHolderRewardsBuilder,
    setup::setup,
    solana_program_test::*,
    solana_sdk::{
        instruction::InstructionError, pubkey::Pubkey, signature::Keypair, signer::Signer,
        transaction::TransactionError,
    },
    spl_associated_token_account::get_associated_token_address,
};

#[tokio::test]
async fn fail_pending_rewards() {
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
        REWARDS_PER_TOKEN_SCALING_FACTOR.checked_mul(1).unwrap(), // accumulated per token
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
        INITIAL_OWNER_BALANCE,
    )
    .await;

    let instruction = CloseHolderRewardsBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token)
        .holder_rewards(holder_rewards)
        .mint(mint)
        .owner(owner.pubkey())
        .instruction();
    let err = execute_with_payer_err(&mut context, instruction, Some(&owner)).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinRewardsError::CloseWithUnclaimedRewards as u32)
        )
    );
}

#[tokio::test]
async fn fail_tokens_still_deposited() {
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
        DEPOSIT_AMOUNT,
        0,
        INITIAL_OWNER_BALANCE,
    )
    .await;

    let instruction = CloseHolderRewardsBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token)
        .holder_rewards(holder_rewards)
        .mint(mint)
        .owner(owner.pubkey())
        .instruction();
    let err = execute_with_payer_err(&mut context, instruction, Some(&owner)).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinRewardsError::CloseWithDepositedTokens as u32)
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

    let instruction = CloseHolderRewardsBuilder::new()
        .holder_rewards_pool(holder_rewards_pool)
        .holder_rewards_pool_token_account(pool_token)
        .holder_rewards(holder_rewards)
        .mint(mint)
        .owner(owner.pubkey())
        .instruction();
    execute_with_payer(&mut context, instruction, Some(&owner)).await;

    // Assert that the owner got the rent lamprots
    let owner_lamports = context
        .banks_client
        .get_account(owner.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    let rent_amount = context
        .banks_client
        .get_rent()
        .await
        .unwrap()
        .minimum_balance(HolderRewards::LEN);
    assert_eq!(owner_lamports, rent_amount);
}

// #[tokio::test]
// async fn owner_can_close_zero_balance() {
//     let mint = Pubkey::new_unique();

//     let mut context = setup().start_with_context().await;
//     let owner = context.payer.pubkey();
//     let token_account = get_associated_token_address(&owner, &mint);
//     let holder_rewards = get_holder_rewards_address(&token_account,
// &paladin_rewards_program::id());     let holder_rewards_pool =
//         get_holder_rewards_pool_address(&mint,
// &paladin_rewards_program::id());

//     setup_mint(&mut context, &mint, 0, None).await;
//     setup_token_account(&mut context, &token_account, &owner, &mint,
// 0).await;     setup_holder_rewards_pool_account(&mut context,
// &holder_rewards_pool, 0, 0).await;     setup_holder_rewards_account(&mut
// context, &holder_rewards, 0, 0).await;

//     let holder_rewards_lamports_before = context
//         .banks_client
//         .get_account(holder_rewards)
//         .await
//         .unwrap()
//         .unwrap()
//         .lamports;
//     let payer_lamports_before = context
//         .banks_client
//         .get_account(context.payer.pubkey())
//         .await
//         .unwrap()
//         .unwrap()
//         .lamports;

//     // Act.
//     let ix = close_holder_rewards(
//         holder_rewards_pool,
//         holder_rewards,
//         token_account,
//         mint,
//         owner,
//     );
//     let tx =
//         Transaction::new_signed_with_payer(&[ix], None, &[&context.payer],
// context.last_blockhash);     context.banks_client.process_transaction(tx).
// await.unwrap();

//     // Assert.
//     assert!(context
//         .banks_client
//         .get_account(holder_rewards)
//         .await
//         .unwrap()
//         .is_none());
//     let payer_lamports_after = context
//         .banks_client
//         .get_account(context.payer.pubkey())
//         .await
//         .unwrap()
//         .unwrap()
//         .lamports;
//     assert_eq!(
//         payer_lamports_after - payer_lamports_before + 5000,
//         holder_rewards_lamports_before
//     );
// }

// #[tokio::test]
// async fn owner_cannot_close_non_zero_balance() {
//     let mint = Pubkey::new_unique();

//     let mut context = setup().start_with_context().await;
//     let owner = context.payer.pubkey();
//     let token_account = get_associated_token_address(&owner, &mint);
//     let holder_rewards = get_holder_rewards_address(&token_account,
// &paladin_rewards_program::id());     let holder_rewards_pool =
//         get_holder_rewards_pool_address(&mint,
// &paladin_rewards_program::id());

//     setup_mint(&mut context, &mint, 1, None).await;
//     setup_token_account(&mut context, &token_account, &owner, &mint,
// 1).await;     setup_holder_rewards_pool_account(&mut context,
// &holder_rewards_pool, 0, 0).await;     setup_holder_rewards_account(&mut
// context, &holder_rewards, 0, 0).await;

//     // Act.
//     let ix = close_holder_rewards(
//         holder_rewards_pool,
//         holder_rewards,
//         token_account,
//         mint,
//         owner,
//     );
//     let tx =
//         Transaction::new_signed_with_payer(&[ix], None, &[&context.payer],
// context.last_blockhash);     let err = context
//         .banks_client
//         .process_transaction(tx)
//         .await
//         .unwrap_err()
//         .unwrap();

//     // Assert.
//     assert_eq!(
//         err,
//         TransactionError::InstructionError(
//             0,
//
// InstructionError::Custom(PaladinRewardsError::InvalidClosingBalance as u32)
//         )
//     );
// }
