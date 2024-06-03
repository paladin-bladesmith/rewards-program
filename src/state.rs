//! Program state types.

use {
    bytemuck::{Pod, Zeroable},
    solana_program::pubkey::Pubkey,
};

/// The seed prefix (`"holder"`) in bytes used to derive the address of the
/// holder rewards account. Seeds: `"holder" + token_account_address`.
pub const SEED_PREFIX_HOLDER_REWARDS: &[u8] = b"holder";
/// The seed prefix (`"mint"`) in bytes used to derive the address of the
/// mint rewards account. Seeds: `"mint" + mint_address`.
pub const SEED_PREFIX_MINT_REWARDS: &[u8] = b"mint";

/// Derive the address of a holder rewards account.
pub fn get_holder_rewards_address(token_account_address: &Pubkey) -> Pubkey {
    get_holder_rewards_address_and_bump_seed(token_account_address).0
}

pub fn get_holder_rewards_address_and_bump_seed(token_account_address: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[SEED_PREFIX_HOLDER_REWARDS, token_account_address.as_ref()],
        &crate::id(),
    )
}

/// Derive the address of a mint rewards account.
pub fn get_mint_rewards_address(mint_address: &Pubkey) -> Pubkey {
    get_mint_rewards_address_and_bump_seed(mint_address).0
}

pub fn get_mint_rewards_address_and_bump_seed(mint_address: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[SEED_PREFIX_MINT_REWARDS, mint_address.as_ref()],
        &crate::id(),
    )
}

/// A holder rewards account which tracks the rewards accumulated by a holder
/// of PAL tokens.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct HolderRewards {
    /// The amount of rewards that can be harvested by the holder.
    pub credits: u64,
}

impl HolderRewards {
    /// Creates a new [HolderRewards](struct.HolderRewards.html) instance.
    pub fn new(credits: u64) -> Self {
        Self { credits }
    }
}

/// Tracks the rewards accumulated by the system and manages the distribution
/// of rewards to stakers.
///
/// All rewards ready to be distributed are stored directly on this account.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct MintRewards {
    /// Running counter of all rewards accumulated by the system over time.
    pub total_rewards: u64,
    /// The address of the piggy bank account.
    pub piggy_bank_address: Pubkey,
    /// The addresses of all staked PAL rewards accounts.
    /// Stored as a slice.
    pub staked_pal_rewards_address: Pubkey,
}

impl MintRewards {
    /// Creates a new [MintRewards](struct.MintRewards.html) instance.
    pub fn new(piggy_bank_address: &Pubkey, staked_pal_rewards_address: &Pubkey) -> Self {
        Self {
            total_rewards: 0,
            piggy_bank_address: *piggy_bank_address,
            staked_pal_rewards_address: *staked_pal_rewards_address,
        }
    }
}
