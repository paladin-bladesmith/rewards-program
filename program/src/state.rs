//! Program state types.
//!
//!
//! The Paladin rewards program's state plays a critical role in managing
//! each holder's share of rewards in the pool.
//!
//! There are two key components involved in this strategy.
//!
//! 1. The pool state tracks the current rewards per token exchange rate, which
//!    is updated _marginally_ each time new rewards are deposited into the
//!    pool.
//! 2. The holder rewards state records what that exchange rate was when this
//!    holder last harvested rewards.
//!
//! This relationship ensures the system can properly manage changing token
//! supply, caused by either minting new tokens or burning.
//!
//!
//! Consider the following scenario.
//!
//! ```text
//! 
//! -- Legend --
//!
//!     `rewards_per_share`:    Total rewards / token supply.
//!     `available_rewards:`    The number of lamports that can be withdrawn
//!                             from the pool without going below the
//!                             rent-exempt minimum.
//!     `last_seen_rate`:       The rewards per share on the pool when the
//!                             holder last harvested.
//! --
//!
//! Pool:   token_supply:       100         Alice:  last_seen_rate:     0
//!         rewards_per_token:  1                   token_balance:      25
//!         available_rewards:  100                 eligible_for:       25
//!
//!                                         Bob:    last_seen_rate:     0
//!                                                 token_balance:      40
//!                                                 eligible_for:       40
//!
//!                                         Carol:  last_seen_rate:     0
//!                                                 token_balance:      35
//!                                                 eligible_for:       35
//!
//! --> Mint 25 tokens to new holder Dave.
//!
//! When Dave's holder rewards account is created, it records the current
//! rewards per token rate, since Dave can't harvest rewards until new rewards
//! are deposited into the pool.
//!
//! Pool:   token_supply:       125         Alice:  last_seen_rate:     0
//!         rewards_per_token:  1                   token_balance:      25
//!         available_rewards:  100                 eligible_for:       25
//!
//!                                         Bob:    last_seen_rate:     0
//!                                                 token_balance:      40
//!                                                 eligible_for:       40
//!
//!                                         Carol:  last_seen_rate:     0
//!                                                 token_balance:      35
//!                                                 eligible_for:       35
//!
//!                                         Dave:   last_seen_rate:     1
//!                                                 token_balance:      25
//!                                                 eligible_for:       0
//!
//! --> Bob harvests.
//!
//! The rewards per token rate is stored in Bob's holder account state.
//!
//! Pool:   token_supply:       125         Alice:  last_seen_rate:     0
//!         rewards_per_token:  1                   token_balance:      25
//!         available_rewards:  60                  eligible_for:       25
//!
//!                                         Bob:    last_seen_rate:     1
//!                                                 token_balance:      40
//!                                                 eligible_for:       0
//!
//!                                         Carol:  last_seen_rate:     0
//!                                                 token_balance:      35
//!                                                 eligible_for:       35
//!
//!                                         Dave:   last_seen_rate:     1
//!                                                 token_balance:      25
//!                                                 eligible_for:       0
//!
//! --> Alice harvests, then burns all of her tokens.
//!
//! Although Alice has modified the token supply by burning, the pool's rate
//! isn't updated until the next reward distribution, so the remaining holders
//! can still claim rewards at the old rate.
//!
//! Pool:   token_supply:       100         Alice:  last_seen_rate:     1
//!         rewards_per_token:  1                   token_balance:      0
//!         available_rewards:  35                  eligible_for:       0
//!
//!                                         Bob:    last_seen_rate:     1
//!                                                 token_balance:      40
//!                                                 eligible_for:       0
//!
//!                                         Carol:  last_seen_rate:     0
//!                                                 token_balance:      35
//!                                                 eligible_for:       35
//!
//!                                         Dave:   last_seen_rate:     1
//!                                                 token_balance:      25
//!                                                 eligible_for:       0
//!
//! --> 200 rewards are deposited into the pool.
//!
//! The new rate is adjusted by calculating the rewards per token on _only_ the
//! newly added rewards, then adding that rate to the existing rate.
//!
//! That means the new rate is 1 + (200 / 100) = 3.
//!
//! Since the rate has now been updated, Bob becomes eligible for a portion of
//! the newly added rewards.
//!
//! He's eligible for (3 - 1) * 40 = 80 rewards.
//!
//! Dave is now eligible for rewards as well, since he has a non-zero balance.
//!
//! He's eligible for (3 - 1) * 25 = 50 rewards.
//!
//! Pool:   token_supply:       100         Alice:  last_seen_rate:     1
//!         rewards_per_token:  3                   token_balance:      0
//!         available_rewards:  235                 eligible_for:       0
//!
//!                                         Bob:    last_seen_rate:     1
//!                                                 token_balance:      40
//!                                                 eligible_for:       80
//!
//!                                         Carol:  last_seen_rate:     0
//!                                                 token_balance:      35
//!                                                 eligible_for:       105
//!
//!                                         Dave:   last_seen_rate:     1
//!                                                 token_balance:      25
//!                                                 eligible_for:       50
//!
//! Now the total unharvested claims is 80 + 105 + 50 = 235, which is exactly
//! what's availabe in the pool.
//! ```

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
pub fn get_holder_rewards_address(token_account_address: &Pubkey, program_id: &Pubkey) -> Pubkey {
    get_holder_rewards_address_and_bump_seed(token_account_address, program_id).0
}

/// Derive the address of a holder rewards account, with bump seed.
pub fn get_holder_rewards_address_and_bump_seed(
    token_account_address: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &collect_holder_rewards_seeds(token_account_address),
        program_id,
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
pub fn get_holder_rewards_pool_address(mint_address: &Pubkey, program_id: &Pubkey) -> Pubkey {
    get_holder_rewards_pool_address_and_bump_seed(mint_address, program_id).0
}

/// Derive the address of a holder rewards pool account, with bump seed.
pub fn get_holder_rewards_pool_address_and_bump_seed(
    mint_address: &Pubkey,
    program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(&collect_holder_rewards_pool_seeds(mint_address), program_id)
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
    /// The rewards per token exchange rate when this holder last harvested.
    ///
    /// Stored as a `u128`, which includes a scaling factor of `1e9` to
    /// represent the exchange rate with 9 decimal places of precision.
    pub last_accumulated_rewards_per_token: u128,
    /// The amount of unharvested rewards currently stored in the holder
    /// rewards account that can be harvested by the holder.
    pub unharvested_rewards: u64,
    _padding: u64,
}
impl HolderRewards {
    pub fn new(last_accumulated_rewards_per_token: u128, unharvested_rewards: u64) -> Self {
        Self {
            last_accumulated_rewards_per_token,
            unharvested_rewards,
            _padding: 0,
        }
    }
}

/// Tracks the rewards accumulated by the system and manages the distribution
/// of rewards to holders.
///
/// All rewards ready to be distributed are stored directly on this account.
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, ShankAccount, Zeroable)]
#[repr(C)]
pub struct HolderRewardsPool {
    /// The current rewards per token exchange rate.
    ///
    /// Stored as a `u128`, which includes a scaling factor of `1e9` to
    /// represent the exchange rate with 9 decimal places of precision.
    pub accumulated_rewards_per_token: u128,
}
