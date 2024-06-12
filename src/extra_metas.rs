use {
    crate::state::{SEED_PREFIX_HOLDER_REWARDS, SEED_PREFIX_HOLDER_REWARDS_POOL},
    spl_tlv_account_resolution::{account::ExtraAccountMeta, seeds::Seed},
};

/// Extra account metas required by the Paladin Rewards program's SPL
/// Transfer Hook Interface implementation.
///
/// Accounts required (* = extra meta):
///
/// 0. `[ ]` Source token account.
/// 1. `[ ]` Token mint.
/// 2. `[ ]` Destination token account.
/// 3. `[ ]` Source owner.
/// 4. `[ ]` Extra account metas account.
/// 5. `[ ]` * Holder rewards pool account.
/// 6. `[w]` * Source holder rewards account.
/// 7. `[w]` * Destination holder rewards account.
pub fn get_extra_account_metas() -> [ExtraAccountMeta; 3] {
    [
        // Holder rewards pool account.
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: SEED_PREFIX_HOLDER_REWARDS_POOL.to_vec(),
                },
                Seed::AccountKey {
                    index: 1, // Mint.
                },
            ],
            false,
            false,
        )
        .unwrap(),
        // Source holder rewards account.
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: SEED_PREFIX_HOLDER_REWARDS.to_vec(),
                },
                Seed::AccountKey {
                    index: 0, // Source token account.
                },
            ],
            false,
            true,
        )
        .unwrap(),
        // Destination holder rewards account.
        ExtraAccountMeta::new_with_seeds(
            &[
                Seed::Literal {
                    bytes: SEED_PREFIX_HOLDER_REWARDS.to_vec(),
                },
                Seed::AccountKey {
                    index: 2, // Destination token account.
                },
            ],
            false,
            true,
        )
        .unwrap(),
    ]
}
