//! End-to-end test.

#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_rewards_program::{
        extra_metas::get_extra_account_metas,
        instruction::{
            distribute_rewards, harvest_rewards, initialize_holder_rewards,
            initialize_holder_rewards_pool,
        },
        state::{
            get_holder_rewards_address, get_holder_rewards_pool_address, HolderRewards,
            HolderRewardsPool,
        },
    },
    setup::{setup, setup_mint, setup_token_account},
    solana_program_test::*,
    solana_sdk::{
        account::Account, instruction::Instruction, pubkey::Pubkey, signature::Keypair,
        signer::Signer, system_instruction, transaction::Transaction,
    },
    spl_associated_token_account::get_associated_token_address,
    spl_tlv_account_resolution::state::ExtraAccountMetaList,
    spl_token_2022::{
        extension::StateWithExtensions,
        state::{Account as TokenAccount, Mint},
    },
    spl_transfer_hook_interface::{
        get_extra_account_metas_address, offchain::add_extra_account_metas_for_execute,
    },
};

struct Pool {
    token_supply: u64,
    accumulated_rewards_per_token: u128,
    pool_excess_lamports: u64,
}

struct Holder {
    token_account_balance: u64,
    last_accumulated_rewards_per_token: u128,
    unharvested_rewards: u64,
}

async fn extra_metas_rent_exempt_lamports(context: &mut ProgramTestContext) -> u64 {
    let extra_metas = get_extra_account_metas();
    let account_size = ExtraAccountMetaList::size_of(extra_metas.len()).unwrap();
    context
        .banks_client
        .get_rent()
        .await
        .expect("get_rent")
        .minimum_balance(account_size as usize)
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

async fn send_transaction(
    context: &mut ProgramTestContext,
    instructions: &[Instruction],
    signers: &[&Keypair],
) {
    let transaction = Transaction::new_signed_with_payer(
        instructions,
        Some(&context.payer.pubkey()),
        signers,
        context.last_blockhash,
    );
    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();
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
        let mint_data = get_account(context, mint).await.data;
        let mint_state = StateWithExtensions::<Mint>::unpack(&mint_data).unwrap();
        assert_eq!(mint_state.base.supply, pool.token_supply);

        let pool_address = get_holder_rewards_pool_address(mint);
        let pool_account = get_account(context, &pool_address).await;

        let pool_excess_lamports = pool_account.lamports - pool_rent_exempt_lamports(context).await;
        assert_eq!(pool_excess_lamports, pool.pool_excess_lamports);

        let pool_state = bytemuck::from_bytes::<HolderRewardsPool>(&pool_account.data);
        assert_eq!(
            pool_state,
            &HolderRewardsPool {
                accumulated_rewards_per_token: pool.accumulated_rewards_per_token,
            }
        );
    }
    // Then evaluate the holders.
    for (token_account_address, checks) in holder_rewards {
        let token_account = get_account(context, token_account_address).await;
        let token_account_state =
            StateWithExtensions::<TokenAccount>::unpack(&token_account.data).unwrap();
        assert_eq!(
            token_account_state.base.amount,
            checks.token_account_balance
        );

        let holder_rewards_address = get_holder_rewards_address(token_account_address);
        let holder_rewards_account = get_account(context, &holder_rewards_address).await;
        let holder_rewards_state =
            bytemuck::from_bytes::<HolderRewards>(&holder_rewards_account.data);
        assert_eq!(
            holder_rewards_state,
            &HolderRewards::new(
                checks.last_accumulated_rewards_per_token,
                checks.unharvested_rewards
            )
        );
    }
}

#[tokio::test]
async fn test_e2e() {
    let mint = Pubkey::new_unique();
    let mint_authority = Keypair::new();

    let holder_rewards_pool = get_holder_rewards_pool_address(&mint);

    let alice = Keypair::new();
    let alice_token_account = get_associated_token_address(&alice.pubkey(), &mint);

    let bob = Keypair::new();
    let bob_token_account = get_associated_token_address(&bob.pubkey(), &mint);

    let carol = Keypair::new();
    let carol_token_account = get_associated_token_address(&carol.pubkey(), &mint);

    let dave = Keypair::new();
    let dave_token_account = get_associated_token_address(&dave.pubkey(), &mint);

    let mut context = setup().start_with_context().await;
    let payer = context.payer.insecure_clone();

    // Initial environment.
    //
    // Pool:   token_supply:       100         Alice:  last_seen_rate:     0
    //         rewards_per_token:  1                   token_balance:      25
    //         available_rewards:  100                 eligible_for:       25
    //
    //                                         Bob:    last_seen_rate:     0
    //                                                 token_balance:      40
    //                                                 eligible_for:       40
    //
    //                                         Carol:  last_seen_rate:     0
    //                                                 token_balance:      35
    //                                                 eligible_for:       35
    {
        // Setup.
        {
            // Initial pool setup.
            setup_mint(&mut context, &mint, &mint_authority.pubkey(), 100).await;

            // Initial holders setup.
            setup_token_account(
                &mut context,
                &alice_token_account,
                &alice.pubkey(),
                &mint,
                25,
            )
            .await;
            setup_token_account(&mut context, &bob_token_account, &bob.pubkey(), &mint, 40).await;
            setup_token_account(
                &mut context,
                &carol_token_account,
                &carol.pubkey(),
                &mint,
                35,
            )
            .await;
        }

        // Create the pool.
        {
            let extra_metas =
                get_extra_account_metas_address(&mint, &paladin_rewards_program::id());
            let pool_rent_exempt_lamports = pool_rent_exempt_lamports(&mut context).await;
            let extra_metas_rent_exempt_lamports =
                extra_metas_rent_exempt_lamports(&mut context).await;

            send_transaction(
                &mut context,
                &[
                    system_instruction::transfer(
                        &payer.pubkey(),
                        &holder_rewards_pool,
                        pool_rent_exempt_lamports,
                    ),
                    system_instruction::transfer(
                        &payer.pubkey(),
                        &extra_metas,
                        extra_metas_rent_exempt_lamports,
                    ),
                    initialize_holder_rewards_pool(
                        &holder_rewards_pool,
                        &extra_metas,
                        &mint,
                        &mint_authority.pubkey(),
                    ),
                ],
                &[&payer, &mint_authority],
            )
            .await;

            validate_state(
                &mut context,
                &mint,
                Pool {
                    token_supply: 100,
                    accumulated_rewards_per_token: 0,
                    pool_excess_lamports: 0,
                },
                &[],
            )
            .await;
        }

        // Create the holders.
        {
            let alice_holder_rewards = get_holder_rewards_address(&alice_token_account);
            let bob_holder_rewards = get_holder_rewards_address(&bob_token_account);
            let carol_holder_rewards = get_holder_rewards_address(&carol_token_account);

            let rent_exempt_lamports = holder_rent_exempt_lamports(&mut context).await;

            send_transaction(
                &mut context,
                &[
                    system_instruction::transfer(
                        &payer.pubkey(),
                        &alice_holder_rewards,
                        rent_exempt_lamports,
                    ),
                    initialize_holder_rewards(
                        &holder_rewards_pool,
                        &alice_holder_rewards,
                        &alice_token_account,
                        &mint,
                    ),
                    system_instruction::transfer(
                        &payer.pubkey(),
                        &bob_holder_rewards,
                        rent_exempt_lamports,
                    ),
                    initialize_holder_rewards(
                        &holder_rewards_pool,
                        &bob_holder_rewards,
                        &bob_token_account,
                        &mint,
                    ),
                    system_instruction::transfer(
                        &payer.pubkey(),
                        &carol_holder_rewards,
                        rent_exempt_lamports,
                    ),
                    initialize_holder_rewards(
                        &holder_rewards_pool,
                        &carol_holder_rewards,
                        &carol_token_account,
                        &mint,
                    ),
                ],
                &[&payer],
            )
            .await;

            validate_state(
                &mut context,
                &mint,
                Pool {
                    token_supply: 100,
                    accumulated_rewards_per_token: 0,
                    pool_excess_lamports: 0,
                },
                &[
                    (
                        &alice_token_account,
                        Holder {
                            token_account_balance: 25,
                            last_accumulated_rewards_per_token: 0,
                            unharvested_rewards: 0,
                        },
                    ),
                    (
                        &bob_token_account,
                        Holder {
                            token_account_balance: 40,
                            last_accumulated_rewards_per_token: 0,
                            unharvested_rewards: 0,
                        },
                    ),
                    (
                        &carol_token_account,
                        Holder {
                            token_account_balance: 35,
                            last_accumulated_rewards_per_token: 0,
                            unharvested_rewards: 0,
                        },
                    ),
                ],
            )
            .await;
        }

        // Distribute the first reward.
        {
            send_transaction(
                &mut context,
                &[distribute_rewards(
                    &payer.pubkey(),
                    &holder_rewards_pool,
                    &mint,
                    100,
                )],
                &[&payer],
            )
            .await;

            validate_state(
                &mut context,
                &mint,
                Pool {
                    token_supply: 100,
                    accumulated_rewards_per_token: 1_000_000_000,
                    pool_excess_lamports: 100,
                },
                &[],
            )
            .await;
        }
    }

    // --> Mint 25 tokens to new holder Dave.
    //
    // When Dave's holder rewards account is created, it records the current
    // rewards per token rate, since Dave can't harvest rewards until new rewards
    // are deposited into the pool.
    //
    // Pool:   token_supply:       125         Alice:  last_seen_rate:     0
    //         rewards_per_token:  1                   token_balance:      25
    //         available_rewards:  100                 eligible_for:       25
    //
    //                                         Bob:    last_seen_rate:     0
    //                                                 token_balance:      40
    //                                                 eligible_for:       40
    //
    //                                         Carol:  last_seen_rate:     0
    //                                                 token_balance:      35
    //                                                 eligible_for:       35
    //
    //                                         Dave:   last_seen_rate:     1
    //                                                 token_balance:      25
    //                                                 eligible_for:       0
    {
        // Set up Dave's token account.
        setup_token_account(&mut context, &dave_token_account, &dave.pubkey(), &mint, 0).await;

        let dave_holder_rewards = get_holder_rewards_address(&dave_token_account);
        let rent_exempt_lamports = holder_rent_exempt_lamports(&mut context).await;

        send_transaction(
            &mut context,
            &[
                spl_token_2022::instruction::mint_to(
                    &spl_token_2022::id(),
                    &mint,
                    &dave_token_account,
                    &mint_authority.pubkey(),
                    &[],
                    25,
                )
                .unwrap(),
                system_instruction::transfer(
                    &payer.pubkey(),
                    &dave_holder_rewards,
                    rent_exempt_lamports,
                ),
                initialize_holder_rewards(
                    &holder_rewards_pool,
                    &dave_holder_rewards,
                    &dave_token_account,
                    &mint,
                ),
            ],
            &[&payer, &mint_authority],
        )
        .await;

        validate_state(
            &mut context,
            &mint,
            Pool {
                token_supply: 125,
                accumulated_rewards_per_token: 1_000_000_000,
                pool_excess_lamports: 100,
            },
            &[
                (
                    &alice_token_account,
                    Holder {
                        token_account_balance: 25,
                        last_accumulated_rewards_per_token: 0,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &bob_token_account,
                    Holder {
                        token_account_balance: 40,
                        last_accumulated_rewards_per_token: 0,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &carol_token_account,
                    Holder {
                        token_account_balance: 35,
                        last_accumulated_rewards_per_token: 0,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &dave_token_account,
                    Holder {
                        token_account_balance: 25,
                        last_accumulated_rewards_per_token: 1_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
            ],
        )
        .await;
    }

    // --> Bob harvests.
    //
    // The rewards per token rate is stored in Bob's holder account state.
    //
    // Pool:   token_supply:       125         Alice:  last_seen_rate:     0
    //         rewards_per_token:  1                   token_balance:      25
    //         available_rewards:  60                  eligible_for:       25
    //
    //                                         Bob:    last_seen_rate:     1
    //                                                 token_balance:      40
    //                                                 eligible_for:       0
    //
    //                                         Carol:  last_seen_rate:     0
    //                                                 token_balance:      35
    //                                                 eligible_for:       35
    //
    //                                         Dave:   last_seen_rate:     1
    //                                                 token_balance:      25
    //                                                 eligible_for:       0
    {
        let bob_holder_rewards = get_holder_rewards_address(&bob_token_account);

        send_transaction(
            &mut context,
            &[harvest_rewards(
                &holder_rewards_pool,
                &bob_holder_rewards,
                &bob_token_account,
                &mint,
            )],
            &[&payer],
        )
        .await;

        validate_state(
            &mut context,
            &mint,
            Pool {
                token_supply: 125,
                accumulated_rewards_per_token: 1_000_000_000,
                pool_excess_lamports: 60,
            },
            &[
                (
                    &alice_token_account,
                    Holder {
                        token_account_balance: 25,
                        last_accumulated_rewards_per_token: 0,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &bob_token_account,
                    Holder {
                        token_account_balance: 40,
                        last_accumulated_rewards_per_token: 1_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &carol_token_account,
                    Holder {
                        token_account_balance: 35,
                        last_accumulated_rewards_per_token: 0,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &dave_token_account,
                    Holder {
                        token_account_balance: 25,
                        last_accumulated_rewards_per_token: 1_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
            ],
        )
        .await;
    }

    // --> Alice harvests, then burns all of her tokens.
    //
    // Although Alice has modified the token supply by burning, the pool's rate
    // isn't updated until the next reward distribution, so the remaining holders
    // can still claim rewards at the old rate.
    //
    // Pool:   token_supply:       100         Alice:  last_seen_rate:     1
    //         rewards_per_token:  1                   token_balance:      0
    //         available_rewards:  35                  eligible_for:       0
    //
    //                                         Bob:    last_seen_rate:     1
    //                                                 token_balance:      40
    //                                                 eligible_for:       0
    //
    //                                         Carol:  last_seen_rate:     0
    //                                                 token_balance:      35
    //                                                 eligible_for:       35
    //
    //                                         Dave:   last_seen_rate:     1
    //                                                 token_balance:      25
    //                                                 eligible_for:       0
    {
        let alice_holder_rewards = get_holder_rewards_address(&alice_token_account);

        let alice_starting_lamports = get_account(&mut context, &alice_token_account)
            .await
            .lamports;

        send_transaction(
            &mut context,
            &[
                harvest_rewards(
                    &holder_rewards_pool,
                    &alice_holder_rewards,
                    &alice_token_account,
                    &mint,
                ),
                spl_token_2022::instruction::burn(
                    &spl_token_2022::id(),
                    &alice_token_account,
                    &mint,
                    &alice.pubkey(),
                    &[],
                    25,
                )
                .unwrap(),
            ],
            &[&payer, &alice],
        )
        .await;

        validate_state(
            &mut context,
            &mint,
            Pool {
                token_supply: 100,
                accumulated_rewards_per_token: 1_000_000_000,
                pool_excess_lamports: 35,
            },
            &[
                (
                    &alice_token_account,
                    Holder {
                        token_account_balance: 0,
                        last_accumulated_rewards_per_token: 1_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &bob_token_account,
                    Holder {
                        token_account_balance: 40,
                        last_accumulated_rewards_per_token: 1_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &carol_token_account,
                    Holder {
                        token_account_balance: 35,
                        last_accumulated_rewards_per_token: 0,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &dave_token_account,
                    Holder {
                        token_account_balance: 25,
                        last_accumulated_rewards_per_token: 1_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
            ],
        )
        .await;

        let alice_ending_lamports = get_account(&mut context, &alice_token_account)
            .await
            .lamports;

        // Alice harvested 25 rewards.
        assert_eq!(alice_ending_lamports - alice_starting_lamports, 25);
    }

    // --> 200 rewards are deposited into the pool.
    //
    // The new rate is adjusted by calculating the rewards per token on _only_ the
    // newly added rewards, then adding that rate to the existing rate.
    //
    // That means the new rate is 1 + (200 / 100) = 3.
    //
    // Since the rate has now been updated, Bob becomes eligible for a portion of
    // the newly added rewards.
    //
    // He's eligible for (3 - 1) * 40 = 80 rewards.
    //
    // Dave is now eligible for rewards as well, since he has a non-zero balance.
    //
    // He's eligible for (3 - 1) * 25 = 50 rewards.
    //
    // Pool:   token_supply:       100         Alice:  last_seen_rate:     1
    //         rewards_per_token:  3                   token_balance:      0
    //         available_rewards:  235                 eligible_for:       0
    //
    //                                         Bob:    last_seen_rate:     1
    //                                                 token_balance:      40
    //                                                 eligible_for:       80
    //
    //                                         Carol:  last_seen_rate:     0
    //                                                 token_balance:      35
    //                                                 eligible_for:       105
    //
    //                                         Dave:   last_seen_rate:     1
    //                                                 token_balance:      25
    //                                                 eligible_for:       50
    {
        send_transaction(
            &mut context,
            &[distribute_rewards(
                &payer.pubkey(),
                &holder_rewards_pool,
                &mint,
                200,
            )],
            &[&payer],
        )
        .await;

        validate_state(
            &mut context,
            &mint,
            Pool {
                token_supply: 100,
                accumulated_rewards_per_token: 3_000_000_000,
                pool_excess_lamports: 235,
            },
            &[
                (
                    &alice_token_account,
                    Holder {
                        token_account_balance: 0,
                        last_accumulated_rewards_per_token: 1_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &bob_token_account,
                    Holder {
                        token_account_balance: 40,
                        last_accumulated_rewards_per_token: 1_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &carol_token_account,
                    Holder {
                        token_account_balance: 35,
                        last_accumulated_rewards_per_token: 0,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &dave_token_account,
                    Holder {
                        token_account_balance: 25,
                        last_accumulated_rewards_per_token: 1_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
            ],
        )
        .await;
    }

    // --> Carol transfers all of her tokens to Dave.
    //
    // As a result of the transfer hook, Carol's eligible rewards move to her
    // holder rewards account's `unharvested_rewards`. Dave's eligible rewards
    // are also moved to his `unharvested_rewards`. Both holders have their
    // `last_seen_rate` updated to the current rate.
    //
    // Now, when either of them go to harvest, they'll see no "new" rewards
    // calculated from the marginal rate, but they can harvest their
    // unharvested rewards.
    //
    // When the next rewards is distributed, the marginal rate will apply to
    // each holder's new token balance.
    //
    // Pool:   token_supply:       100         Alice:  last_seen_rate:     1
    //         rewards_per_token:  3                   token_balance:      0
    //         available_rewards:  235                 eligible_for:       0
    //                                                 unharvested:        0
    //
    //                                         Bob:    last_seen_rate:     1
    //                                                 token_balance:      40
    //                                                 eligible_for:       80
    //                                                 unharvested:        0
    //
    //                                         Carol:  last_seen_rate:     3
    //                                                 token_balance:      0
    //                                                 eligible_for:       0
    //                                                 unharvested:        105
    //
    //                                         Dave:   last_seen_rate:     3
    //                                                 token_balance:      60
    //                                                 eligible_for:       0
    //                                                 unharvested:        50
    {
        let instruction = transfer_with_extra_metas_instruction(
            &mut context,
            &carol_token_account,
            &mint,
            &dave_token_account,
            &carol.pubkey(),
            35,
            0,
        )
        .await;
        send_transaction(&mut context, &[instruction], &[&payer, &carol]).await;

        validate_state(
            &mut context,
            &mint,
            Pool {
                token_supply: 100,
                accumulated_rewards_per_token: 3_000_000_000,
                pool_excess_lamports: 235,
            },
            &[
                (
                    &alice_token_account,
                    Holder {
                        token_account_balance: 0,
                        last_accumulated_rewards_per_token: 1_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &bob_token_account,
                    Holder {
                        token_account_balance: 40,
                        last_accumulated_rewards_per_token: 1_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &carol_token_account,
                    Holder {
                        token_account_balance: 0,
                        last_accumulated_rewards_per_token: 3_000_000_000,
                        unharvested_rewards: 105,
                    },
                ),
                (
                    &dave_token_account,
                    Holder {
                        token_account_balance: 60,
                        last_accumulated_rewards_per_token: 3_000_000_000,
                        unharvested_rewards: 50,
                    },
                ),
            ],
        )
        .await;
    }

    // --> 300 rewards are deposited into the pool.
    //
    // The new rate is 3 + (300 / 100) = 6.
    //
    // Each holder's share of the new rewards is calculated from the marginal
    // rate applied to their token account balance.
    //
    // Alice: (6 - 1) * 0 = 0
    // Bob:   (6 - 1) * 40 = 200
    // Carol: (6 - 3) * 0 = 0
    // Dave:  (6 - 3) * 60 = 180
    //
    // Pool:   token_supply:       100         Alice:  last_seen_rate:     1
    //         rewards_per_token:  6                   token_balance:      0
    //         available_rewards:  535                 eligible_for:       0
    //                                                 unharvested:        0
    //
    //                                         Bob:    last_seen_rate:     1
    //                                                 token_balance:      40
    //                                                 eligible_for:       200
    //                                                 unharvested:        0
    //
    //                                         Carol:  last_seen_rate:     3
    //                                                 token_balance:      0
    //                                                 eligible_for:       0
    //                                                 unharvested:        105
    //
    //                                         Dave:   last_seen_rate:     3
    //                                                 token_balance:      60
    //                                                 eligible_for:       180
    //                                                 unharvested:        50
    {
        send_transaction(
            &mut context,
            &[distribute_rewards(
                &payer.pubkey(),
                &holder_rewards_pool,
                &mint,
                300,
            )],
            &[&payer],
        )
        .await;

        validate_state(
            &mut context,
            &mint,
            Pool {
                token_supply: 100,
                accumulated_rewards_per_token: 6_000_000_000,
                pool_excess_lamports: 535,
            },
            &[
                (
                    &alice_token_account,
                    Holder {
                        token_account_balance: 0,
                        last_accumulated_rewards_per_token: 1_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &bob_token_account,
                    Holder {
                        token_account_balance: 40,
                        last_accumulated_rewards_per_token: 1_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &carol_token_account,
                    Holder {
                        token_account_balance: 0,
                        last_accumulated_rewards_per_token: 3_000_000_000,
                        unharvested_rewards: 105,
                    },
                ),
                (
                    &dave_token_account,
                    Holder {
                        token_account_balance: 60,
                        last_accumulated_rewards_per_token: 3_000_000_000,
                        unharvested_rewards: 50,
                    },
                ),
            ],
        )
        .await;
    }

    // --> Bob, Carol and Dave harvest.
    //
    // Pool:   token_supply:       100         Alice:  last_seen_rate:     1
    //         rewards_per_token:  6                   token_balance:      0
    //         available_rewards:  0                   eligible_for:       0
    //                                                 unharvested:        0
    //
    //                                         Bob:    last_seen_rate:     6
    //                                                 token_balance:      40
    //                                                 eligible_for:       0
    //                                                 unharvested:        0
    //
    //                                         Carol:  last_seen_rate:     6
    //                                                 token_balance:      0
    //                                                 eligible_for:       0
    //                                                 unharvested:        0
    //
    //                                         Dave:   last_seen_rate:     6
    //                                                 token_balance:      60
    //                                                 eligible_for:       0
    //                                                 unharvested:        0
    {
        let bob_holder_rewards = get_holder_rewards_address(&bob_token_account);
        let carol_holder_rewards = get_holder_rewards_address(&carol_token_account);
        let dave_holder_rewards = get_holder_rewards_address(&dave_token_account);

        let bob_starting_lamports = get_account(&mut context, &bob_token_account).await.lamports;
        let carol_starting_lamports = get_account(&mut context, &carol_token_account)
            .await
            .lamports;
        let dave_starting_lamports = get_account(&mut context, &dave_token_account)
            .await
            .lamports;

        send_transaction(
            &mut context,
            &[
                harvest_rewards(
                    &holder_rewards_pool,
                    &bob_holder_rewards,
                    &bob_token_account,
                    &mint,
                ),
                harvest_rewards(
                    &holder_rewards_pool,
                    &carol_holder_rewards,
                    &carol_token_account,
                    &mint,
                ),
                harvest_rewards(
                    &holder_rewards_pool,
                    &dave_holder_rewards,
                    &dave_token_account,
                    &mint,
                ),
            ],
            &[&payer],
        )
        .await;

        validate_state(
            &mut context,
            &mint,
            Pool {
                token_supply: 100,
                accumulated_rewards_per_token: 6_000_000_000,
                pool_excess_lamports: 0,
            },
            &[
                (
                    &alice_token_account,
                    Holder {
                        token_account_balance: 0,
                        last_accumulated_rewards_per_token: 1_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &bob_token_account,
                    Holder {
                        token_account_balance: 40,
                        last_accumulated_rewards_per_token: 6_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &carol_token_account,
                    Holder {
                        token_account_balance: 0,
                        last_accumulated_rewards_per_token: 6_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
                (
                    &dave_token_account,
                    Holder {
                        token_account_balance: 60,
                        last_accumulated_rewards_per_token: 6_000_000_000,
                        unharvested_rewards: 0,
                    },
                ),
            ],
        )
        .await;

        let bob_ending_lamports = get_account(&mut context, &bob_token_account).await.lamports;
        let carol_ending_lamports = get_account(&mut context, &carol_token_account)
            .await
            .lamports;
        let dave_ending_lamports = get_account(&mut context, &dave_token_account)
            .await
            .lamports;

        // Bob harvested 200 rewards.
        assert_eq!(bob_ending_lamports - bob_starting_lamports, 200);

        // Carol harvested 105 rewards.
        assert_eq!(carol_ending_lamports - carol_starting_lamports, 105);

        // Dave harvested 180 + 50 = 230 rewards.
        assert_eq!(dave_ending_lamports - dave_starting_lamports, 230);
    }
}
