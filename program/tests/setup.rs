// #![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use {
    paladin_rewards_program::{
        extra_metas::get_extra_account_metas,
        state::{HolderRewards, HolderRewardsPool},
    },
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        pubkey::Pubkey,
        system_program,
    },
    spl_pod::primitives::{PodBool, PodU64},
    spl_tlv_account_resolution::state::ExtraAccountMetaList,
    spl_token_2022::{
        extension::{
            transfer_hook::{TransferHook, TransferHookAccount},
            BaseStateWithExtensionsMut, ExtensionType, PodStateWithExtensionsMut,
        },
        pod::{PodAccount, PodCOption, PodMint},
        state::AccountState,
    },
    spl_transfer_hook_interface::{
        get_extra_account_metas_address, instruction::ExecuteInstruction,
    },
};

pub fn setup() -> ProgramTest {
    ProgramTest::new(
        "paladin_rewards_program",
        paladin_rewards_program::id(),
        processor!(paladin_rewards_program::processor::process),
    )
}

pub async fn setup_mint(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    supply: u64,
) {
    let account_size =
        ExtensionType::try_calculate_account_len::<PodMint>(&[ExtensionType::TransferHook])
            .unwrap();

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(account_size);

    let mut data = vec![0; account_size];
    {
        let mut state =
            PodStateWithExtensionsMut::<PodMint>::unpack_uninitialized(&mut data).unwrap();
        state
            .init_extension::<TransferHook>(true)
            .unwrap()
            .program_id = Some(paladin_rewards_program::id()).try_into().unwrap();
        *state.base = PodMint {
            mint_authority: PodCOption::some(*mint_authority),
            is_initialized: PodBool::from(true),
            supply: PodU64::from(supply),
            ..PodMint::default()
        };
        state.init_account_type().unwrap();
    }

    context.set_account(
        mint,
        &AccountSharedData::from(Account {
            lamports,
            data,
            owner: spl_token_2022::id(),
            ..Account::default()
        }),
    );
}

async fn setup_token_account_common(
    context: &mut ProgramTestContext,
    token_account: &Pubkey,
    owner: &Pubkey,
    mint: &Pubkey,
    amount: u64,
    is_transferring: bool,
) {
    let account_size = ExtensionType::try_calculate_account_len::<PodAccount>(&[
        ExtensionType::TransferHookAccount,
    ])
    .unwrap();

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(account_size);

    let mut data = vec![0; account_size];
    {
        let mut state =
            PodStateWithExtensionsMut::<PodAccount>::unpack_uninitialized(&mut data).unwrap();
        state
            .init_extension::<TransferHookAccount>(true)
            .unwrap()
            .transferring = PodBool::from(is_transferring);
        *state.base = PodAccount {
            amount: PodU64::from(amount),
            mint: *mint,
            owner: *owner,
            state: AccountState::Initialized.into(),
            ..PodAccount::default()
        };
        state.init_account_type().unwrap();
    }

    context.set_account(
        token_account,
        &AccountSharedData::from(Account {
            lamports,
            data,
            owner: spl_token_2022::id(),
            ..Account::default()
        }),
    );
}

pub async fn setup_token_account(
    context: &mut ProgramTestContext,
    token_account: &Pubkey,
    owner: &Pubkey,
    mint: &Pubkey,
    amount: u64,
) {
    setup_token_account_common(context, token_account, owner, mint, amount, false).await;
}

pub async fn setup_token_account_transferring(
    context: &mut ProgramTestContext,
    token_account: &Pubkey,
    owner: &Pubkey,
    mint: &Pubkey,
    amount: u64,
) {
    setup_token_account_common(context, token_account, owner, mint, amount, true).await;
}

#[allow(clippy::arithmetic_side_effects)]
pub async fn setup_system_account(
    context: &mut ProgramTestContext,
    address: &Pubkey,
    excess_lamports: u64,
) {
    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(0) + excess_lamports;

    context.set_account(
        address,
        &AccountSharedData::new(lamports, 0, &system_program::id()),
    );
}

#[allow(clippy::arithmetic_side_effects)]
pub async fn setup_holder_rewards_pool_account(
    context: &mut ProgramTestContext,
    holder_rewards_pool_address: &Pubkey,
    excess_lamports: u64,
    accumulated_rewards_per_token: u128,
) {
    let state = HolderRewardsPool {
        accumulated_rewards_per_token,
    };
    let data = bytemuck::bytes_of(&state).to_vec();

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(data.len()) + excess_lamports;

    context.set_account(
        holder_rewards_pool_address,
        &AccountSharedData::from(Account {
            lamports,
            data,
            owner: paladin_rewards_program::id(),
            ..Account::default()
        }),
    );
}

#[allow(clippy::arithmetic_side_effects)]
pub async fn setup_holder_rewards_account(
    context: &mut ProgramTestContext,
    holder_rewards: &Pubkey,
    unharvested_rewards: u64,
    last_accumulated_rewards_per_token: u128,
) {
    let state = HolderRewards::new(last_accumulated_rewards_per_token, unharvested_rewards);
    let data = bytemuck::bytes_of(&state).to_vec();

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(data.len());

    context.set_account(
        holder_rewards,
        &AccountSharedData::from(Account {
            lamports,
            data,
            owner: paladin_rewards_program::id(),
            ..Account::default()
        }),
    );
}

pub async fn setup_extra_metas_account(context: &mut ProgramTestContext, mint: &Pubkey) {
    let address = get_extra_account_metas_address(mint, &paladin_rewards_program::id());

    let extra_metas = get_extra_account_metas();
    let data_len = ExtraAccountMetaList::size_of(extra_metas.len()).unwrap();

    let mut data = vec![0; data_len];
    ExtraAccountMetaList::init::<ExecuteInstruction>(&mut data, &extra_metas).unwrap();

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(data_len);

    context.set_account(
        &address,
        &AccountSharedData::from(Account {
            lamports,
            data,
            owner: paladin_rewards_program::id(),
            ..Account::default()
        }),
    );
}
