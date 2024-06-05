//! Program state types.

use {
    bytemuck::{Pod, Zeroable},
    solana_program::pubkey::Pubkey,
};

/// The seed prefix (`"holder"`) in bytes used to derive the address of the
/// holder rewards account. Seeds: `"holder" + token_account_address`.
pub const SEED_PREFIX_HOLDER_REWARDS: &[u8] = b"holder";
/// The seed prefix (`"staker"`) in bytes used to derive the address of the
/// staker rewards account. Seeds: `"staker" + mint_address`.
pub const SEED_PREFIX_STAKER_REWARDS: &[u8] = b"staker";

/// Derive the address of a holder rewards account.
pub fn get_holder_rewards_address(token_account_address: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[SEED_PREFIX_HOLDER_REWARDS, token_account_address.as_ref()],
        &crate::id(),
    )
    .0
}

/// Derive the address of a staker rewards account.
pub fn get_staker_rewards_address(mint_address: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[SEED_PREFIX_STAKER_REWARDS, mint_address.as_ref()],
        &crate::id(),
    )
    .0
}

/// A holder rewards account which tracks the rewards accumulated by a holder
/// of PAL tokens.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct HolderRewards {
    /// The amount of unharvested rewards currently stored in the holder
    /// rewards account that can be harvested by the holder.
    pub unharvested_rewards: u64,
}

/// Tracks the rewards accumulated by the system and manages the distribution
/// of rewards to stakers and holders.
///
/// All rewards ready to be distributed are stored directly on this account.
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[repr(C)]
pub struct StakerRewards {
    /// The address of the piggy bank account.
    pub piggy_bank_address: Pubkey,
    /// Total holder rewards available for distribution.
    pub total_holder_rewards: u64,
}
