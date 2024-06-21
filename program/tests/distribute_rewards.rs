#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_rewards_program::{instruction::distribute_rewards, state::HolderRewardsPool},
    setup::{setup, setup_holder_rewards_pool_account, setup_system_account},
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        instruction::InstructionError,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        transaction::{Transaction, TransactionError},
    },
};

#[tokio::test]
async fn fail_payer_not_signer() {
    let holder_rewards_pool = Pubkey::new_unique();
    let payer = Keypair::new();
    let amount = 500_000_000_000;

    let mut context = setup().start_with_context().await;

    let mut instruction = distribute_rewards(&payer.pubkey(), &holder_rewards_pool, amount);
    instruction.accounts[0].is_signer = false; // Not signer.

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer], // Missing payer.
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
        TransactionError::InstructionError(0, InstructionError::MissingRequiredSignature)
    );
}

#[tokio::test]
async fn fail_holder_rewards_pool_incorrect_owner() {
    let holder_rewards_pool = Pubkey::new_unique();
    let payer = Keypair::new();
    let amount = 500_000_000_000;

    let mut context = setup().start_with_context().await;
    setup_system_account(&mut context, &payer.pubkey(), amount).await;

    // Set up a holder rewards pool account with incorrect owner.
    {
        context.set_account(
            &holder_rewards_pool,
            &AccountSharedData::new_data(
                100_000_000,
                &vec![0; 8],
                &Pubkey::new_unique(), // Incorrect owner.
            )
            .unwrap(),
        );
    }

    let instruction = distribute_rewards(&payer.pubkey(), &holder_rewards_pool, amount);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &payer],
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
async fn fail_holder_rewards_pool_invalid_data() {
    let holder_rewards_pool = Pubkey::new_unique();
    let payer = Keypair::new();
    let amount = 500_000_000_000;

    let mut context = setup().start_with_context().await;
    setup_system_account(&mut context, &payer.pubkey(), amount).await;

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

    let instruction = distribute_rewards(&payer.pubkey(), &holder_rewards_pool, amount);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &payer],
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

#[allow(clippy::arithmetic_side_effects)]
#[tokio::test]
async fn success() {
    let holder_rewards_pool = Pubkey::new_unique();
    let payer = Keypair::new();
    let amount = 500_000_000_000;

    let mut context = setup().start_with_context().await;
    setup_system_account(&mut context, &payer.pubkey(), amount).await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;

    // For checks later.
    let payer_beginning_lamports = context
        .banks_client
        .get_account(payer.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;

    let instruction = distribute_rewards(&payer.pubkey(), &holder_rewards_pool, amount);

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &payer],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Assert the holder rewards pool's total rewards was updated.
    let holder_rewards_pool_account = context
        .banks_client
        .get_account(holder_rewards_pool)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        bytemuck::from_bytes::<HolderRewardsPool>(&holder_rewards_pool_account.data),
        &HolderRewardsPool {
            total_rewards: amount
        }
    );

    // Assert the pool was debited lamports.
    let rent = context.banks_client.get_rent().await.unwrap();
    let expected_lamports = rent.minimum_balance(std::mem::size_of::<HolderRewardsPool>()) + amount;
    assert_eq!(holder_rewards_pool_account.lamports, expected_lamports);

    // Assert the payer's account balance was debited.
    let payer_resulting_lamports = context
        .banks_client
        .get_account(payer.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(payer_resulting_lamports, payer_beginning_lamports - amount,);
}
