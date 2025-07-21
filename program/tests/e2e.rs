//! End-to-end test.

#![cfg(feature = "test-sbf")]

mod execute_utils;
mod setup;

use {
    crate::{
        execute_utils::execute_with_payer,
        setup::{
            send_rewards_to_pool, setup_holder_rewards_account,
            setup_holder_rewards_account_with_token_account,
            setup_holder_rewards_pool_account_with_token_account, setup_owner, DEPOSIT_AMOUNT,
            INITIAL_OWNER_BALANCE,
        },
    },
    paladin_rewards_program::{
        instruction::{
            close_holder_rewards, deposit, harvest_rewards, initialize_holder_rewards,
            initialize_holder_rewards_pool, withdraw,
        },
        processor::REWARDS_PER_TOKEN_SCALING_FACTOR,
        state::{
            get_holder_rewards_address, get_holder_rewards_pool_address, HolderRewards,
            HolderRewardsPool,
        },
    },
    setup::{setup, setup_mint, setup_token_account},
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        instruction::Instruction,
        program_pack::Pack,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        system_instruction,
        transaction::Transaction,
    },
    spl_associated_token_account::get_associated_token_address,
    spl_token::state::{Account as TokenAccount, Mint},
};

struct Pool {
    total_deposited: u64,
    accumulated_rewards_per_token: u128,
    pool_excess_lamports: u64,
    lamports_last: u64,
}

struct Holder {
    last_accumulated_rewards_per_token: u128,
    deposited: u64,
    expected_lamports: u64,
}

async fn pool_rent_exempt_lamports(context: &mut ProgramTestContext) -> u64 {
    context
        .banks_client
        .get_rent()
        .await
        .expect("get_rent")
        .minimum_balance(std::mem::size_of::<HolderRewardsPool>())
}

async fn holder_rent_exempt_lamports(context: &mut ProgramTestContext) -> u64 {
    context
        .banks_client
        .get_rent()
        .await
        .expect("get_rent")
        .minimum_balance(std::mem::size_of::<HolderRewards>())
}

async fn wallet_rent_exempt_lamports(context: &mut ProgramTestContext) -> u64 {
    context
        .banks_client
        .get_rent()
        .await
        .expect("get_rent")
        .minimum_balance(std::mem::size_of::<Account>())
}

async fn get_account(context: &mut ProgramTestContext, address: &Pubkey) -> Account {
    context
        .banks_client
        .get_account(*address)
        .await
        .expect("get_account")
        .expect("account not found")
}

async fn validate_state(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    pool: Pool,
    holder_rewards: &[(&Pubkey, Holder)],
) {
    // First evaluate the pool.
    {
        let pool_address = get_holder_rewards_pool_address(mint, &paladin_rewards_program::id());
        let pool_account = get_account(context, &pool_address).await;
        let pool_token = get_associated_token_address(&pool_address, &mint);
        let pool_token_account = get_account(context, &pool_token).await;
        let pool_token_state = TokenAccount::unpack(&pool_token_account.data).unwrap();
        assert_eq!(pool_token_state.amount, pool.total_deposited);

        let pool_rent_exempt_lamports = pool_rent_exempt_lamports(context).await;
        let pool_excess_lamports = pool_account.lamports - pool_rent_exempt_lamports;
        assert_eq!(pool_excess_lamports, pool.pool_excess_lamports);

        let pool_state = bytemuck::from_bytes::<HolderRewardsPool>(&pool_account.data);
        assert_eq!(
            pool_state,
            &HolderRewardsPool {
                accumulated_rewards_per_token: pool.accumulated_rewards_per_token,
                lamports_last: pool.lamports_last + pool_rent_exempt_lamports,
                _padding: 0,
            }
        );
    }

    // Then evaluate the holders.
    let wallet_rent_exempt_lamports = wallet_rent_exempt_lamports(context).await;

    for (owner, checks) in holder_rewards {
        let owner_lamports = get_account(context, owner).await.lamports;

        let holder_rewards_address =
            get_holder_rewards_address(owner, &paladin_rewards_program::id());
        let holder_rewards_account = get_account(context, &holder_rewards_address).await;
        let holder_rewards_state =
            bytemuck::from_bytes::<HolderRewards>(&holder_rewards_account.data);
        assert_eq!(
            holder_rewards_state,
            &HolderRewards {
                last_accumulated_rewards_per_token: checks.last_accumulated_rewards_per_token,
                deposited: checks.deposited,
                _padding: 0,
            }
        );

        // Asserts lamports
        assert_eq!(
            owner_lamports - wallet_rent_exempt_lamports,
            checks.expected_lamports,
            "Owner lamports: {} | rent: {} | expected: {}",
            owner_lamports,
            wallet_rent_exempt_lamports,
            checks.expected_lamports,
        );
    }
}

#[tokio::test]
async fn test_e2e() {
    let mut context = setup().start_with_context().await;

    let mint = Pubkey::new_unique();
    setup_mint(&mut context, &mint, INITIAL_OWNER_BALANCE * 4, None).await;

    // Setup pool with everything 0
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

    // Setup Alice holder
    let alice = Keypair::new();
    setup_owner(&mut context, &alice.pubkey()).await;
    let alice_holder_rewards =
        get_holder_rewards_address(&alice.pubkey(), &paladin_rewards_program::id());
    let alice_token = get_associated_token_address(&alice.pubkey(), &mint);

    setup_holder_rewards_account_with_token_account(
        &mut context,
        &mint,
        &alice.pubkey(),
        &alice_holder_rewards,
        &alice_token,
        0,
        0,
        INITIAL_OWNER_BALANCE, // token balance
    )
    .await;

    // Setup Bob
    let bob = Keypair::new();
    setup_owner(&mut context, &bob.pubkey()).await;
    let bob_holder_rewards =
        get_holder_rewards_address(&bob.pubkey(), &paladin_rewards_program::id());
    let bob_token = get_associated_token_address(&bob.pubkey(), &mint);

    setup_holder_rewards_account_with_token_account(
        &mut context,
        &mint,
        &bob.pubkey(),
        &bob_holder_rewards,
        &bob_token,
        0,
        0,
        INITIAL_OWNER_BALANCE, // token balance
    )
    .await;

    // Setup carol
    let carol = Keypair::new();
    setup_owner(&mut context, &carol.pubkey()).await;
    let carol_holder_rewards =
        get_holder_rewards_address(&carol.pubkey(), &paladin_rewards_program::id());
    let carol_token = get_associated_token_address(&carol.pubkey(), &mint);

    setup_holder_rewards_account_with_token_account(
        &mut context,
        &mint,
        &carol.pubkey(),
        &carol_holder_rewards,
        &carol_token,
        0,
        0,
        INITIAL_OWNER_BALANCE, // token balance
    )
    .await;

    // Setup Dave
    let dave = Keypair::new();
    setup_owner(&mut context, &dave.pubkey()).await;
    let dave_holder_rewards =
        get_holder_rewards_address(&dave.pubkey(), &paladin_rewards_program::id());
    let dave_token = get_associated_token_address(&dave.pubkey(), &mint);

    setup_holder_rewards_account_with_token_account(
        &mut context,
        &mint,
        &dave.pubkey(),
        &dave_holder_rewards,
        &dave_token,
        0,
        0,
        INITIAL_OWNER_BALANCE, // token balance
    )
    .await;

    // Round 0
    //
    // Pool:   total_deposited:    0           Alice:  deposited:       0
    //         rewards_per_token:  0                   last_seen_rate:  0
    //         available_rewards:  0                   eligible_for:    0
    //
    //                                         Bob:    deposited:       0
    //                                                 last_seen_rate:  0
    //                                                 eligible_for:    0
    //
    //                                         Carol:  deposited:       0
    //                                                 last_seen_rate:  0
    //                                                 eligible_for:    0
    //
    //                                         Dave:   deposited:       0
    //                                                 last_seen_rate:  0
    //                                                 eligible_for:    0

    validate_state(
        &mut context,
        &mint,
        Pool {
            total_deposited: 0,
            accumulated_rewards_per_token: 0,
            pool_excess_lamports: 0,
            lamports_last: 0,
        },
        &[],
    )
    .await;

    // Round 1
    //
    // Pool:   total_deposited:    150         Alice:  deposited:       100
    //         rewards_per_token:  1                   last_seen_rate:  1
    //         available_rewards:  150                 eligible_for:    100
    //
    //                                         Bob:    deposited:       50
    //                                                 last_seen_rate:  1
    //                                                 eligible_for:    50
    //
    //                                         Carol:  deposited:       0
    //                                                 last_seen_rate:  0
    //                                                 eligible_for:    0
    //
    //                                         Dave:   deposited:       0
    //                                                 last_seen_rate:  0
    //                                                 eligible_for:    0
    context.warp_forward_force_reward_interval_end().unwrap();

    // Alice deposits 100 tokens
    let instruction = deposit(
        &holder_rewards_pool,
        &pool_token,
        &alice_holder_rewards,
        &alice_token,
        &mint,
        &alice.pubkey(),
        100,
    );
    execute_with_payer(&mut context, instruction, Some(&alice)).await;

    // Bob deposits 50 tokens
    let instruction = deposit(
        &holder_rewards_pool,
        &pool_token,
        &bob_holder_rewards,
        &bob_token,
        &mint,
        &bob.pubkey(),
        50,
    );
    execute_with_payer(&mut context, instruction, Some(&bob)).await;

    // Validate counting is correct
    validate_state(
        &mut context,
        &mint,
        Pool {
            total_deposited: 150,
            accumulated_rewards_per_token: 0,
            pool_excess_lamports: 0,
            lamports_last: 0,
        },
        &[
            (
                &alice.pubkey(),
                Holder {
                    last_accumulated_rewards_per_token: 0,
                    deposited: 100,
                    expected_lamports: 0,
                },
            ),
            (
                &bob.pubkey(),
                Holder {
                    last_accumulated_rewards_per_token: 0,
                    deposited: 50,
                    expected_lamports: 0,
                },
            ),
        ],
    )
    .await;

    // Send 150 rewards
    send_rewards_to_pool(&mut context, &holder_rewards_pool, 150).await;

    // Alice harvest rewards
    let instruction = harvest_rewards(
        &holder_rewards_pool,
        &pool_token,
        &alice_holder_rewards,
        &mint,
        &alice.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&alice)).await;

    // Bob harvest rewards
    let instruction = harvest_rewards(
        &holder_rewards_pool,
        &pool_token,
        &bob_holder_rewards,
        &mint,
        &bob.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&bob)).await;

    // Validate with rewards
    validate_state(
        &mut context,
        &mint,
        Pool {
            total_deposited: 150,
            accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR,
            pool_excess_lamports: 0, // All rewards were distributed so its back to 0
            lamports_last: 0,        // All rewards were distributed so its back to 0
        },
        &[
            (
                &alice.pubkey(),
                Holder {
                    last_accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR,
                    deposited: 100,
                    expected_lamports: 100,
                },
            ),
            (
                &bob.pubkey(),
                Holder {
                    last_accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR,
                    deposited: 50,
                    expected_lamports: 50,
                },
            ),
        ],
    )
    .await;

    // Round 2
    //
    // Pool:   total_deposited:    300         Alice:  deposited:       100
    //         rewards_per_token:  2                   last_seen_rate:  2
    //         available_rewards:  300                 eligible_for:    100
    //
    //                                         Bob:    deposited:       50
    //                                                 last_seen_rate:  1
    //                                                 eligible_for:    50
    //
    //                                         Carol:  deposited:       150
    //                                                 last_seen_rate:  2
    //                                                 eligible_for:    150
    //
    //                                         Dave:   deposited:       0
    //                                                 last_seen_rate:  0
    //                                                 eligible_for:    0
    context.warp_forward_force_reward_interval_end().unwrap();

    // Carol deposits 150 tokens
    let instruction = deposit(
        &holder_rewards_pool,
        &pool_token,
        &carol_holder_rewards,
        &carol_token,
        &mint,
        &carol.pubkey(),
        150,
    );
    execute_with_payer(&mut context, instruction, Some(&carol)).await;

    // Send 300 rewards
    send_rewards_to_pool(&mut context, &holder_rewards_pool, 300).await;

    // Alice harvest rewards
    let instruction = harvest_rewards(
        &holder_rewards_pool,
        &pool_token,
        &alice_holder_rewards,
        &mint,
        &alice.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&alice)).await;

    // Carol harvest rewards
    let instruction = harvest_rewards(
        &holder_rewards_pool,
        &pool_token,
        &carol_holder_rewards,
        &mint,
        &carol.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&carol)).await;

    // Validate with rewards
    validate_state(
        &mut context,
        &mint,
        Pool {
            total_deposited: 300,
            accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR * 2,
            pool_excess_lamports: 50,
            lamports_last: 50,
        },
        &[
            (
                &alice.pubkey(),
                Holder {
                    last_accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR * 2,
                    deposited: 100,
                    expected_lamports: 200,
                },
            ),
            (
                &bob.pubkey(),
                Holder {
                    last_accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR,
                    deposited: 50,
                    expected_lamports: 50,
                },
            ),
            (
                &carol.pubkey(),
                Holder {
                    last_accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR * 2,
                    deposited: 150,
                    expected_lamports: 150,
                },
            ),
        ],
    )
    .await;

    // Round 3
    //
    // Alice withdraws her tokens, and Dave enters with 100 tokens.
    // 300 rewards are added, which means Bob eligible for 100 rewards
    // (50 from previous round and 50 from this one).
    // Pool balance should be 100 because dave will not harvest this round
    //
    // Pool:   total_deposited:    300         Alice:  deposited:       0
    //         rewards_per_token:  3                   last_seen_rate:  2
    //         available_rewards:  300                 eligible_for:    0
    //
    //                                         Bob:    deposited:       50
    //                                                 last_seen_rate:  3
    //                                                 eligible_for:    100
    //
    //                                         Carol:  deposited:       150
    //                                                 last_seen_rate:  3
    //                                                 eligible_for:    150
    //
    //                                         Dave:   deposited:       100
    //                                                 last_seen_rate:  2
    //                                                 eligible_for:    100
    context.warp_forward_force_reward_interval_end().unwrap();

    // Alice withdraws tokens
    let instruction = withdraw(
        &holder_rewards_pool,
        &pool_token,
        &alice_holder_rewards,
        &alice_token,
        &mint,
        &alice.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&alice)).await;

    // Dave deposits
    let instruction = deposit(
        &holder_rewards_pool,
        &pool_token,
        &dave_holder_rewards,
        &dave_token,
        &mint,
        &dave.pubkey(),
        100,
    );
    execute_with_payer(&mut context, instruction, Some(&dave)).await;

    // Send 300 rewards
    send_rewards_to_pool(&mut context, &holder_rewards_pool, 300).await;

    // Bob harvest rewards
    let instruction = harvest_rewards(
        &holder_rewards_pool,
        &pool_token,
        &bob_holder_rewards,
        &mint,
        &bob.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&bob)).await;

    // Carol harvest rewards
    let instruction = harvest_rewards(
        &holder_rewards_pool,
        &pool_token,
        &carol_holder_rewards,
        &mint,
        &carol.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&carol)).await;

    validate_state(
        &mut context,
        &mint,
        Pool {
            total_deposited: 300,
            accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR * 3,
            pool_excess_lamports: 100,
            lamports_last: 100,
        },
        &[
            (
                &alice.pubkey(),
                Holder {
                    last_accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR * 2,
                    deposited: 0,
                    expected_lamports: 200,
                },
            ),
            (
                &bob.pubkey(),
                Holder {
                    last_accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR * 3,
                    deposited: 50,
                    expected_lamports: 150,
                },
            ),
            (
                &carol.pubkey(),
                Holder {
                    last_accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR * 3,
                    deposited: 150,
                    expected_lamports: 300,
                },
            ),
            (
                &dave.pubkey(),
                Holder {
                    last_accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR * 2,
                    deposited: 100,
                    expected_lamports: 0,
                },
            ),
        ],
    )
    .await;

    // Round 4
    //
    // Alice deposit 100 tokens (was 0), bob closes his account after withdraw.
    // carol deposit another 150 tokens,
    // 500 more rewards are added
    //
    // Pool:   total_deposited:    500         Alice:  deposited:       100
    //         rewards_per_token:  4                   last_seen_rate:  3
    //         available_rewards:  500                 eligible_for:    100
    //
    //                                         Bob:    deposited:       0
    //                                                 last_seen_rate:  0
    //                                                 eligible_for:    0
    //
    //                                         Carol:  deposited:       300
    //                                                 last_seen_rate:  3
    //                                                 eligible_for:    300
    //
    //                                         Dave:   deposited:       100
    //                                                 last_seen_rate:  2
    //                                                 eligible_for:    200
    context.warp_forward_force_reward_interval_end().unwrap();

    // Alice deposits
    let instruction = deposit(
        &holder_rewards_pool,
        &pool_token,
        &alice_holder_rewards,
        &alice_token,
        &mint,
        &alice.pubkey(),
        100,
    );
    execute_with_payer(&mut context, instruction, Some(&alice)).await;

    // Bob withdraws tokens
    let instruction = withdraw(
        &holder_rewards_pool,
        &pool_token,
        &bob_holder_rewards,
        &bob_token,
        &mint,
        &bob.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&bob)).await;

    // Bob closes account
    let instruction = close_holder_rewards(
        &holder_rewards_pool,
        &pool_token,
        &bob_holder_rewards,
        &mint,
        &bob.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&bob)).await;

    // Carol deposits
    let instruction = deposit(
        &holder_rewards_pool,
        &pool_token,
        &carol_holder_rewards,
        &carol_token,
        &mint,
        &carol.pubkey(),
        150,
    );
    execute_with_payer(&mut context, instruction, Some(&carol)).await;

    // Send 500 rewards
    send_rewards_to_pool(&mut context, &holder_rewards_pool, 500).await;

    // Alice harvest rewards
    let instruction = harvest_rewards(
        &holder_rewards_pool,
        &pool_token,
        &alice_holder_rewards,
        &mint,
        &alice.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&alice)).await;

    // Carol harvest rewards
    let instruction = harvest_rewards(
        &holder_rewards_pool,
        &pool_token,
        &carol_holder_rewards,
        &mint,
        &carol.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&carol)).await;

    validate_state(
        &mut context,
        &mint,
        Pool {
            total_deposited: 500,
            accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR * 4,
            pool_excess_lamports: 200,
            lamports_last: 200,
        },
        &[
            (
                &alice.pubkey(),
                Holder {
                    last_accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR * 4,
                    deposited: 100,
                    expected_lamports: 300,
                },
            ),
            (
                &carol.pubkey(),
                Holder {
                    last_accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR * 4,
                    deposited: 300,
                    expected_lamports: 600,
                },
            ),
            (
                &dave.pubkey(),
                Holder {
                    last_accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR * 2,
                    deposited: 100,
                    expected_lamports: 0,
                },
            ),
        ],
    )
    .await;

    // Round 5
    //
    // Alice and Carol wothdraw and close accounts, Bob deposits 100 tokens
    // 200 rewards are added, bob and dave harvest rewards, pool should be left at 0
    //
    // Pool:   total_deposited:    200         Alice:  deposited:       0
    //         rewards_per_token:  5                   last_seen_rate:  0
    //         available_rewards:  200                 eligible_for:    0
    //
    //                                         Bob:    deposited:       100
    //                                                 last_seen_rate:  5
    //                                                 eligible_for:    100
    //
    //                                         Carol:  deposited:       0
    //                                                 last_seen_rate:  0
    //                                                 eligible_for:    0
    //
    //                                         Dave:   deposited:       100
    //                                                 last_seen_rate:  5
    //                                                 eligible_for:    300
    context.warp_forward_force_reward_interval_end().unwrap();

    // Alice withdraws tokens
    let instruction = withdraw(
        &holder_rewards_pool,
        &pool_token,
        &alice_holder_rewards,
        &alice_token,
        &mint,
        &alice.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&alice)).await;

    // Alice closes account
    let instruction = close_holder_rewards(
        &holder_rewards_pool,
        &pool_token,
        &alice_holder_rewards,
        &mint,
        &alice.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&alice)).await;

    // Carol withdraws tokens
    let instruction = withdraw(
        &holder_rewards_pool,
        &pool_token,
        &carol_holder_rewards,
        &carol_token,
        &mint,
        &carol.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&carol)).await;

    // Carol closes account
    let instruction = close_holder_rewards(
        &holder_rewards_pool,
        &pool_token,
        &carol_holder_rewards,
        &mint,
        &carol.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&carol)).await;

    // Bob creates new holder account
    setup_holder_rewards_account(&mut context, &bob_holder_rewards, 0, 0).await;

    // Bob deposits
    let instruction = deposit(
        &holder_rewards_pool,
        &pool_token,
        &bob_holder_rewards,
        &bob_token,
        &mint,
        &bob.pubkey(),
        100,
    );
    execute_with_payer(&mut context, instruction, Some(&bob)).await;

    // Send 200 rewards
    send_rewards_to_pool(&mut context, &holder_rewards_pool, 200).await;

    // Bob harvest rewards
    let instruction = harvest_rewards(
        &holder_rewards_pool,
        &pool_token,
        &bob_holder_rewards,
        &mint,
        &bob.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&bob)).await;

    // Dave harvest rewards
    let instruction = harvest_rewards(
        &holder_rewards_pool,
        &pool_token,
        &dave_holder_rewards,
        &mint,
        &dave.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&dave)).await;

    let holder_rent_exempt_lamports = holder_rent_exempt_lamports(&mut context).await;
    validate_state(
        &mut context,
        &mint,
        Pool {
            total_deposited: 200,
            accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR * 5,
            pool_excess_lamports: 0,
            lamports_last: 0,
        },
        &[
            (
                &bob.pubkey(),
                Holder {
                    last_accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR * 5,
                    deposited: 100,
                    expected_lamports: 150 + 100 + holder_rent_exempt_lamports, // Bob closed account so he got the rent
                },
            ),
            (
                &dave.pubkey(),
                Holder {
                    last_accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR * 5,
                    deposited: 100,
                    expected_lamports: 0 + 300,
                },
            ),
        ],
    )
    .await;

    // Round 6
    //
    // All acconnts are closed after rewards are added without harvesting
    //
    // Pool:   total_deposited:    0         Alice:    deposited:       0
    //         rewards_per_token:  6                   last_seen_rate:  0
    //         available_rewards:  0                   eligible_for:    0
    //
    //                                         Bob:    deposited:       0
    //                                                 last_seen_rate:  0
    //                                                 eligible_for:    0
    //
    //                                         Carol:  deposited:       0
    //                                                 last_seen_rate:  0
    //                                                 eligible_for:    0
    //
    //                                         Dave:   deposited:       0
    //                                                 last_seen_rate:  0
    //                                                 eligible_for:    0
    context.warp_forward_force_reward_interval_end().unwrap();

    // Send 200 rewards
    send_rewards_to_pool(&mut context, &holder_rewards_pool, 200).await;

    // Bob withdraws tokens
    let instruction = withdraw(
        &holder_rewards_pool,
        &pool_token,
        &bob_holder_rewards,
        &bob_token,
        &mint,
        &bob.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&bob)).await;

    // Bob closes account
    let instruction = close_holder_rewards(
        &holder_rewards_pool,
        &pool_token,
        &bob_holder_rewards,
        &mint,
        &bob.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&bob)).await;

    // Dave withdraws tokens
    let instruction = withdraw(
        &holder_rewards_pool,
        &pool_token,
        &dave_holder_rewards,
        &dave_token,
        &mint,
        &dave.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&dave)).await;

    // Bob closes account
    let instruction = close_holder_rewards(
        &holder_rewards_pool,
        &pool_token,
        &dave_holder_rewards,
        &mint,
        &dave.pubkey(),
    );
    execute_with_payer(&mut context, instruction, Some(&dave)).await;

    validate_state(
        &mut context,
        &mint,
        Pool {
            total_deposited: 0,
            accumulated_rewards_per_token: REWARDS_PER_TOKEN_SCALING_FACTOR * 6,
            pool_excess_lamports: 0,
            lamports_last: 0,
        },
        &[],
    )
    .await;

    let wallet_rent_exempt_lamports = wallet_rent_exempt_lamports(&mut context).await;

    // assert bob have correct lamports ammount (rewards + new rewards + closed account lamports)
    let bob_lamports = get_account(&mut context, &bob.pubkey()).await.lamports;
    assert_eq!(
        bob_lamports,
        250 + 100 + wallet_rent_exempt_lamports + (holder_rent_exempt_lamports * 2)
    ); // Bob closed his account twice

    // assert dave have correct lamports ammount (rewards + new rewards + closed account lamports)
    let dave_lamports = get_account(&mut context, &dave.pubkey()).await.lamports;
    assert_eq!(
        dave_lamports,
        300 + 100 + wallet_rent_exempt_lamports + holder_rent_exempt_lamports
    );
}
