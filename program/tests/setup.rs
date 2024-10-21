#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use {
    paladin_rewards_program::{
        extra_metas::get_extra_account_metas,
        state::{HolderRewards, HolderRewardsPool},
    },
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        program_option::COption,
        pubkey::Pubkey,
        system_program,
    },
    spl_pod::primitives::PodBool,
    spl_tlv_account_resolution::state::ExtraAccountMetaList,
    spl_token_2022::{
        extension::{
            transfer_hook::{TransferHook, TransferHookAccount},
            BaseStateWithExtensionsMut, ExtensionType, StateWithExtensionsMut,
        },
        state::{Account as TokenAccount, AccountState, Mint},
    },
    spl_transfer_hook_interface::{
        get_extra_account_metas_address, instruction::ExecuteInstruction,
    },
};

pub fn setup() -> ProgramTest {
    ProgramTest::new(
        "paladin_rewards_program",
        paladin_rewards_program::id(),
        None,
    )
}

pub async fn setup_mint(
    context: &mut ProgramTestContext,
    mint: &Pubkey,
    mint_authority: &Pubkey,
    supply: u64,
) {
    let account_size =
        ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::TransferHook]).unwrap();

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(account_size);

    let mut data = vec![0; account_size];
    {
        let mut state = StateWithExtensionsMut::<Mint>::unpack_uninitialized(&mut data).unwrap();
        state
            .init_extension::<TransferHook>(true)
            .unwrap()
            .program_id = Some(paladin_rewards_program::id()).try_into().unwrap();
        state.base = Mint {
            mint_authority: COption::Some(*mint_authority),
            is_initialized: true,
            supply,
            ..Mint::default()
        };
        state.pack_base();
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
    let account_size = ExtensionType::try_calculate_account_len::<TokenAccount>(&[
        ExtensionType::TransferHookAccount,
    ])
    .unwrap();

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(account_size);

    let mut data = vec![0; account_size];
    {
        let mut state =
            StateWithExtensionsMut::<TokenAccount>::unpack_uninitialized(&mut data).unwrap();
        state
            .init_extension::<TransferHookAccount>(true)
            .unwrap()
            .transferring = PodBool::from(is_transferring);
        state.base = TokenAccount {
            amount,
            mint: *mint,
            owner: *owner,
            state: AccountState::Initialized,
            ..TokenAccount::default()
        };
        state.pack_base();
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

pub async fn setup_sponsor(context: &mut ProgramTestContext, sponsor: &Pubkey) {
    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(0);

    context.set_account(
        sponsor,
        &AccountSharedData::from(Account {
            lamports,
            ..Account::default()
        }),
    )
}

pub async fn setup_rent_exempt_account(
    context: &mut ProgramTestContext,
    address: &Pubkey,
    excess_lamports: u64,
    owner: &Pubkey,
) {
    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(0) + excess_lamports;

    context.set_account(address, &AccountSharedData::new(lamports, 0, owner));
}

#[allow(clippy::arithmetic_side_effects)]
pub async fn setup_system_account(
    context: &mut ProgramTestContext,
    address: &Pubkey,
    excess_lamports: u64,
) {
    setup_rent_exempt_account(context, address, excess_lamports, &system_program::id()).await;
}

#[allow(clippy::arithmetic_side_effects)]
pub async fn setup_holder_rewards_pool_account(
    context: &mut ProgramTestContext,
    holder_rewards_pool_address: &Pubkey,
    excess_lamports: u64,
    accumulated_rewards_per_token: u128,
) {
    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(HolderRewardsPool::LEN) + excess_lamports;
    let state = HolderRewardsPool {
        accumulated_rewards_per_token,
        lamports_last: lamports,
        _padding: 0,
    };
    let data = bytemuck::bytes_of(&state).to_vec();

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
    rent_sponsor: Option<(Pubkey, u64)>,
) {
    let rent = context.banks_client.get_rent().await.unwrap();
    let (rent_sponsor, rent_debt, minimum_balance) = rent_sponsor
        .map(|(sponsor, minimum_balance)| {
            (
                sponsor,
                rent.minimum_balance(HolderRewards::LEN) * 11 / 10,
                minimum_balance,
            )
        })
        .unwrap_or_default();
    let state = HolderRewards {
        last_accumulated_rewards_per_token,
        unharvested_rewards,
        rent_sponsor,
        rent_debt,
        minimum_balance,
        _padding: 0,
    };
    let data = bytemuck::bytes_of(&state).to_vec();

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
