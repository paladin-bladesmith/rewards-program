//! Most of these test cases are checked by Token-2022, but it doesn't hurt to
//! check them by directly invoking the program's `ExecuteInstruction`.

#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_rewards_program::{
        error::PaladinRewardsError,
        state::{get_holder_rewards_address, get_holder_rewards_pool_address},
    },
    setup::{
        setup, setup_holder_rewards_account, setup_holder_rewards_pool_account,
        setup_token_account, setup_token_account_transferring,
    },
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        instruction::{AccountMeta, Instruction, InstructionError},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        transaction::{Transaction, TransactionError},
    },
    spl_associated_token_account::get_associated_token_address,
    spl_transfer_hook_interface::error::TransferHookError,
};

#[allow(clippy::too_many_arguments)]
fn execute_with_extra_metas_instruction(
    source: &Pubkey,
    mint: &Pubkey,
    destination: &Pubkey,
    owner: &Pubkey,
    holder_rewards_pool: &Pubkey,
    source_holder_rewards: &Pubkey,
    destination_holder_rewards: &Pubkey,
    amount: u64,
) -> Instruction {
    spl_transfer_hook_interface::instruction::execute_with_extra_account_metas(
        &paladin_rewards_program::id(),
        source,
        mint,
        destination,
        owner,
        &Pubkey::new_unique(), // (Extra metas) Doesn't matter if we're invoking directly.
        &[
            AccountMeta::new_readonly(*holder_rewards_pool, false),
            AccountMeta::new(*source_holder_rewards, false),
            AccountMeta::new(*destination_holder_rewards, false),
        ],
        amount,
    )
}

#[tokio::test]
async fn fail_holder_rewards_pool_incorrect_owner() {
    let mint = Pubkey::new_unique();
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let source_owner = Keypair::new();
    let source_token_account = get_associated_token_address(&source_owner.pubkey(), &mint);
    let source_holder_rewards = get_holder_rewards_address(&source_token_account);

    let destination_owner = Pubkey::new_unique();
    let destination_token_account = get_associated_token_address(&destination_owner, &mint);
    let destination_holder_rewards = get_holder_rewards_address(&destination_token_account);

    let mut context = setup().start_with_context().await;

    // Set up a holder rewards pool account with incorrect owner.
    {
        context.set_account(
            &holder_rewards_pool,
            &AccountSharedData::new_data(100_000_000, &vec![5; 8], &Pubkey::new_unique()).unwrap(),
        );
    }

    let instruction = execute_with_extra_metas_instruction(
        &source_token_account,
        &mint,
        &destination_token_account,
        &source_owner.pubkey(),
        &holder_rewards_pool,
        &source_holder_rewards,
        &destination_holder_rewards,
        0,
    );

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
    let mint = Pubkey::new_unique();
    let holder_rewards_pool = Pubkey::new_unique(); // Incorrect holder rewards pool address.

    let source_owner = Keypair::new();
    let source_token_account = get_associated_token_address(&source_owner.pubkey(), &mint);
    let source_holder_rewards = get_holder_rewards_address(&source_token_account);

    let destination_owner = Pubkey::new_unique();
    let destination_token_account = get_associated_token_address(&destination_owner, &mint);
    let destination_holder_rewards = get_holder_rewards_address(&destination_token_account);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;

    let instruction = execute_with_extra_metas_instruction(
        &source_token_account,
        &mint,
        &destination_token_account,
        &source_owner.pubkey(),
        &holder_rewards_pool,
        &source_holder_rewards,
        &destination_holder_rewards,
        0,
    );

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
    let mint = Pubkey::new_unique();
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let source_owner = Keypair::new();
    let source_token_account = get_associated_token_address(&source_owner.pubkey(), &mint);
    let source_holder_rewards = get_holder_rewards_address(&source_token_account);

    let destination_owner = Pubkey::new_unique();
    let destination_token_account = get_associated_token_address(&destination_owner, &mint);
    let destination_holder_rewards = get_holder_rewards_address(&destination_token_account);

    let mut context = setup().start_with_context().await;

    // Set up a holder rewards pool account with invalid data.
    {
        context.set_account(
            &holder_rewards_pool,
            &AccountSharedData::new_data(100_000_000, &vec![5; 16], &paladin_rewards_program::id())
                .unwrap(),
        );
    }

    let instruction = execute_with_extra_metas_instruction(
        &source_token_account,
        &mint,
        &destination_token_account,
        &source_owner.pubkey(),
        &holder_rewards_pool,
        &source_holder_rewards,
        &destination_holder_rewards,
        0,
    );

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
async fn fail_source_holder_rewards_incorrect_address() {
    let mint = Pubkey::new_unique();
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let source_owner = Keypair::new();
    let source_token_account = get_associated_token_address(&source_owner.pubkey(), &mint);
    let source_holder_rewards = Pubkey::new_unique(); // Incorrect source holder rewards address.

    let destination_owner = Pubkey::new_unique();
    let destination_token_account = get_associated_token_address(&destination_owner, &mint);
    let destination_holder_rewards = get_holder_rewards_address(&destination_token_account);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_holder_rewards_account(&mut context, &source_holder_rewards, 0, 0).await;

    let instruction = execute_with_extra_metas_instruction(
        &source_token_account,
        &mint,
        &destination_token_account,
        &source_owner.pubkey(),
        &holder_rewards_pool,
        &source_holder_rewards,
        &destination_holder_rewards,
        0,
    );

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
async fn fail_source_holder_rewards_invalid_data() {
    let mint = Pubkey::new_unique();
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let source_owner = Keypair::new();
    let source_token_account = get_associated_token_address(&source_owner.pubkey(), &mint);
    let source_holder_rewards = get_holder_rewards_address(&source_token_account);

    let destination_owner = Pubkey::new_unique();
    let destination_token_account = get_associated_token_address(&destination_owner, &mint);
    let destination_holder_rewards = get_holder_rewards_address(&destination_token_account);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_holder_rewards_account(&mut context, &source_holder_rewards, 0, 0).await;

    // Setup source holder rewards account with invalid data.
    {
        context.set_account(
            &source_holder_rewards,
            &AccountSharedData::new_data(100_000_000, &vec![5; 32], &paladin_rewards_program::id())
                .unwrap(),
        );
    }

    let instruction = execute_with_extra_metas_instruction(
        &source_token_account,
        &mint,
        &destination_token_account,
        &source_owner.pubkey(),
        &holder_rewards_pool,
        &source_holder_rewards,
        &destination_holder_rewards,
        0,
    );

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
async fn fail_source_token_account_invalid_data() {
    let mint = Pubkey::new_unique();
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let source_owner = Keypair::new();
    let source_token_account = get_associated_token_address(&source_owner.pubkey(), &mint);
    let source_holder_rewards = get_holder_rewards_address(&source_token_account);

    let destination_owner = Pubkey::new_unique();
    let destination_token_account = get_associated_token_address(&destination_owner, &mint);
    let destination_holder_rewards = get_holder_rewards_address(&destination_token_account);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_holder_rewards_account(&mut context, &source_holder_rewards, 0, 0).await;

    // Set up source token account with invalid data.
    {
        context.set_account(
            &source_token_account,
            &AccountSharedData::new_data(100_000_000, &vec![5; 165], &spl_token_2022::id())
                .unwrap(),
        );
    }

    let instruction = execute_with_extra_metas_instruction(
        &source_token_account,
        &mint,
        &destination_token_account,
        &source_owner.pubkey(),
        &holder_rewards_pool,
        &source_holder_rewards,
        &destination_holder_rewards,
        0,
    );

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
async fn fail_source_token_account_mint_mismatch() {
    let mint = Pubkey::new_unique();
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let source_owner = Keypair::new();
    let source_token_account = get_associated_token_address(&source_owner.pubkey(), &mint);
    let source_holder_rewards = get_holder_rewards_address(&source_token_account);

    let destination_owner = Pubkey::new_unique();
    let destination_token_account = get_associated_token_address(&destination_owner, &mint);
    let destination_holder_rewards = get_holder_rewards_address(&destination_token_account);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_holder_rewards_account(&mut context, &source_holder_rewards, 0, 0).await;
    setup_token_account_transferring(
        &mut context,
        &source_token_account,
        &source_owner.pubkey(),
        &Pubkey::new_unique(), // Incorrect mint.
        10,
    )
    .await;

    let instruction = execute_with_extra_metas_instruction(
        &source_token_account,
        &mint,
        &destination_token_account,
        &source_owner.pubkey(),
        &holder_rewards_pool,
        &source_holder_rewards,
        &destination_holder_rewards,
        0,
    );

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
async fn fail_source_token_account_not_transferring() {
    let mint = Pubkey::new_unique();
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let source_owner = Keypair::new();
    let source_token_account = get_associated_token_address(&source_owner.pubkey(), &mint);
    let source_holder_rewards = get_holder_rewards_address(&source_token_account);

    let destination_owner = Pubkey::new_unique();
    let destination_token_account = get_associated_token_address(&destination_owner, &mint);
    let destination_holder_rewards = get_holder_rewards_address(&destination_token_account);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_holder_rewards_account(&mut context, &source_holder_rewards, 0, 0).await;
    // Not transferring.
    setup_token_account(
        &mut context,
        &source_token_account,
        &source_owner.pubkey(),
        &mint,
        10,
    )
    .await;

    let instruction = execute_with_extra_metas_instruction(
        &source_token_account,
        &mint,
        &destination_token_account,
        &source_owner.pubkey(),
        &holder_rewards_pool,
        &source_holder_rewards,
        &destination_holder_rewards,
        0,
    );

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
            InstructionError::Custom(TransferHookError::ProgramCalledOutsideOfTransfer as u32)
        )
    );
}

#[tokio::test]
async fn fail_destination_holder_rewards_incorrect_address() {
    let mint = Pubkey::new_unique();
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let source_owner = Keypair::new();
    let source_token_account = get_associated_token_address(&source_owner.pubkey(), &mint);
    let source_holder_rewards = get_holder_rewards_address(&source_token_account);

    let destination_owner = Pubkey::new_unique();
    let destination_token_account = get_associated_token_address(&destination_owner, &mint);
    let destination_holder_rewards = Pubkey::new_unique(); // Incorrect source holder rewards address.

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_holder_rewards_account(&mut context, &source_holder_rewards, 0, 0).await;
    setup_holder_rewards_account(&mut context, &destination_holder_rewards, 0, 0).await;
    setup_token_account_transferring(
        &mut context,
        &source_token_account,
        &source_owner.pubkey(),
        &mint,
        10,
    )
    .await;

    let instruction = execute_with_extra_metas_instruction(
        &source_token_account,
        &mint,
        &destination_token_account,
        &source_owner.pubkey(),
        &holder_rewards_pool,
        &source_holder_rewards,
        &destination_holder_rewards,
        0,
    );

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
async fn fail_destination_holder_rewards_invalid_data() {
    let mint = Pubkey::new_unique();
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let source_owner = Keypair::new();
    let source_token_account = get_associated_token_address(&source_owner.pubkey(), &mint);
    let source_holder_rewards = get_holder_rewards_address(&source_token_account);

    let destination_owner = Pubkey::new_unique();
    let destination_token_account = get_associated_token_address(&destination_owner, &mint);
    let destination_holder_rewards = get_holder_rewards_address(&destination_token_account);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_holder_rewards_account(&mut context, &source_holder_rewards, 0, 0).await;
    setup_token_account_transferring(
        &mut context,
        &source_token_account,
        &source_owner.pubkey(),
        &mint,
        10,
    )
    .await;

    // Setup destination holder rewards account with invalid data.
    {
        context.set_account(
            &destination_holder_rewards,
            &AccountSharedData::new_data(100_000_000, &vec![5; 32], &paladin_rewards_program::id())
                .unwrap(),
        );
    }

    let instruction = execute_with_extra_metas_instruction(
        &source_token_account,
        &mint,
        &destination_token_account,
        &source_owner.pubkey(),
        &holder_rewards_pool,
        &source_holder_rewards,
        &destination_holder_rewards,
        0,
    );

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
async fn fail_destination_token_account_invalid_data() {
    let mint = Pubkey::new_unique();
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let source_owner = Keypair::new();
    let source_token_account = get_associated_token_address(&source_owner.pubkey(), &mint);
    let source_holder_rewards = get_holder_rewards_address(&source_token_account);

    let destination_owner = Pubkey::new_unique();
    let destination_token_account = get_associated_token_address(&destination_owner, &mint);
    let destination_holder_rewards = get_holder_rewards_address(&destination_token_account);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_holder_rewards_account(&mut context, &source_holder_rewards, 0, 0).await;
    setup_holder_rewards_account(&mut context, &destination_holder_rewards, 0, 0).await;
    setup_token_account_transferring(
        &mut context,
        &source_token_account,
        &source_owner.pubkey(),
        &mint,
        10,
    )
    .await;

    // Set up destination token account with invalid data.
    {
        context.set_account(
            &destination_token_account,
            &AccountSharedData::new_data(100_000_000, &vec![5; 165], &spl_token_2022::id())
                .unwrap(),
        );
    }

    let instruction = execute_with_extra_metas_instruction(
        &source_token_account,
        &mint,
        &destination_token_account,
        &source_owner.pubkey(),
        &holder_rewards_pool,
        &source_holder_rewards,
        &destination_holder_rewards,
        0,
    );

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
async fn fail_destination_token_account_mint_mismatch() {
    let mint = Pubkey::new_unique();
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let source_owner = Keypair::new();
    let source_token_account = get_associated_token_address(&source_owner.pubkey(), &mint);
    let source_holder_rewards = get_holder_rewards_address(&source_token_account);

    let destination_owner = Pubkey::new_unique();
    let destination_token_account = get_associated_token_address(&destination_owner, &mint);
    let destination_holder_rewards = get_holder_rewards_address(&destination_token_account);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_holder_rewards_account(&mut context, &source_holder_rewards, 0, 0).await;
    setup_holder_rewards_account(&mut context, &destination_holder_rewards, 0, 0).await;
    setup_token_account_transferring(
        &mut context,
        &source_token_account,
        &source_owner.pubkey(),
        &mint,
        10,
    )
    .await;
    setup_token_account_transferring(
        &mut context,
        &destination_token_account,
        &destination_owner,
        &Pubkey::new_unique(), // Incorrect mint.
        10,
    )
    .await;

    let instruction = execute_with_extra_metas_instruction(
        &source_token_account,
        &mint,
        &destination_token_account,
        &source_owner.pubkey(),
        &holder_rewards_pool,
        &source_holder_rewards,
        &destination_holder_rewards,
        0,
    );

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
async fn fail_destination_token_account_not_transferring() {
    let mint = Pubkey::new_unique();
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let source_owner = Keypair::new();
    let source_token_account = get_associated_token_address(&source_owner.pubkey(), &mint);
    let source_holder_rewards = get_holder_rewards_address(&source_token_account);

    let destination_owner = Pubkey::new_unique();
    let destination_token_account = get_associated_token_address(&destination_owner, &mint);
    let destination_holder_rewards = get_holder_rewards_address(&destination_token_account);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_holder_rewards_account(&mut context, &source_holder_rewards, 0, 0).await;
    setup_holder_rewards_account(&mut context, &destination_holder_rewards, 0, 0).await;
    setup_token_account_transferring(
        &mut context,
        &source_token_account,
        &source_owner.pubkey(),
        &mint,
        10,
    )
    .await;
    // Not transferring.
    setup_token_account(
        &mut context,
        &destination_token_account,
        &destination_owner,
        &mint,
        10,
    )
    .await;

    let instruction = execute_with_extra_metas_instruction(
        &source_token_account,
        &mint,
        &destination_token_account,
        &source_owner.pubkey(),
        &holder_rewards_pool,
        &source_holder_rewards,
        &destination_holder_rewards,
        0,
    );

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
            InstructionError::Custom(TransferHookError::ProgramCalledOutsideOfTransfer as u32)
        )
    );
}
