//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

use borsh::{BorshDeserialize, BorshSerialize};

/// Accounts.
pub struct InitializeHolderRewards {
    /// Holder rewards pool account.
    pub holder_rewards_pool: solana_program::pubkey::Pubkey,
    /// Holder rewards pool token account.
    pub holder_rewards_pool_token_account: solana_program::pubkey::Pubkey,
    /// Token account owner.
    pub owner: solana_program::pubkey::Pubkey,
    /// Holder rewards account.
    pub holder_rewards: solana_program::pubkey::Pubkey,
    /// Token mint.
    pub mint: solana_program::pubkey::Pubkey,
    /// System program.
    pub system_program: solana_program::pubkey::Pubkey,
}

impl InitializeHolderRewards {
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(&[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(6 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.holder_rewards_pool,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.holder_rewards_pool_token_account,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.owner, true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.holder_rewards,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.mint, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.system_program,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let data = InitializeHolderRewardsInstructionData::new()
            .try_to_vec()
            .unwrap();

        solana_program::instruction::Instruction {
            program_id: crate::PALADIN_REWARDS_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InitializeHolderRewardsInstructionData {
    discriminator: u8,
}

impl InitializeHolderRewardsInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 1 }
    }
}

impl Default for InitializeHolderRewardsInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Instruction builder for `InitializeHolderRewards`.
///
/// ### Accounts:
///
///   0. `[writable]` holder_rewards_pool
///   1. `[]` holder_rewards_pool_token_account
///   2. `[writable, signer]` owner
///   3. `[writable]` holder_rewards
///   4. `[]` mint
///   5. `[optional]` system_program (default to
///      `11111111111111111111111111111111`)
#[derive(Clone, Debug, Default)]
pub struct InitializeHolderRewardsBuilder {
    holder_rewards_pool: Option<solana_program::pubkey::Pubkey>,
    holder_rewards_pool_token_account: Option<solana_program::pubkey::Pubkey>,
    owner: Option<solana_program::pubkey::Pubkey>,
    holder_rewards: Option<solana_program::pubkey::Pubkey>,
    mint: Option<solana_program::pubkey::Pubkey>,
    system_program: Option<solana_program::pubkey::Pubkey>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl InitializeHolderRewardsBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    /// Holder rewards pool account.
    #[inline(always)]
    pub fn holder_rewards_pool(
        &mut self,
        holder_rewards_pool: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.holder_rewards_pool = Some(holder_rewards_pool);
        self
    }
    /// Holder rewards pool token account.
    #[inline(always)]
    pub fn holder_rewards_pool_token_account(
        &mut self,
        holder_rewards_pool_token_account: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.holder_rewards_pool_token_account = Some(holder_rewards_pool_token_account);
        self
    }
    /// Token account owner.
    #[inline(always)]
    pub fn owner(&mut self, owner: solana_program::pubkey::Pubkey) -> &mut Self {
        self.owner = Some(owner);
        self
    }
    /// Holder rewards account.
    #[inline(always)]
    pub fn holder_rewards(&mut self, holder_rewards: solana_program::pubkey::Pubkey) -> &mut Self {
        self.holder_rewards = Some(holder_rewards);
        self
    }
    /// Token mint.
    #[inline(always)]
    pub fn mint(&mut self, mint: solana_program::pubkey::Pubkey) -> &mut Self {
        self.mint = Some(mint);
        self
    }
    /// `[optional account, default to '11111111111111111111111111111111']`
    /// System program.
    #[inline(always)]
    pub fn system_program(&mut self, system_program: solana_program::pubkey::Pubkey) -> &mut Self {
        self.system_program = Some(system_program);
        self
    }
    /// Add an additional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: solana_program::instruction::AccountMeta,
    ) -> &mut Self {
        self.__remaining_accounts.push(account);
        self
    }
    /// Add additional accounts to the instruction.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[solana_program::instruction::AccountMeta],
    ) -> &mut Self {
        self.__remaining_accounts.extend_from_slice(accounts);
        self
    }
    #[allow(clippy::clone_on_copy)]
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        let accounts = InitializeHolderRewards {
            holder_rewards_pool: self
                .holder_rewards_pool
                .expect("holder_rewards_pool is not set"),
            holder_rewards_pool_token_account: self
                .holder_rewards_pool_token_account
                .expect("holder_rewards_pool_token_account is not set"),
            owner: self.owner.expect("owner is not set"),
            holder_rewards: self.holder_rewards.expect("holder_rewards is not set"),
            mint: self.mint.expect("mint is not set"),
            system_program: self
                .system_program
                .unwrap_or(solana_program::pubkey!("11111111111111111111111111111111")),
        };

        accounts.instruction_with_remaining_accounts(&self.__remaining_accounts)
    }
}

/// `initialize_holder_rewards` CPI accounts.
pub struct InitializeHolderRewardsCpiAccounts<'a, 'b> {
    /// Holder rewards pool account.
    pub holder_rewards_pool: &'b solana_program::account_info::AccountInfo<'a>,
    /// Holder rewards pool token account.
    pub holder_rewards_pool_token_account: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token account owner.
    pub owner: &'b solana_program::account_info::AccountInfo<'a>,
    /// Holder rewards account.
    pub holder_rewards: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token mint.
    pub mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// System program.
    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `initialize_holder_rewards` CPI instruction.
pub struct InitializeHolderRewardsCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Holder rewards pool account.
    pub holder_rewards_pool: &'b solana_program::account_info::AccountInfo<'a>,
    /// Holder rewards pool token account.
    pub holder_rewards_pool_token_account: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token account owner.
    pub owner: &'b solana_program::account_info::AccountInfo<'a>,
    /// Holder rewards account.
    pub holder_rewards: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token mint.
    pub mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// System program.
    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
}

impl<'a, 'b> InitializeHolderRewardsCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: InitializeHolderRewardsCpiAccounts<'a, 'b>,
    ) -> Self {
        Self {
            __program: program,
            holder_rewards_pool: accounts.holder_rewards_pool,
            holder_rewards_pool_token_account: accounts.holder_rewards_pool_token_account,
            owner: accounts.owner,
            holder_rewards: accounts.holder_rewards,
            mint: accounts.mint,
            system_program: accounts.system_program,
        }
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], &[])
    }
    #[inline(always)]
    pub fn invoke_with_remaining_accounts(
        &self,
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(&[], remaining_accounts)
    }
    #[inline(always)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed_with_remaining_accounts(signers_seeds, &[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed_with_remaining_accounts(
        &self,
        signers_seeds: &[&[&[u8]]],
        remaining_accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> solana_program::entrypoint::ProgramResult {
        let mut accounts = Vec::with_capacity(6 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.holder_rewards_pool.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.holder_rewards_pool_token_account.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.owner.key,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.holder_rewards.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.mint.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.system_program.key,
            false,
        ));
        remaining_accounts.iter().for_each(|remaining_account| {
            accounts.push(solana_program::instruction::AccountMeta {
                pubkey: *remaining_account.0.key,
                is_signer: remaining_account.1,
                is_writable: remaining_account.2,
            })
        });
        let data = InitializeHolderRewardsInstructionData::new()
            .try_to_vec()
            .unwrap();

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::PALADIN_REWARDS_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(6 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.holder_rewards_pool.clone());
        account_infos.push(self.holder_rewards_pool_token_account.clone());
        account_infos.push(self.owner.clone());
        account_infos.push(self.holder_rewards.clone());
        account_infos.push(self.mint.clone());
        account_infos.push(self.system_program.clone());
        remaining_accounts
            .iter()
            .for_each(|remaining_account| account_infos.push(remaining_account.0.clone()));

        if signers_seeds.is_empty() {
            solana_program::program::invoke(&instruction, &account_infos)
        } else {
            solana_program::program::invoke_signed(&instruction, &account_infos, signers_seeds)
        }
    }
}

/// Instruction builder for `InitializeHolderRewards` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` holder_rewards_pool
///   1. `[]` holder_rewards_pool_token_account
///   2. `[writable, signer]` owner
///   3. `[writable]` holder_rewards
///   4. `[]` mint
///   5. `[]` system_program
#[derive(Clone, Debug)]
pub struct InitializeHolderRewardsCpiBuilder<'a, 'b> {
    instruction: Box<InitializeHolderRewardsCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> InitializeHolderRewardsCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(InitializeHolderRewardsCpiBuilderInstruction {
            __program: program,
            holder_rewards_pool: None,
            holder_rewards_pool_token_account: None,
            owner: None,
            holder_rewards: None,
            mint: None,
            system_program: None,
            __remaining_accounts: Vec::new(),
        });
        Self { instruction }
    }
    /// Holder rewards pool account.
    #[inline(always)]
    pub fn holder_rewards_pool(
        &mut self,
        holder_rewards_pool: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.holder_rewards_pool = Some(holder_rewards_pool);
        self
    }
    /// Holder rewards pool token account.
    #[inline(always)]
    pub fn holder_rewards_pool_token_account(
        &mut self,
        holder_rewards_pool_token_account: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.holder_rewards_pool_token_account =
            Some(holder_rewards_pool_token_account);
        self
    }
    /// Token account owner.
    #[inline(always)]
    pub fn owner(&mut self, owner: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.owner = Some(owner);
        self
    }
    /// Holder rewards account.
    #[inline(always)]
    pub fn holder_rewards(
        &mut self,
        holder_rewards: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.holder_rewards = Some(holder_rewards);
        self
    }
    /// Token mint.
    #[inline(always)]
    pub fn mint(&mut self, mint: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.mint = Some(mint);
        self
    }
    /// System program.
    #[inline(always)]
    pub fn system_program(
        &mut self,
        system_program: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.system_program = Some(system_program);
        self
    }
    /// Add an additional account to the instruction.
    #[inline(always)]
    pub fn add_remaining_account(
        &mut self,
        account: &'b solana_program::account_info::AccountInfo<'a>,
        is_writable: bool,
        is_signer: bool,
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .push((account, is_writable, is_signer));
        self
    }
    /// Add additional accounts to the instruction.
    ///
    /// Each account is represented by a tuple of the `AccountInfo`, a `bool`
    /// indicating whether the account is writable or not, and a `bool`
    /// indicating whether the account is a signer or not.
    #[inline(always)]
    pub fn add_remaining_accounts(
        &mut self,
        accounts: &[(
            &'b solana_program::account_info::AccountInfo<'a>,
            bool,
            bool,
        )],
    ) -> &mut Self {
        self.instruction
            .__remaining_accounts
            .extend_from_slice(accounts);
        self
    }
    #[inline(always)]
    pub fn invoke(&self) -> solana_program::entrypoint::ProgramResult {
        self.invoke_signed(&[])
    }
    #[allow(clippy::clone_on_copy)]
    #[allow(clippy::vec_init_then_push)]
    pub fn invoke_signed(
        &self,
        signers_seeds: &[&[&[u8]]],
    ) -> solana_program::entrypoint::ProgramResult {
        let instruction = InitializeHolderRewardsCpi {
            __program: self.instruction.__program,

            holder_rewards_pool: self
                .instruction
                .holder_rewards_pool
                .expect("holder_rewards_pool is not set"),

            holder_rewards_pool_token_account: self
                .instruction
                .holder_rewards_pool_token_account
                .expect("holder_rewards_pool_token_account is not set"),

            owner: self.instruction.owner.expect("owner is not set"),

            holder_rewards: self
                .instruction
                .holder_rewards
                .expect("holder_rewards is not set"),

            mint: self.instruction.mint.expect("mint is not set"),

            system_program: self
                .instruction
                .system_program
                .expect("system_program is not set"),
        };
        instruction.invoke_signed_with_remaining_accounts(
            signers_seeds,
            &self.instruction.__remaining_accounts,
        )
    }
}

#[derive(Clone, Debug)]
struct InitializeHolderRewardsCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    holder_rewards_pool: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    holder_rewards_pool_token_account: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    owner: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    holder_rewards: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    mint: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    system_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}
