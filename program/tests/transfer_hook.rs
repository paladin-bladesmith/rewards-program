//! Most of these test cases are checked by Token-2022, but it doesn't hurt to
//! check them by directly invoking the program's `ExecuteInstruction`.

#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_rewards_program::{
        error::PaladinRewardsError,
        state::{get_holder_rewards_address, get_holder_rewards_pool_address, HolderRewards},
    },
    setup::{
        setup, setup_extra_metas_account, setup_holder_rewards_account,
        setup_holder_rewards_pool_account, setup_mint, setup_token_account,
        setup_token_account_transferring,
    },
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        instruction::{AccountMeta, Instruction, InstructionError},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        transaction::{Transaction, TransactionError},
    },
    spl_associated_token_account::get_associated_token_address,
    spl_transfer_hook_interface::{
        error::TransferHookError, get_extra_account_metas_address,
        offchain::add_extra_account_metas_for_execute,
    },
    test_case::test_case,
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

async fn transfer_with_extra_metas_instruction(
    context: &mut ProgramTestContext,
    source: &Pubkey,
    mint: &Pubkey,
    destination: &Pubkey,
    owner: &Pubkey,
    amount: u64,
    decimals: u8,
) -> Instruction {
    let mut instruction = spl_token_2022::instruction::transfer_checked(
        &spl_token_2022::id(),
        source,
        mint,
        destination,
        owner,
        &[],
        amount,
        decimals,
    )
    .unwrap();

    // The closure required by `add_extra_account_metas_for_execute` is a pain,
    // so just grab the extra metas account ahead of time, since we know our
    // extra metas don't require account data, therefore don't require loading
    // any other accounts.
    let extra_metas_address = get_extra_account_metas_address(mint, &paladin_rewards_program::id());
    let extra_metas_account = context
        .banks_client
        .get_account(extra_metas_address)
        .await
        .unwrap()
        .unwrap();

    add_extra_account_metas_for_execute(
        &mut instruction,
        &paladin_rewards_program::id(),
        source,
        mint,
        destination,
        owner,
        amount,
        |key| {
            let data = if key.eq(&extra_metas_address) {
                Some(extra_metas_account.data.clone())
            } else {
                None
            };
            async move { Ok(data) }
        },
    )
    .await
    .unwrap();

    instruction
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
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![5; 14], /* Since this account is all integers, this will always
                                    * succeed if size is correct. */
                owner: paladin_rewards_program::id(),
                ..Account::default()
            }),
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
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![5; 30], /* Since this account is all integers, this will always
                                    * succeed if size is correct. */
                owner: paladin_rewards_program::id(),
                ..Account::default()
            }),
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
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![5; 165],
                owner: spl_token_2022::id(),
                ..Account::default()
            }),
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
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![5; 34], /* Since this account is all integers, this will always
                                    * succeed if size is correct. */
                owner: paladin_rewards_program::id(),
                ..Account::default()
            }),
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
            &AccountSharedData::from(Account {
                lamports: 100_000_000,
                data: vec![5; 165],
                owner: spl_token_2022::id(),
                ..Account::default()
            }),
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

struct Pool {
    accumulated_rewards_per_token: u128,
}

struct PoolAddresses {
    mint: Pubkey,
    holder_rewards_pool: Pubkey,
}

impl PoolAddresses {
    fn new() -> Self {
        let mint = Pubkey::new_unique();
        let holder_rewards_pool = get_holder_rewards_pool_address(&mint);
        Self {
            mint,
            holder_rewards_pool,
        }
    }
}

struct Holder {
    token_account_balance: u64,
    last_accumulated_rewards_per_token: u128,
    unharvested_rewards: u64,
    expected_unharvested_rewards: u64,
}

struct HolderAddresses {
    owner: Pubkey,
    token_account: Pubkey,
    holder_rewards: Pubkey,
}

impl HolderAddresses {
    fn new(owner: &Pubkey, mint: &Pubkey) -> Self {
        let token_account = get_associated_token_address(owner, mint);
        let holder_rewards = get_holder_rewards_address(&token_account);
        Self {
            owner: *owner,
            token_account,
            holder_rewards,
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn setup_direct_invoke(
    context: &mut ProgramTestContext,
    pool: &Pool,
    pool_addresses: &PoolAddresses,
    source: &Holder,
    source_addresses: &HolderAddresses,
    destination: &Holder,
    destination_addresses: &HolderAddresses,
    transfer_amount: u64,
) {
    let Pool {
        accumulated_rewards_per_token,
    } = pool;
    let PoolAddresses {
        mint,
        holder_rewards_pool,
    } = pool_addresses;
    let Holder {
        token_account_balance: source_token_account_balance,
        last_accumulated_rewards_per_token: source_last_accumulated_rewards_per_token,
        unharvested_rewards: source_unharvested_rewards,
        expected_unharvested_rewards: _,
    } = source;
    let HolderAddresses {
        owner: source_owner,
        token_account: source_token_account,
        holder_rewards: source_holder_rewards,
    } = source_addresses;
    let Holder {
        token_account_balance: destination_token_account_balance,
        last_accumulated_rewards_per_token: destination_last_accumulated_rewards_per_token,
        unharvested_rewards: destination_unharvested_rewards,
        expected_unharvested_rewards: _,
    } = destination;
    let HolderAddresses {
        owner: destination_owner,
        token_account: destination_token_account,
        holder_rewards: destination_holder_rewards,
    } = destination_addresses;

    setup_holder_rewards_pool_account(
        context,
        holder_rewards_pool,
        0, // Excess lamports (unused here).
        *accumulated_rewards_per_token,
    )
    .await;
    setup_holder_rewards_account(
        context,
        source_holder_rewards,
        *source_unharvested_rewards,
        *source_last_accumulated_rewards_per_token,
    )
    .await;
    setup_holder_rewards_account(
        context,
        destination_holder_rewards,
        *destination_unharvested_rewards,
        *destination_last_accumulated_rewards_per_token,
    )
    .await;
    setup_token_account_transferring(
        context,
        source_token_account,
        source_owner,
        mint,
        *source_token_account_balance - transfer_amount, // Post-transfer balance.
    )
    .await;
    setup_token_account_transferring(
        context,
        destination_token_account,
        destination_owner,
        mint,
        *destination_token_account_balance + transfer_amount, // Post-transfer balance.
    )
    .await;
    setup_mint(
        context,
        mint,
        &Pubkey::new_unique(),
        100_000, // Token supply (unused here).
    )
    .await;
}

async fn setup_transfer_hook(
    context: &mut ProgramTestContext,
    pool: &Pool,
    pool_addresses: &PoolAddresses,
    source: &Holder,
    source_addresses: &HolderAddresses,
    destination: &Holder,
    destination_addresses: &HolderAddresses,
) {
    let Pool {
        accumulated_rewards_per_token,
    } = pool;
    let PoolAddresses {
        mint,
        holder_rewards_pool,
    } = pool_addresses;
    let Holder {
        token_account_balance: source_token_account_balance,
        last_accumulated_rewards_per_token: source_last_accumulated_rewards_per_token,
        unharvested_rewards: source_unharvested_rewards,
        expected_unharvested_rewards: _,
    } = source;
    let HolderAddresses {
        owner: source_owner,
        token_account: source_token_account,
        holder_rewards: source_holder_rewards,
    } = source_addresses;
    let Holder {
        token_account_balance: destination_token_account_balance,
        last_accumulated_rewards_per_token: destination_last_accumulated_rewards_per_token,
        unharvested_rewards: destination_unharvested_rewards,
        expected_unharvested_rewards: _,
    } = destination;
    let HolderAddresses {
        owner: destination_owner,
        token_account: destination_token_account,
        holder_rewards: destination_holder_rewards,
    } = destination_addresses;

    setup_extra_metas_account(context, mint).await;
    setup_holder_rewards_pool_account(
        context,
        holder_rewards_pool,
        0, // Excess lamports (unused here).
        *accumulated_rewards_per_token,
    )
    .await;
    setup_holder_rewards_account(
        context,
        source_holder_rewards,
        *source_unharvested_rewards,
        *source_last_accumulated_rewards_per_token,
    )
    .await;
    setup_holder_rewards_account(
        context,
        destination_holder_rewards,
        *destination_unharvested_rewards,
        *destination_last_accumulated_rewards_per_token,
    )
    .await;
    setup_token_account(
        context,
        source_token_account,
        source_owner,
        mint,
        *source_token_account_balance,
    )
    .await;
    setup_token_account(
        context,
        destination_token_account,
        destination_owner,
        mint,
        *destination_token_account_balance,
    )
    .await;
    setup_mint(
        context,
        mint,
        &Pubkey::new_unique(),
        100_000, // Token supply (unused here).
    )
    .await;
}

async fn check_holder_rewards(
    context: &mut ProgramTestContext,
    pool: &Pool,
    holder: &Holder,
    holder_addresses: &HolderAddresses,
) {
    let holder_rewards_account = context
        .banks_client
        .get_account(holder_addresses.holder_rewards)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        bytemuck::from_bytes::<HolderRewards>(&holder_rewards_account.data),
        &HolderRewards::new(
            pool.accumulated_rewards_per_token,
            holder.expected_unharvested_rewards,
        ),
    );
}

#[test_case(
    Pool {
        accumulated_rewards_per_token: 0,
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        expected_unharvested_rewards: 0,
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
        expected_unharvested_rewards: 0,
    };
    "all zeroes, no unharvested rewards"
)]
#[test_case(
    Pool {
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
        unharvested_rewards: 0,
        expected_unharvested_rewards: 0,
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
        unharvested_rewards: 0,
        expected_unharvested_rewards: 0,
    };
    "rate unchanged, no unharvested rewards"
)]
#[test_case(
    Pool {
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 500_000_000, // 0.5 rewards per token.
        unharvested_rewards: 0,
        expected_unharvested_rewards: 50, // (1 - 0.5) * 100 = 50
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 500_000_000, // 0.5 rewards per token.
        unharvested_rewards: 0,
        expected_unharvested_rewards: 50, // (1 - 0.5) * 100 = 50
    };
    "last seen 0.5, current rate 1, diff to unharvested rewards"
)]
#[test_case(
    Pool {
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 250_000_000, // 0.25 rewards per token.
        unharvested_rewards: 0,
        expected_unharvested_rewards: 75, // (1 - 0.25) * 100 = 75
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 750_000_000, // 0.75 rewards per token.
        unharvested_rewards: 0,
        expected_unharvested_rewards: 25, // (1 - 0.75) * 100 = 25
    };
    "source last seen 0.25, dest last seen 0.75, current rate 1, both diffs to unharvested rewards"
)]
#[test_case(
    Pool {
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 250_000_000, // 0.25 rewards per token.
        unharvested_rewards: 100,
        expected_unharvested_rewards: 175, // (1 - 0.25) * 100 + 100 = 175
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 750_000_000, // 0.75 rewards per token.
        unharvested_rewards: 0,
        expected_unharvested_rewards: 25, // (1 - 0.75) * 100 = 25
    };
    "source last seen 0.25 with unharvested, dest last seen 0.75, current rate 1, both diffs to unharvested rewards"
)]
#[test_case(
    Pool {
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 250_000_000, // 0.25 rewards per token.
        unharvested_rewards: 0,
        expected_unharvested_rewards: 75, // (1 - 0.25) * 100 = 75
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 750_000_000, // 0.75 rewards per token.
        unharvested_rewards: 200,
        expected_unharvested_rewards: 225, // (1 - 0.75) * 100 + 200 = 225
    };
    "source last seen 0.25, dest last seen 0.75 with unharvested, current rate 1, both diffs to unharvested rewards"
)]
#[test_case(
    Pool {
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 250_000_000, // 0.25 rewards per token.
        unharvested_rewards: 100,
        expected_unharvested_rewards: 175, // (1 - 0.25) * 100 + 100 = 175
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 750_000_000, // 0.75 rewards per token.
        unharvested_rewards: 200,
        expected_unharvested_rewards: 225, // (1 - 0.75) * 100 + 200 = 225
    };
    "source last seen 0.25 with unharvested, dest last seen 0.75 with unharvested, current rate 1, both diffs to unharvested rewards"
)]
#[tokio::test]
async fn success(pool: Pool, source: Holder, destination: Holder) {
    let source_owner = Keypair::new();
    let destination_owner = Pubkey::new_unique();

    let transfer_amount = 10; // Doesn't matter to our system.

    let pool_addresses = PoolAddresses::new();
    let source_addresses = HolderAddresses::new(&source_owner.pubkey(), &pool_addresses.mint);
    let destination_addresses = HolderAddresses::new(&destination_owner, &pool_addresses.mint);

    // First test directly invoking the program.
    {
        let mut context = setup().start_with_context().await;
        setup_direct_invoke(
            &mut context,
            &pool,
            &pool_addresses,
            &source,
            &source_addresses,
            &destination,
            &destination_addresses,
            transfer_amount,
        )
        .await;

        let instruction = execute_with_extra_metas_instruction(
            &source_addresses.token_account,
            &pool_addresses.mint,
            &destination_addresses.token_account,
            &source_addresses.owner,
            &pool_addresses.holder_rewards_pool,
            &source_addresses.holder_rewards,
            &destination_addresses.holder_rewards,
            transfer_amount,
        );

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

        check_holder_rewards(&mut context, &pool, &source, &source_addresses).await;
        check_holder_rewards(&mut context, &pool, &destination, &destination_addresses).await;
    }

    // Then test transfer hook with Token-2022.
    {
        let mut context = setup().start_with_context().await;
        setup_transfer_hook(
            &mut context,
            &pool,
            &pool_addresses,
            &source,
            &source_addresses,
            &destination,
            &destination_addresses,
        )
        .await;

        let instruction = transfer_with_extra_metas_instruction(
            &mut context,
            &source_addresses.token_account,
            &pool_addresses.mint,
            &destination_addresses.token_account,
            &source_addresses.owner,
            transfer_amount,
            0, // Decimals.
        )
        .await;

        let transaction = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&context.payer.pubkey()),
            &[&context.payer, &source_owner],
            context.last_blockhash,
        );

        context
            .banks_client
            .process_transaction(transaction)
            .await
            .unwrap();

        check_holder_rewards(&mut context, &pool, &source, &source_addresses).await;
        check_holder_rewards(&mut context, &pool, &destination, &destination_addresses).await;
    }
}
