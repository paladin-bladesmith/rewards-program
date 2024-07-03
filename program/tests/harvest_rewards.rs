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
    test_case::test_case,
};

#[tokio::test]
async fn fail_token_account_invalid_data() {
    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account, &paladin_rewards_program::id());
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;

    // Setup token account with invalid data.
    {
        context.set_account(
            &token_account,
            &AccountSharedData::new_data(100_000_000, &vec![5; 165], &spl_token_2022::id())
                .unwrap(),
        );
    }

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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
    let holder_rewards = get_holder_rewards_address(&token_account, &paladin_rewards_program::id());
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;
    setup_token_account(
        &mut context,
        &token_account,
        &owner,
        &Pubkey::new_unique(), // Incorrect mint.
        0,
    )
    .await;

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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
    let holder_rewards = get_holder_rewards_address(&token_account, &paladin_rewards_program::id());
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_account(&mut context, &holder_rewards, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;

    // Setup holder rewards pool account with incorrect owner.
    {
        context.set_account(
            &holder_rewards_pool,
            &AccountSharedData::new_data(100_000_000, &vec![5; 8], &system_program::id()).unwrap(),
        );
    }

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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
    let holder_rewards = get_holder_rewards_address(&token_account, &paladin_rewards_program::id());
    let holder_rewards_pool = Pubkey::new_unique(); // Incorrect holder rewards pool address.

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_holder_rewards_account(&mut context, &holder_rewards, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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
    let holder_rewards = get_holder_rewards_address(&token_account, &paladin_rewards_program::id());
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;
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

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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
    let holder_rewards = get_holder_rewards_address(&token_account, &paladin_rewards_program::id());
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;

    // Setup holder rewards account with incorrect owner.
    {
        context.set_account(
            &holder_rewards,
            &AccountSharedData::new_data(100_000_000, &vec![5; 16], &system_program::id()).unwrap(),
        );
    }

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_holder_rewards_account(&mut context, &holder_rewards, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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
    let holder_rewards = get_holder_rewards_address(&token_account, &paladin_rewards_program::id());
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
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

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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

struct Pool {
    excess_lamports: u64,
    accumulated_rewards_per_token: u128,
}

struct Holder {
    token_account_balance: u64,
    last_accumulated_rewards_per_token: u128,
    unharvested_rewards: u64,
}

#[test_case(
    Pool {
        excess_lamports: 0,
        accumulated_rewards_per_token: 0,
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
    },
    0,
    0;
    "All zeroes, no rewards"
)]
#[test_case(
    Pool {
        excess_lamports: 1_000_000,
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
        unharvested_rewards: 0,
    },
    0,
    0;
    "Last harvested 1.0 rate, rate unchanged, no rewards"
)]
#[test_case(
    Pool {
        excess_lamports: 1_000_000,
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 100,
        last_accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
        unharvested_rewards: 500_000,
    },
    500_000, // Unharvested.
    0;
    "Last harvested 1.0 rate, rate unchanged, some unharvested, receive unharvested"
)]
#[test_case(
    Pool {
        excess_lamports: 50_000,
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 100_000,
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
    },
    50_000, // Pool excess.
    50_000; // Remainder.
    "No last harvested rate, eligible for 1 rate, pool is underfunded, receive pool excess"
)]
#[test_case(
    Pool {
        excess_lamports: 1_000_000,
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 10_000,
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 0,
    },
    10_000,
    0;
    "No last harvested rate, eligible for 1 rate, pool has enough, receive share"
)]
#[test_case(
    Pool {
        excess_lamports: 1_000_000,
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 10_000,
        last_accumulated_rewards_per_token: 0,
        unharvested_rewards: 10_000,
    },
    20_000, // 10_000 share + 10_000 unharvested
    0;
    "No last harvested rate, some unharvested, eligible for 1 rate, pool has enough, receive share + unharvested"
)]
#[test_case(
    Pool {
        excess_lamports: 10_000,
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 10_000,
        last_accumulated_rewards_per_token: 500_000_000, // 0.5 rewards per token.
        unharvested_rewards: 0,
    },
    5_000, // (1 - 0.5) * 10_000
    0;
    "Last harvested 0.5 rate, eligible for 0.5 rate, pool has enough, receive share"
)]
#[test_case(
    Pool {
        excess_lamports: 10_000,
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 10_000,
        last_accumulated_rewards_per_token: 500_000_000, // 0.5 rewards per token.
        unharvested_rewards: 1_000,
    },
    6_000, // (1 - 0.5) * 10_000 = 5_000 share + 1_000 unharvested
    0;
    "Last harvested 0.5 rate, some unharvested, eligible for 0.5 rate, pool has enough, receive share + unharvested"
)]
#[test_case(
    Pool {
        excess_lamports: 10_000,
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 10_000,
        last_accumulated_rewards_per_token: 500_000_000, // 0.5 rewards per token.
        unharvested_rewards: 8_000,
    },
    10_000, // Pool excess.
    3_000; // 10_000 pool excess - [(1 - 0.5) * 10_000 = 5_000 share + 8_000 unharvested]
    "Last harvested 0.5 rate, some unharvested, eligible for 0.5 rate, pool underfunded, receive pool excess"
)]
#[test_case(
    Pool {
        excess_lamports: 10_000,
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 10_000,
        last_accumulated_rewards_per_token: 250_000_000, // 0.25 rewards per token.
        unharvested_rewards: 0,
    },
    7_500, // (1 - 0.25) * 10_000
    0;
    "Last harvested 0.25 rate, eligible for 0.75 rate, pool has enough, receive share"
)]
#[test_case(
    Pool {
        excess_lamports: 10_000,
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 10_000,
        last_accumulated_rewards_per_token: 250_000_000, // 0.25 rewards per token.
        unharvested_rewards: 1_000,
    },
    8_500, // (1 - 0.25) * 10_000 = 7_500 share + 1_000 unharvested
    0;
    "Last harvested 0.25 rate, some unharvested, eligible for 0.75 rate, pool has enough, receive share + unharvested"
)]
#[test_case(
    Pool {
        excess_lamports: 10_000,
        accumulated_rewards_per_token: 1_000_000_000, // 1 reward per token.
    },
    Holder {
        token_account_balance: 10_000,
        last_accumulated_rewards_per_token: 250_000_000, // 0.25 rewards per token.
        unharvested_rewards: 4_000,
    },
    10_000, // Pool excess.
    1_500; // 10_000 pool excess - [(1 - 0.25) * 10_000 = 7_500 share + 4_000 unharvested]
    "Last harvested 0.25 rate, some unharvested, eligible for 0.75 rate, pool underfunded, receive pool excess"
)]
#[tokio::test]
async fn success(
    pool: Pool,
    holder: Holder,
    expected_harvested_rewards: u64,
    expected_unharvested_rewards: u64,
) {
    let Pool {
        excess_lamports,
        accumulated_rewards_per_token,
    } = pool;

    let Holder {
        token_account_balance,
        last_accumulated_rewards_per_token,
        unharvested_rewards,
    } = holder;

    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account, &paladin_rewards_program::id());
    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(
        &mut context,
        &holder_rewards_pool,
        excess_lamports,
        accumulated_rewards_per_token,
    )
    .await;
    setup_holder_rewards_account(
        &mut context,
        &holder_rewards,
        unharvested_rewards,
        last_accumulated_rewards_per_token,
    )
    .await;
    setup_token_account(
        &mut context,
        &token_account,
        &owner,
        &mint,
        token_account_balance,
    )
    .await;

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

    let instruction = harvest_rewards(&holder_rewards_pool, &holder_rewards, &token_account, &mint);

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

    // Assert the holder rewards account state was updated.
    let holder_rewards_account = context
        .banks_client
        .get_account(holder_rewards)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        bytemuck::from_bytes::<HolderRewards>(&holder_rewards_account.data),
        &HolderRewards::new(accumulated_rewards_per_token, expected_unharvested_rewards),
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
        pool_beginning_lamports.saturating_sub(expected_harvested_rewards),
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
        token_account_beginning_lamports.saturating_add(expected_harvested_rewards),
    );
}
