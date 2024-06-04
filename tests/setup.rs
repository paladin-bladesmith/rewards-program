#![cfg(feature = "test-sbf")]

use {
    solana_program_test::*,
    solana_sdk::{
        account::{Account, AccountSharedData},
        program_option::COption,
        pubkey::Pubkey,
    },
    spl_token_2022::{
        extension::{
            transfer_hook::TransferHook, BaseStateWithExtensionsMut, ExtensionType,
            StateWithExtensionsMut,
        },
        state::Mint,
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
        ExtensionType::try_calculate_account_len::<Mint>(&[ExtensionType::TransferHook]).unwrap();

    let rent = context.banks_client.get_rent().await.unwrap();
    let lamports = rent.minimum_balance(account_size);

    let mut data = vec![0; account_size];
    {
        let mut state = StateWithExtensionsMut::<Mint>::unpack_uninitialized(&mut data).unwrap();
        state.init_extension::<TransferHook>(true).unwrap();
        state
            .get_extension_mut::<TransferHook>()
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
