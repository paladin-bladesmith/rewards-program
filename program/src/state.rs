//! Program state types.

use {
    bytemuck::{Pod, Zeroable},
    shank::ShankAccount,
    solana_program::pubkey::Pubkey,
};

/// The seed prefix (`"holder"`) in bytes used to derive the address of a
/// token account's holder rewards account.
/// Seeds: `"holder" + token_account_address`.
pub const SEED_PREFIX_HOLDER_REWARDS: &[u8] = b"holder";
/// The seed prefix (`"holder_pool"`) in bytes used to derive the address of
/// the mint's holder rewards pool account.
/// Seeds: `"holder_pool" + mint_address`.
pub const SEED_PREFIX_HOLDER_REWARDS_POOL: &[u8] = b"holder_pool";

/// Derive the address of a holder rewards account.
pub fn get_holder_rewards_address(token_account_address: &Pubkey) -> Pubkey {
    get_holder_rewards_address_and_bump_seed(token_account_address).0
}

/// Derive the address of a holder rewards account, with bump seed.
pub fn get_holder_rewards_address_and_bump_seed(token_account_address: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &collect_holder_rewards_seeds(token_account_address),
        &crate::id(),
    )
}

pub(crate) fn collect_holder_rewards_seeds(token_account_address: &Pubkey) -> [&[u8]; 2] {
    [SEED_PREFIX_HOLDER_REWARDS, token_account_address.as_ref()]
}

pub(crate) fn collect_holder_rewards_signer_seeds<'a>(
    token_account_address: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [
        SEED_PREFIX_HOLDER_REWARDS,
        token_account_address.as_ref(),
        bump_seed,
    ]
}

/// Derive the address of a holder rewards pool account.
pub fn get_holder_rewards_pool_address(mint_address: &Pubkey) -> Pubkey {
    get_holder_rewards_pool_address_and_bump_seed(mint_address).0
}

/// Derive the address of a holder rewards pool account, with bump seed.
pub fn get_holder_rewards_pool_address_and_bump_seed(mint_address: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &collect_holder_rewards_pool_seeds(mint_address),
        &crate::id(),
    )
}

pub(crate) fn collect_holder_rewards_pool_seeds(mint_address: &Pubkey) -> [&[u8]; 2] {
    [SEED_PREFIX_HOLDER_REWARDS_POOL, mint_address.as_ref()]
}

pub(crate) fn collect_holder_rewards_pool_signer_seeds<'a>(
    mint_address: &'a Pubkey,
    bump_seed: &'a [u8],
) -> [&'a [u8]; 3] {
    [
        SEED_PREFIX_HOLDER_REWARDS_POOL,
        mint_address.as_ref(),
        bump_seed,
    ]
}

/// A holder rewards account which tracks the rewards accumulated by a holder
/// of tokens.
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, ShankAccount, Zeroable)]
#[repr(C)]
pub struct HolderRewards {
    /// The last seen total rewards amount in the aggregate holder rewards
    /// account.
    pub last_seen_total_rewards: u64,
    /// The amount of unharvested rewards currently stored in the holder
    /// rewards account that can be harvested by the holder.
    pub unharvested_rewards: u64,
}

/// Tracks the rewards accumulated by the system and manages the distribution
/// of rewards to holders.
///
/// All rewards ready to be distributed are stored directly on this account.
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, ShankAccount, Zeroable)]
#[repr(C)]
pub struct HolderRewardsPool {
    /// Total holder rewards available for distribution.
    pub total_rewards: u64,
}
