#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_rewards_program::{
        error::PaladinRewardsError,
        instruction::harvest_rewards,
        state::{get_holder_rewards_address, get_holder_rewards_pool_address, HolderRewards},
    },
    setup::{
        setup, setup_holder_rewards_account, setup_holder_rewards_pool_account, setup_mint,
        setup_token_account,
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
async fn fail_mint_invalid_data() {
    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;

    // Set up a mint with invalid data.
    {
        context.set_account(
            &mint,
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
async fn fail_token_account_invalid_data() {
    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), 0).await;

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
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_account(&mut context, &holder_rewards, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), 0).await;

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
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = Pubkey::new_unique(); // Incorrect holder rewards pool address.

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_holder_rewards_account(&mut context, &holder_rewards, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), 0).await;

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
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), 0).await;

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
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), 0).await;

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
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_holder_rewards_account(&mut context, &holder_rewards, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), 0).await;

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
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_token_account(&mut context, &token_account, &owner, &mint, 0).await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), 0).await;

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

struct System {
    token_supply: u64,
    total_rewards: u64,
    pool_excess_lamports: u64,
}

struct Holder {
    token_account_balance: u64,
    last_seen_total_rewards: u64,
    unharvested_rewards: u64,
}

#[test_case(
    System {
        token_supply: 0,
        total_rewards: 0,
        pool_excess_lamports: 0,
    },
    Holder {
        token_account_balance: 0,
        last_seen_total_rewards: 0,
        unharvested_rewards: 0,
    },
    0,
    0;
    "all zeroes, no rewards"
)]
#[test_case(
    System {
        token_supply: 10_000,
        total_rewards: 10_000,
        pool_excess_lamports: 20_000,
    },
    Holder {
        token_account_balance: 0,
        last_seen_total_rewards: 0,
        unharvested_rewards: 0,
    },
    0,
    0;
    "share of token supply is zero, 0 unharvested, 0 last seen, no rewards"
)]
#[test_case(
    System {
        token_supply: 10_000,
        total_rewards: 10_000,
        pool_excess_lamports: 20_000,
    },
    Holder {
        token_account_balance: 0,
        last_seen_total_rewards: 5_000, // Last saw half.
        unharvested_rewards: 2_500, // Not yet harvested.
    },
    2_500, // Fully harvested unharvested rewards.
    0; // Token account balance is zero, no new rewards.
    "share of token supply is zero, some unharvested, half last seen, pool has enough, only unharvested"
)]
#[test_case(
    System {
        token_supply: 10_000,
        total_rewards: 10_000,
        pool_excess_lamports: 1_000, // Not enough.
    },
    Holder {
        token_account_balance: 0,
        last_seen_total_rewards: 5_000, // Last saw half.
        unharvested_rewards: 2_500, // Not yet harvested.
    },
    1_000, // Only harvested what was in the pool.
    2_500 - 1_000; // Token account balance is zero, no new rewards, but some was unharvested since pool was underfunded.
    "share of token supply is zero, some unharvested, half last seen, pool underfunded, pool excess harvested, rest unharvested"
)]
#[test_case(
    System {
        token_supply: 0,
        total_rewards: 10_000,
        pool_excess_lamports: 20_000,
    },
    Holder {
        token_account_balance: 0,
        last_seen_total_rewards: 0,
        unharvested_rewards: 0,
    },
    0, // Token supply is zero, no new rewards.
    0; // No unharvested rewards to harvest.
    "token supply is zero, 0 unharvested, 0 last seen, no rewards"
)]
#[test_case(
    System {
        token_supply: 0,
        total_rewards: 10_000,
        pool_excess_lamports: 20_000,
    },
    Holder {
        token_account_balance: 0,
        last_seen_total_rewards: 5_000, // Last saw half.
        unharvested_rewards: 2_500, // Not yet harvested.
    },
    2_500, // Fully harvested unharvested rewards.
    0; // Token supply is zero, no new rewards.
    "token supply is zero, some unharvested, half last seen, pool has enough, only unharvested"
)]
#[test_case(
    System {
        token_supply: 0,
        total_rewards: 10_000,
        pool_excess_lamports: 1_000, // Not enough.
    },
    Holder {
        token_account_balance: 0,
        last_seen_total_rewards: 5_000, // Last saw half.
        unharvested_rewards: 2_500, // Not yet harvested.
    },
    1_000, // Only harvested what was in the pool.
    2_500 - 1_000; // Token supply is zero, no new rewards, but some was unharvested since pool was underfunded.
    "token supply is zero, some unharvested, half last seen, pool underfunded, pool excess"
)]
#[test_case(
    System {
        token_supply: 10_000,
        total_rewards: 10_000,
        pool_excess_lamports: 20_000,
    },
    Holder {
        token_account_balance: 5_000,
        last_seen_total_rewards: 0,
        unharvested_rewards: 0,
    },
    5_000, // 50% of total rewards.
    0; // No unharvested rewards remain, pool had enough.
    "50% of token supply, 0 unharvested, 0 last seen, pool has enough, 50% of rewards"
)]
#[test_case(
    System {
        token_supply: 10_000,
        total_rewards: 10_000,
        pool_excess_lamports: 2_000, // Not enough.
    },
    Holder {
        token_account_balance: 5_000,
        last_seen_total_rewards: 0,
        unharvested_rewards: 0,
    },
    2_000, // Pool excess.
    5_000 - 2_000; // 50% of total rewards, but pool was underfunded.
    "50% of token supply, 0 unharvested, 0 last seen, pool underfunded, pool excess harvested, rest unharvested"
)]
#[test_case(
    System {
        token_supply: 10_000,
        total_rewards: 10_000,
        pool_excess_lamports: 20_000,
    },
    Holder {
        token_account_balance: 5_000,
        last_seen_total_rewards: 5_000, // Last saw half.
        unharvested_rewards: 0, // Harvested since last seen.
    },
    2_500, // 50% of unseen rewards.
    0; // No unharvested rewards remain, pool had enough.
    "50% of token supply, 0 unharvested, half last seen, pool has enough, half of 50% of rewards"
)]
#[test_case(
    System {
        token_supply: 10_000,
        total_rewards: 10_000,
        pool_excess_lamports: 1_000, // Not enough.
    },
    Holder {
        token_account_balance: 5_000,
        last_seen_total_rewards: 5_000, // Last saw half.
        unharvested_rewards: 0, // Harvested since last seen.
    },
    1_000, // Pool excess.
    2_500 - 1_000; // 50% of unseen rewards, but pool was underfunded.
    "50% of token supply, 0 unharvested, half last seen, pool underfunded, pool excess harvested, rest unharvested"
)]
#[test_case(
    System {
        token_supply: 10_000,
        total_rewards: 10_000,
        pool_excess_lamports: 20_000,
    },
    Holder {
        token_account_balance: 5_000,
        last_seen_total_rewards: 5_000, // Last saw half.
        unharvested_rewards: 2_500, // Not yet harvested.
    },
    5_000, // 50% of total rewards.
    0; // No unharvested rewards remain, pool had enough.
    "50% of token supply, some unharvested, half last seen, pool has enough, 50% of rewards"
)]
#[test_case(
    System {
        token_supply: 10_000,
        total_rewards: 10_000,
        pool_excess_lamports: 1_000, // Not enough.
    },
    Holder {
        token_account_balance: 5_000,
        last_seen_total_rewards: 5_000, // Last saw half.
        unharvested_rewards: 2_500, // Not yet harvested.
    },
    1_000, // Pool excess.
    5_000 - 1_000; // 50% of total rewards, but pool was underfunded.
    "50% of token supply, some unharvested, half last seen, pool underfunded, pool excess harvested, rest unharvested"
)]
#[tokio::test]
async fn success(
    system: System,
    holder: Holder,
    expected_harvested_rewards: u64,
    expected_unharvested_rewards: u64,
) {
    let System {
        token_supply,
        total_rewards,
        pool_excess_lamports,
    } = system;

    let Holder {
        token_account_balance,
        last_seen_total_rewards,
        unharvested_rewards,
    } = holder;

    let owner = Pubkey::new_unique();
    let mint = Pubkey::new_unique();

    let token_account = get_associated_token_address(&owner, &mint);
    let holder_rewards = get_holder_rewards_address(&token_account);
    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(
        &mut context,
        &holder_rewards_pool,
        pool_excess_lamports,
        total_rewards,
    )
    .await;
    setup_holder_rewards_account(
        &mut context,
        &holder_rewards,
        unharvested_rewards,
        last_seen_total_rewards,
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
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), token_supply).await;

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
            unharvested_rewards: expected_unharvested_rewards,
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
