#![cfg(feature = "test-sbf")]

mod setup;

use {
    crate::setup::{
        send_rewards_to_pool, setup_holder_rewards_account_with_token_account,
        setup_holder_rewards_pool_account_with_token_account,
    },
    paladin_rewards_program::{
        error::PaladinRewardsError,
        instruction::{deposit, initialize_holder_rewards},
        state::{
            get_holder_rewards_address, get_holder_rewards_pool_address, HolderRewards,
            HolderRewardsPool,
        },
    },
    setup::{setup, setup_holder_rewards_pool_account, setup_mint, setup_token_account},
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData, ReadableAccount},
        instruction::InstructionError,
        program_pack::Pack,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        system_program,
        transaction::{Transaction, TransactionError},
    },
    spl_associated_token_account::get_associated_token_address,
    spl_token::state::Account as TokenAccount,
};

const DEPOSIT_AMOUNT: u64 = 250_000_000;
const INITIAL_OWNER_BALANCE: u64 = 1_000_000_000;

#[tokio::test]
async fn success() {
    let owner = Keypair::new();
    let mint = Pubkey::new_unique();

    let mut context = setup().start_with_context().await;

    // fund owner
    // context.set_account(
    //     &owner.pubkey(),
    //     &AccountSharedData::new(1_000_000, 0, &system_program::id()),
    // );

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

    let instruction = deposit(
        holder_rewards_pool,
        pool_token,
        holder_rewards,
        owner_token,
        mint,
        owner.pubkey(),
        DEPOSIT_AMOUNT,
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

    // Assert pool balance is DEPOSIT_AMOUNT.
    let pool_token_account = context
        .banks_client
        .get_account(pool_token)
        .await
        .unwrap()
        .unwrap();
    let pool_token_account_balance = TokenAccount::unpack(&pool_token_account.data)
        .unwrap()
        .amount;
    assert_eq!(pool_token_account_balance, DEPOSIT_AMOUNT);

    // Assert owner balance is INITIAL_OWNER_BALANCE - DEPOSIT_AMOUNT.
    let owner_token_account = context
        .banks_client
        .get_account(owner_token)
        .await
        .unwrap()
        .unwrap();
    let owner_token_account_balance = TokenAccount::unpack(&owner_token_account.data)
        .unwrap()
        .amount;
    assert_eq!(
        owner_token_account_balance,
        INITIAL_OWNER_BALANCE - DEPOSIT_AMOUNT
    );

    let holder_rewards_account = context
        .banks_client
        .get_account(holder_rewards)
        .await
        .unwrap()
        .unwrap();
    let holder_deposited =
        bytemuck::from_bytes::<HolderRewards>(&holder_rewards_account.data).total_deposited;
    assert_eq!(holder_deposited, DEPOSIT_AMOUNT);

    // Confirm that rewards are being sent on 2nd deposit.
    let rewards_amount = 1_000_000_000;
    let previous_pool_lamports = context
        .banks_client
        .get_account(holder_rewards_pool)
        .await
        .unwrap()
        .unwrap()
        .lamports;
    send_rewards_to_pool(&mut context, &holder_rewards_pool, rewards_amount).await;
    // Assert rewards were sent
    let check_pool_lamports = context
        .banks_client
        .get_account(holder_rewards_pool)
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(check_pool_lamports, previous_pool_lamports + rewards_amount);

    // Deposit again to check if rewards are being sent
    let instruction = deposit(
        holder_rewards_pool,
        pool_token,
        holder_rewards,
        owner_token,
        mint,
        owner.pubkey(),
        DEPOSIT_AMOUNT / 2,
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

    // Assert pool balance is DEPOSIT_AMOUNT * 2.
    let pool_token_account = context
        .banks_client
        .get_account(pool_token)
        .await
        .unwrap()
        .unwrap();
    let pool_token_account_balance = TokenAccount::unpack(&pool_token_account.data)
        .unwrap()
        .amount;
    assert_eq!(
        pool_token_account_balance,
        DEPOSIT_AMOUNT + DEPOSIT_AMOUNT / 2
    );

    // Assert owner balance is INITIAL_OWNER_BALANCE - DEPOSIT_AMOUNT * 2.
    let owner_token_account = context
        .banks_client
        .get_account(owner_token)
        .await
        .unwrap()
        .unwrap();
    let owner_token_account_balance = TokenAccount::unpack(&owner_token_account.data)
        .unwrap()
        .amount;
    assert_eq!(
        owner_token_account_balance,
        INITIAL_OWNER_BALANCE - (DEPOSIT_AMOUNT + DEPOSIT_AMOUNT / 2)
    );

    let holder_rewards_pool_account = context
        .banks_client
        .get_account(holder_rewards_pool)
        .await
        .unwrap()
        .unwrap();
    let pool_state = bytemuck::from_bytes::<HolderRewardsPool>(&holder_rewards_pool_account.data);
    let holder_reward_account = context
        .banks_client
        .get_account(holder_rewards)
        .await
        .unwrap()
        .unwrap();
    let holder_rewards_state = bytemuck::from_bytes::<HolderRewards>(&holder_reward_account.data);
    assert_eq!(
        holder_rewards_state.last_accumulated_rewards_per_token,
        pool_state.accumulated_rewards_per_token
    );

    // Assert pool sent all rewards to holder (single holder)
    let current_pool_lamports = context
        .banks_client
        .get_account(holder_rewards_pool)
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(current_pool_lamports, previous_pool_lamports);

    // Assert rewards were sent to owner
    let current_owner_lamports = context
        .banks_client
        .get_account(owner.pubkey())
        .await
        .unwrap()
        .unwrap()
        .lamports;
    assert_eq!(current_owner_lamports, rewards_amount);
}
