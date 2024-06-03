#![cfg(feature = "test-sbf")]

mod setup;

use {
    paladin_rewards_program::{
        error::PaladinRewardsError,
        instruction::initialize_mint_reward_info,
        state::{get_mint_rewards_address, MintRewards, SEED_PREFIX_HOLDER_REWARDS},
    },
    setup::{setup, setup_mint},
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        instruction::InstructionError,
        program_option::COption,
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
        system_program,
        transaction::{Transaction, TransactionError},
    },
    spl_pod::slice::PodSlice,
    spl_tlv_account_resolution::{
        account::ExtraAccountMeta, seeds::Seed, state::ExtraAccountMetaList,
    },
    spl_token_2022::{
        extension::{
            transfer_fee::TransferFeeConfig, transfer_hook::TransferHook,
            BaseStateWithExtensionsMut, ExtensionType, StateWithExtensionsMut,
        },
        state::Mint,
    },
    spl_transfer_hook_interface::{
        get_extra_account_metas_address, instruction::ExecuteInstruction,
    },
    spl_type_length_value::state::{TlvState, TlvStateBorrowed},
};

#[tokio::test]
async fn fail_invalid_mint() {
    let mint = Pubkey::new_unique();
    let mint_authority = Keypair::new();
    let piggy_bank = Pubkey::new_unique();
    let staked_rewards = Pubkey::new_unique();

    let distribution = get_mint_rewards_address(&mint);
    let extra_metas = get_extra_account_metas_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;

    // Set up a mint with invalid data.
    {
        context.set_account(
            &mint,
            &AccountSharedData::new_data(100_000_000, &vec![5; 165], &spl_token_2022::id())
                .unwrap(),
        );
    }

    let instruction = initialize_mint_reward_info(
        &distribution,
        &extra_metas,
        &mint,
        &mint_authority.pubkey(),
        &piggy_bank,
        &staked_rewards,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
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
async fn fail_incorrect_mint_authority() {
    let mint = Pubkey::new_unique();
    let mint_authority = Keypair::new();
    let piggy_bank = Pubkey::new_unique();
    let staked_rewards = Pubkey::new_unique();

    let distribution = get_mint_rewards_address(&mint);
    let extra_metas = get_extra_account_metas_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;
    setup_mint(
        &mut context,
        &mint,
        &Pubkey::new_unique(), // Incorrect mint authority.
    );

    let instruction = initialize_mint_reward_info(
        &distribution,
        &extra_metas,
        &mint,
        &mint_authority.pubkey(),
        &piggy_bank,
        &staked_rewards,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
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
            InstructionError::Custom(PaladinRewardsError::IncorrectMintAuthority as u32)
        )
    );
}

#[tokio::test]
async fn fail_mint_authority_not_signer() {
    let mint = Pubkey::new_unique();
    let mint_authority = Keypair::new();
    let piggy_bank = Pubkey::new_unique();
    let staked_rewards = Pubkey::new_unique();

    let distribution = get_mint_rewards_address(&mint);
    let extra_metas = get_extra_account_metas_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;
    setup_mint(&mut context, &mint, &mint_authority.pubkey());

    let mut instruction = initialize_mint_reward_info(
        &distribution,
        &extra_metas,
        &mint,
        &mint_authority.pubkey(),
        &piggy_bank,
        &staked_rewards,
    );
    instruction.accounts[3].is_signer = false;

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer], // Missing mint authority.
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
async fn fail_mint_missing_transfer_hook_extension() {
    let mint = Pubkey::new_unique();
    let mint_authority = Keypair::new();
    let piggy_bank = Pubkey::new_unique();
    let staked_rewards = Pubkey::new_unique();

    let distribution = get_mint_rewards_address(&mint);
    let extra_metas = get_extra_account_metas_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;

    // Set up a mint without a `TransferHook` extension.
    {
        let account_size = ExtensionType::try_calculate_account_len::<Mint>(&[
            ExtensionType::TransferFeeConfig, // Not the correct extension.
        ])
        .unwrap();
        let mut account_data = vec![0; account_size];
        let mut state =
            StateWithExtensionsMut::<Mint>::unpack_uninitialized(&mut account_data).unwrap();
        state.init_extension::<TransferFeeConfig>(true).unwrap();
        state.base = Mint {
            mint_authority: COption::Some(mint_authority.pubkey()),
            is_initialized: true,
            ..Mint::default()
        };
        state.pack_base();
        state.init_account_type().unwrap();

        context.set_account(
            &mint,
            &AccountSharedData::from(Account {
                lamports: 1_000_000_000,
                data: account_data,
                owner: spl_token_2022::id(),
                ..Account::default()
            }),
        );
    }

    let instruction = initialize_mint_reward_info(
        &distribution,
        &extra_metas,
        &mint,
        &mint_authority.pubkey(),
        &piggy_bank,
        &staked_rewards,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
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
async fn fail_incorrect_transfer_hook_program_id() {
    let mint = Pubkey::new_unique();
    let mint_authority = Keypair::new();
    let piggy_bank = Pubkey::new_unique();
    let staked_rewards = Pubkey::new_unique();

    let distribution = get_mint_rewards_address(&mint);
    let extra_metas = get_extra_account_metas_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;

    // Set up a mint with a `TransferHook` extension, but with the wrong
    // program ID.
    {
        let account_size = ExtensionType::try_calculate_account_len::<Mint>(&[
            ExtensionType::TransferHook, // Correct extension.
        ])
        .unwrap();
        let mut account_data = vec![0; account_size];
        let mut state =
            StateWithExtensionsMut::<Mint>::unpack_uninitialized(&mut account_data).unwrap();
        state.init_extension::<TransferHook>(true).unwrap();
        state
            .get_extension_mut::<TransferHook>()
            .unwrap()
            .program_id = Some(Pubkey::new_unique()).try_into().unwrap(); // Incorrect program ID.
        state.base = Mint {
            mint_authority: COption::Some(mint_authority.pubkey()),
            is_initialized: true,
            ..Mint::default()
        };
        state.pack_base();
        state.init_account_type().unwrap();

        context.set_account(
            &mint,
            &AccountSharedData::from(Account {
                lamports: 1_000_000_000,
                data: account_data,
                owner: spl_token_2022::id(),
                ..Account::default()
            }),
        );
    }

    let instruction = initialize_mint_reward_info(
        &distribution,
        &extra_metas,
        &mint,
        &mint_authority.pubkey(),
        &piggy_bank,
        &staked_rewards,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
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
            InstructionError::Custom(PaladinRewardsError::IncorrectTransferHookProgramId as u32)
        )
    );
}

#[tokio::test]
async fn fail_incorrect_distribution_address() {
    let mint = Pubkey::new_unique();
    let mint_authority = Keypair::new();
    let piggy_bank = Pubkey::new_unique();
    let staked_rewards = Pubkey::new_unique();

    let distribution = Pubkey::new_unique(); // Incorrect distribution address.
    let extra_metas = get_extra_account_metas_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;
    setup_mint(&mut context, &mint, &mint_authority.pubkey());

    let instruction = initialize_mint_reward_info(
        &distribution,
        &extra_metas,
        &mint,
        &mint_authority.pubkey(),
        &piggy_bank,
        &staked_rewards,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
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
            InstructionError::Custom(
                PaladinRewardsError::IncorrectDistributionAccountAddress as u32
            )
        )
    );
}

#[tokio::test]
async fn fail_distribution_account_initialized() {
    let mint = Pubkey::new_unique();
    let mint_authority = Keypair::new();
    let piggy_bank = Pubkey::new_unique();
    let staked_rewards = Pubkey::new_unique();

    let distribution = get_mint_rewards_address(&mint);
    let extra_metas = get_extra_account_metas_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;
    setup_mint(&mut context, &mint, &mint_authority.pubkey());

    // Set up an initialized distribution account.
    {
        context.set_account(
            &distribution,
            &AccountSharedData::from(Account {
                lamports: 1_000_000_000,
                data: vec![2; 45],
                owner: paladin_rewards_program::id(),
                ..Account::default()
            }),
        );
    }

    let instruction = initialize_mint_reward_info(
        &distribution,
        &extra_metas,
        &mint,
        &mint_authority.pubkey(),
        &piggy_bank,
        &staked_rewards,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
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
        TransactionError::InstructionError(0, InstructionError::AccountAlreadyInitialized)
    );
}

#[tokio::test]
async fn fail_incorrect_extra_metas_address() {
    let mint = Pubkey::new_unique();
    let mint_authority = Keypair::new();
    let piggy_bank = Pubkey::new_unique();
    let staked_rewards = Pubkey::new_unique();

    let distribution = get_mint_rewards_address(&mint);
    let extra_metas = Pubkey::new_unique(); // Incorrect extra metas address.

    let mut context = setup().start_with_context().await;
    setup_mint(&mut context, &mint, &mint_authority.pubkey());

    let instruction = initialize_mint_reward_info(
        &distribution,
        &extra_metas,
        &mint,
        &mint_authority.pubkey(),
        &piggy_bank,
        &staked_rewards,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
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
            InstructionError::Custom(PaladinRewardsError::IncorrectExtraMetasAccountAddress as u32)
        )
    );
}

#[tokio::test]
async fn fail_extra_metas_account_initialized() {
    let mint = Pubkey::new_unique();
    let mint_authority = Keypair::new();
    let piggy_bank = Pubkey::new_unique();
    let staked_rewards = Pubkey::new_unique();

    let distribution = get_mint_rewards_address(&mint);
    let extra_metas = get_extra_account_metas_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;
    setup_mint(&mut context, &mint, &mint_authority.pubkey());

    // Set up an initialized extra metas account.
    {
        context.set_account(
            &extra_metas,
            &AccountSharedData::from(Account {
                lamports: 1_000_000_000,
                data: vec![2; 45],
                owner: paladin_rewards_program::id(),
                ..Account::default()
            }),
        );
    }

    let instruction = initialize_mint_reward_info(
        &distribution,
        &extra_metas,
        &mint,
        &mint_authority.pubkey(),
        &piggy_bank,
        &staked_rewards,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
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
        TransactionError::InstructionError(0, InstructionError::AccountAlreadyInitialized)
    );
}

#[tokio::test]
async fn success() {
    let mint = Pubkey::new_unique();
    let mint_authority = Keypair::new();
    let piggy_bank = Pubkey::new_unique();
    let staked_rewards = Pubkey::new_unique();

    let distribution = get_mint_rewards_address(&mint);
    let extra_metas = get_extra_account_metas_address(&mint, &paladin_rewards_program::id());

    let mut context = setup().start_with_context().await;
    setup_mint(&mut context, &mint, &mint_authority.pubkey());

    // Fund the distribution account and extra metas account.
    {
        let rent = context.banks_client.get_rent().await.unwrap();
        let lamports = rent.minimum_balance(std::mem::size_of::<MintRewards>());
        context.set_account(
            &distribution,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
        let lamports = rent.minimum_balance(ExtraAccountMetaList::size_of(5).unwrap());
        context.set_account(
            &extra_metas,
            &AccountSharedData::new(lamports, 0, &system_program::id()),
        );
    }

    let instruction = initialize_mint_reward_info(
        &distribution,
        &extra_metas,
        &mint,
        &mint_authority.pubkey(),
        &piggy_bank,
        &staked_rewards,
    );

    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&context.payer.pubkey()),
        &[&context.payer, &mint_authority],
        context.last_blockhash,
    );

    context
        .banks_client
        .process_transaction(transaction)
        .await
        .unwrap();

    // Check the distribution account.
    let distribution_account = context
        .banks_client
        .get_account(distribution)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        bytemuck::from_bytes::<MintRewards>(&distribution_account.data),
        &MintRewards {
            piggy_bank_address: piggy_bank,
            staked_pal_rewards_address: staked_rewards,
            total_rewards: 0,
        }
    );

    // Check the extra metas account.
    let extra_metas_account = context
        .banks_client
        .get_account(extra_metas)
        .await
        .unwrap()
        .unwrap();
    let state = TlvStateBorrowed::unpack(&extra_metas_account.data).unwrap();
    let bytes = state.get_first_bytes::<ExecuteInstruction>().unwrap();
    let extra_account_metas = PodSlice::<ExtraAccountMeta>::unpack(bytes).unwrap();
    assert_eq!(
        extra_account_metas.data(),
        &[
            // 5: Token-2022 program.
            ExtraAccountMeta::new_with_pubkey(&spl_token_2022::id(), false, false).unwrap(),
            // 6: Associated Token program.
            ExtraAccountMeta::new_with_pubkey(&spl_associated_token_account::id(), false, false)
                .unwrap(),
            // 7: Holder token account.
            ExtraAccountMeta::new_external_pda_with_seeds(
                6, // Associated Token program.
                &[
                    Seed::AccountKey {
                        index: 3, // Source owner.
                    },
                    Seed::AccountKey {
                        index: 5, // Token-2022 program.
                    },
                    Seed::AccountKey {
                        index: 1, // Mint.
                    },
                ],
                false,
                false,
            )
            .unwrap(),
            // 8: Holder rewards.
            ExtraAccountMeta::new_with_seeds(
                &[
                    Seed::Literal {
                        bytes: SEED_PREFIX_HOLDER_REWARDS.to_vec(),
                    },
                    Seed::AccountKey {
                        index: 7, // Holder token account.
                    },
                ],
                false,
                true,
            )
            .unwrap(),
            // 9: Distribution account.
            ExtraAccountMeta::new_with_pubkey(&distribution, false, true).unwrap(),
        ],
    );
}
