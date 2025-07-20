#![cfg(feature = "test-sbf")]

mod execute_utils;
mod setup;

use {
    crate::{
        execute_utils::execute_with_payer_err, setup::{
            setup_holder_rewards_account_with_token_account,
            setup_holder_rewards_pool_account_with_token_account,
            DEPOSIT_AMOUNT,
        }
    },
    paladin_rewards_program::{
        error::PaladinRewardsError, instruction::harvest_rewards, processor::REWARDS_PER_TOKEN_SCALING_FACTOR, state::{
            get_holder_rewards_address, get_holder_rewards_pool_address, HolderRewards,
        }
    },
    setup::setup,
    solana_program_test::*,
    solana_sdk::{
        instruction::InstructionError, pubkey::Pubkey, signature::Keypair, signer::Signer, transaction::{Transaction, TransactionError}
    },
    spl_associated_token_account::get_associated_token_address,
    test_case::test_case,
};

#[tokio::test]
async fn fail_not_enough_lamports() {
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
        DEPOSIT_AMOUNT / 25,                // Not enough rewards (expected DEPOSIT_AMOUNT / 50)
        REWARDS_PER_TOKEN_SCALING_FACTOR,   // accumalated per token
        DEPOSIT_AMOUNT,                     // pool balance (total deposited by all users)
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
        DEPOSIT_AMOUNT,                         // total deposited for holder
        REWARDS_PER_TOKEN_SCALING_FACTOR / 50,  // last rewards per token
        0, // token balance
    )
    .await;

    let instruction = harvest_rewards(
        &holder_rewards_pool,
        &pool_token,
        &holder_rewards,
        &mint,
        &owner.pubkey(),
    );

    let err = execute_with_payer_err(&mut context, instruction, Some(&owner)).await;

    assert_eq!(
        err,
        TransactionError::InstructionError(
            0,
            InstructionError::Custom(PaladinRewardsError::RewardsExcessPoolBalance as u32)
        )
    );
}

struct Pool {
    excess_lamports: u64,
    accumulated_rewards_per_token: u128,
    total_deposited: u64,
}

struct Holder {
    last_accumulated_rewards_per_token: u128,
    deposited: u64,
}

#[test_case(
    Pool {
        excess_lamports: 0,
        accumulated_rewards_per_token: 0,
        total_deposited: 0,
    },
    Holder {
        last_accumulated_rewards_per_token: 0,
        deposited: 0,
    },
    0;
    "All zeroes, no rewards"
)]
#[test_case(
    Pool {
        excess_lamports: 0,
        accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR, // 1 reward per token.    
        total_deposited: DEPOSIT_AMOUNT,
    },
    Holder {
        last_accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR, // 1 reward per token.
        deposited: DEPOSIT_AMOUNT,
    },
    0;
    "Last harvested 1.0 rate, rate unchanged, no rewards"
)]
#[test_case(
    Pool {
        excess_lamports: DEPOSIT_AMOUNT * 2,
        accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR - 1, // 1 reward per token from u128::MAX.     
        total_deposited: DEPOSIT_AMOUNT * 2,
    },
    Holder {
        last_accumulated_rewards_per_token: u128::MAX, // Maximum rate.
        deposited: DEPOSIT_AMOUNT,
    },
    DEPOSIT_AMOUNT;
    "Accumulated amount wrapping is working" 
)]
#[test_case(
    Pool {
        excess_lamports: DEPOSIT_AMOUNT,
        accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR, // 1 reward per token.
        total_deposited: DEPOSIT_AMOUNT,
    },
    Holder {
        last_accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR / 4, // 25% rewards per token.
        deposited: DEPOSIT_AMOUNT,
    },
    DEPOSIT_AMOUNT * 75 / 100; // 75%
    "Last harvested 0.25 rate, eligible for 0.75 rate, pool has enough, receive share" 
)]
#[test_case(
    Pool {
        excess_lamports: DEPOSIT_AMOUNT * 2,
        accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR, // 1 reward per token.     
        total_deposited: DEPOSIT_AMOUNT * 2,
    },
    Holder {
        last_accumulated_rewards_per_token: 0,
        deposited: DEPOSIT_AMOUNT,
    },
    DEPOSIT_AMOUNT;
    "Harvest 50% based on holders deposited amount" 
)]
#[test_case(
    Pool {
        excess_lamports: DEPOSIT_AMOUNT * 2,
        accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR + 1, // 1 and some reward per token.     
        total_deposited: DEPOSIT_AMOUNT * 2,
    },
    Holder {
        last_accumulated_rewards_per_token: 0,
        deposited: DEPOSIT_AMOUNT,
    },
    DEPOSIT_AMOUNT; 
    "Confirm rounding over" 
)]
#[test_case(
    Pool {
        excess_lamports: DEPOSIT_AMOUNT * 2,
        accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR - 1, // Just less then 1 reward per token.     
        total_deposited: DEPOSIT_AMOUNT * 2,
    },
    Holder {
        last_accumulated_rewards_per_token: 0,
        deposited: DEPOSIT_AMOUNT,
    },
    DEPOSIT_AMOUNT - 1; 
    "Confirm rounding under" 
)]
#[test_case(
    Pool {
        excess_lamports: DEPOSIT_AMOUNT,
        accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR, // 1 reward per token.  
        total_deposited: 0,   
    },
    Holder {
        last_accumulated_rewards_per_token: 0,
        deposited: DEPOSIT_AMOUNT,
    },
    DEPOSIT_AMOUNT;
    "Harvet repay all rewards"
)]
#[tokio::test]
async fn success(
    pool: Pool,
    holder: Holder,
    expected_harvested_rewards: u64,
) {
    let Pool {
        excess_lamports,
        accumulated_rewards_per_token,
        total_deposited,
    } = pool;
    let Holder {
        last_accumulated_rewards_per_token,
        deposited, 
    } = holder;

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
        excess_lamports,
        accumulated_rewards_per_token,
        total_deposited,
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
        deposited,
        last_accumulated_rewards_per_token,
        0,
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
    let owner_beginning_lamports: u64 = 0;

    let instruction = harvest_rewards(
        &holder_rewards_pool,
        &pool_token,
        &holder_rewards,
        &mint,
        &owner.pubkey(),
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &owner],
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
        &HolderRewards {
            last_accumulated_rewards_per_token: accumulated_rewards_per_token,
            deposited,
            _padding: 0,
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
        pool_beginning_lamports
            .checked_sub(expected_harvested_rewards)
            .unwrap(),
    );

    // Assert the token account's balance was credited.
    let owner_resulting_lamports = if expected_harvested_rewards > 0 {
        context
            .banks_client
            .get_account(owner.pubkey())
            .await
            .unwrap()
            .unwrap()
            .lamports
    } else {
        assert!(context
            .banks_client
            .get_account(owner.pubkey())
            .await
            .unwrap().is_none());

        0
    };
    assert_eq!(
        owner_resulting_lamports,
        owner_beginning_lamports
            .checked_add(expected_harvested_rewards)
            .unwrap(),
    );
}
