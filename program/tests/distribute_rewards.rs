#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_rewards_program::{
        error::PaladinRewardsError,
        instruction::distribute_rewards,
        state::{get_holder_rewards_pool_address, HolderRewardsPool},
    },
    setup::{setup, setup_holder_rewards_pool_account, setup_mint, setup_system_account},
    solana_program_test::*,
    solana_sdk::{
        account::AccountSharedData,
        instruction::InstructionError,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        transaction::{Transaction, TransactionError},
    },
    test_case::test_case,
};

#[tokio::test]
async fn fail_mint_invalid_data() {
    let mint = Pubkey::new_unique();

    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let payer = Keypair::new();
    let amount = 500_000_000_000;

    let mut context = setup().start_with_context().await;

    // Set up a mint with invalid data.
    {
        context.set_account(
            &mint,
            &AccountSharedData::new_data(100_000_000, &vec![5; 165], &spl_token_2022::id())
                .unwrap(),
        );
    }

    let instruction = distribute_rewards(&payer.pubkey(), &holder_rewards_pool, &mint, amount);

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

#[tokio::test]
async fn fail_payer_not_signer() {
    let mint = Pubkey::new_unique();
    let token_supply = 100_000;

    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let payer = Keypair::new();
    let amount = 500_000_000_000;

    let mut context = setup().start_with_context().await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), token_supply).await;

    let mut instruction = distribute_rewards(&payer.pubkey(), &holder_rewards_pool, &mint, amount);
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
    let mint = Pubkey::new_unique();
    let token_supply = 100_000;

    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let payer = Keypair::new();
    let amount = 500_000_000_000;

    let mut context = setup().start_with_context().await;
    setup_system_account(&mut context, &payer.pubkey(), amount).await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), token_supply).await;

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

    let instruction = distribute_rewards(&payer.pubkey(), &holder_rewards_pool, &mint, amount);

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
async fn fail_holder_rewards_pool_invalid_address() {
    let mint = Pubkey::new_unique();
    let token_supply = 100_000;

    let holder_rewards_pool = Pubkey::new_unique(); // Incorrect holder rewards pool address.
    let payer = Keypair::new();
    let amount = 500_000_000_000;

    let mut context = setup().start_with_context().await;
    setup_holder_rewards_pool_account(&mut context, &holder_rewards_pool, 0, 0).await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), token_supply).await;

    let instruction = distribute_rewards(&payer.pubkey(), &holder_rewards_pool, &mint, amount);

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
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinRewardsError::IncorrectHolderRewardsPoolAddress as u32)
        )
    );
}

#[tokio::test]
async fn fail_holder_rewards_pool_invalid_data() {
    let mint = Pubkey::new_unique();
    let token_supply = 100_000;

    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let payer = Keypair::new();
    let amount = 500_000_000_000;

    let mut context = setup().start_with_context().await;
    setup_system_account(&mut context, &payer.pubkey(), amount).await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), token_supply).await;

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

    let instruction = distribute_rewards(&payer.pubkey(), &holder_rewards_pool, &mint, amount);

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

struct InitialPool {
    token_supply: u64,
    accumulated_rewards_per_token: u128,
}

struct ExpectedPool {
    accumulated_rewards_per_token: u128,
}

#[allow(clippy::arithmetic_side_effects)]
#[test_case(
    InitialPool {
        token_supply: 0,
        accumulated_rewards_per_token: 0,
    },
    ExpectedPool {
        accumulated_rewards_per_token: 0,
    },
    100_000;
    "Zero token supply, zero rewards per token, increment total rewards"
)]
#[test_case(
    InitialPool {
        token_supply: 100_000,
        accumulated_rewards_per_token: 0,
    },
    ExpectedPool {
        accumulated_rewards_per_token: 2_500_000_000, // 0% + 250_000 / 100_000 = 250%
    },
    250_000;
    "Zero initial rate and rewards, resulting rate 250%"
)]
#[test_case(
    InitialPool {
        token_supply: 1_000_000,
        accumulated_rewards_per_token: 0,
    },
    ExpectedPool {
        accumulated_rewards_per_token: 100_000_000, // 0% + 100_000 / 1_000_000 = 10%
    },
    100_000;
    "Zero initial rate and rewards, resulting rate 10%"
)]
#[test_case(
    InitialPool {
        token_supply: 1_000_000,
        accumulated_rewards_per_token: 0,
    },
    ExpectedPool {
        accumulated_rewards_per_token: 1_000_000, // 0% + 1_000 / 1_000_000 = 0.1%
    },
    1_000;
    "Zero initial rate and rewards, resulting rate 0.1%"
)]
#[test_case(
    InitialPool {
        token_supply: 1_000_000,
        accumulated_rewards_per_token: 0,
    },
    ExpectedPool {
        accumulated_rewards_per_token: 1_000, // 0 + 1 / 1_000_000 = 0.0001%
    },
    1;
    "Zero initial rate and rewards, resulting rate 0.0001%"
)]
#[test_case(
    InitialPool {
        token_supply: 100_000,
        accumulated_rewards_per_token: 500_000_000, // 50%
    },
    ExpectedPool {
        accumulated_rewards_per_token: 525_000_000, // 50% + 2_500 / 100_000 = 52.5%
    },
    2_500;
    "50% initial rate, rewards increase by 5%, resulting rate 52.5%"
)]
#[test_case(
    InitialPool {
        token_supply: 100_000,
        accumulated_rewards_per_token: 500_000_000, // 50%
    },
    ExpectedPool {
        accumulated_rewards_per_token: 1_000_000_000, // 50% + 50_000 / 100_000 = 100%
    },
    50_000;
    "50% initial rate, rewards increase by 100%, resulting rate 100%"
)]
#[test_case(
    InitialPool {
        token_supply: 100_000,
        accumulated_rewards_per_token: 500_000_000, // 50%
    },
    ExpectedPool {
        accumulated_rewards_per_token: 1_750_000_000, // 50% + 125_000 / 100_000 = 175%
    },
    125_000;
    "50% initial rate, rewards increase by 250%, resulting rate 175%"
)]
#[tokio::test]
async fn success(initial: InitialPool, expected: ExpectedPool, reward_amount: u64) {
    let InitialPool {
        token_supply,
        accumulated_rewards_per_token,
    } = initial;
    let ExpectedPool {
        accumulated_rewards_per_token: expected_accumulated_rewards_per_token,
    } = expected;

    let mint = Pubkey::new_unique();

    let holder_rewards_pool =
        get_holder_rewards_pool_address(&mint, &paladin_rewards_program::id());
    let payer = Keypair::new();

    let mut context = setup().start_with_context().await;
    setup_system_account(&mut context, &payer.pubkey(), reward_amount).await;
    setup_holder_rewards_pool_account(
        &mut context,
        &holder_rewards_pool,
        0, // Excess lamports (not used here).
        accumulated_rewards_per_token,
    )
    .await;
    setup_mint(&mut context, &mint, &Pubkey::new_unique(), token_supply).await;

    // For checks later.
    let payer_beginning_lamports = context
        .banks_client
        .get_account(payer.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;

    let instruction =
        distribute_rewards(&payer.pubkey(), &holder_rewards_pool, &mint, reward_amount);

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
            accumulated_rewards_per_token: expected_accumulated_rewards_per_token,
        },
    );

    // Assert the pool was credited lamports.
    let rent = context.banks_client.get_rent().await.unwrap();
    let expected_lamports =
        rent.minimum_balance(std::mem::size_of::<HolderRewardsPool>()) + reward_amount;
    assert_eq!(holder_rewards_pool_account.lamports, expected_lamports);

    // Assert the payer's account balance was debited.
    let payer_resulting_lamports = context
        .banks_client
        .get_account(payer.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(
        payer_resulting_lamports,
        payer_beginning_lamports - reward_amount
    );
}
