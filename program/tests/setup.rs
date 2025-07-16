#![cfg(feature = "test-sbf")]
#![allow(dead_code)]

use {
    paladin_rewards_program::state::{HolderRewards, HolderRewardsPool},
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        program_pack::Pack,
        pubkey::Pubkey,
        system_program,
    },
    spl_token::state::{Account as TokenAccount, AccountState, Mint},
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
    supply: u64,
    mint_authority: Option<Pubkey>,
) {
    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(spl_token::state::Mint::LEN);

    let mut data = vec![0; spl_token::state::Mint::LEN];
    {
        let state = Mint {
            is_initialized: true,
            supply,
            mint_authority: mint_authority.try_into().unwrap(),
            ..Mint::default()
        };
        Mint::pack(state, &mut data).unwrap();
    }

    context.set_account(
        mint,
        &AccountSharedData::from(Account {
            lamports,
            data,
            owner: spl_token::id(),
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
) {
    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(TokenAccount::LEN);

    let mut data = vec![0; TokenAccount::LEN];
    {
        let state = TokenAccount {
            amount,
            mint: *mint,
            owner: *owner,
            state: AccountState::Initialized,
            ..TokenAccount::default()
        };
        TokenAccount::pack(state, &mut data).unwrap();
    }

    context.set_account(
        token_account,
        &AccountSharedData::from(Account {
            lamports,
            data,
            owner: spl_token::id(),
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
    setup_token_account_common(context, token_account, owner, mint, amount).await;
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
    total_deposited: u64,
    last_accumulated_rewards_per_token: u128,
) {
    let state = HolderRewards {
        last_accumulated_rewards_per_token,
        total_deposited,
        _padding: 0,
    };
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
