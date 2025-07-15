#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_rewards_program::{
        error::PaladinRewardsError,
        instruction::close_holder_rewards,
        state::{get_holder_rewards_address, get_holder_rewards_pool_address},
    },
    setup::{
        setup, setup_holder_rewards_account, setup_holder_rewards_pool_account, setup_mint,
        setup_token_account,
    },
    solana_program_test::*,
    solana_sdk::{
        instruction::InstructionError,
        pubkey::Pubkey,
        signer::Signer,
        transaction::{Transaction, TransactionError},
    },
    spl_associated_token_account::get_associated_token_address,
};

#[tokio::test]
async fn owner_can_close_zero_balance() {
    let mint = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    let owner = context.payer.pubkey();
    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account, &paladin_rewards_program::id());
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());

    setup_mint(&mut context, &mint, 0, None).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_holder_rewards_account(&mut context, &holder_rewards, 0, 0).await;

    let holder_rewards_lamports_before = context
        .banks_client
        .get_account(holder_rewards)
        .await
        .unwrap()
        .unwrap()
        .lamports;
    let payer_lamports_before = context
        .banks_client
        .get_account(context.payer.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;

    // Act.
    let ix = close_holder_rewards(
        holder_rewards_pool,
        holder_rewards,
        token_account,
        mint,
        owner,
    );
    let tx =
        Transaction::new_signed_with_payer(&[ix], None, &[&context.payer], context.last_blockhash);
    context.banks_client.process_transaction(tx).await.unwrap();

    // Assert.
    assert!(context
        .banks_client
        .get_account(holder_rewards)
        .await
        .unwrap()
        .is_none());
    let payer_lamports_after = context
        .banks_client
        .get_account(context.payer.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(
        payer_lamports_after - payer_lamports_before + 5000,
        holder_rewards_lamports_before
    );
}

#[tokio::test]
async fn owner_cannot_close_non_zero_balance() {
    let mint = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;
    let owner = context.payer.pubkey();
    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account, &paladin_rewards_program::id());
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());

    setup_mint(&mut context, &mint, 1, None).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 1).await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_holder_rewards_account(&mut context, &holder_rewards, 0, 0).await;

    // Act.
    let ix = close_holder_rewards(
        holder_rewards_pool,
        holder_rewards,
        token_account,
        mint,
        owner,
    );
    let tx =
        Transaction::new_signed_with_payer(&[ix], None, &[&context.payer], context.last_blockhash);
    let err = context
        .banks_client
        .process_transaction(tx)
        .await
        .unwrap_err()
        .unwrap();

    // Assert.
    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinRewardsError::InvalidClosingBalance as u32)
        )
    );
}
