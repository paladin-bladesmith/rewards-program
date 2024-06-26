//! This code was AUTOGENERATED using the kinobi library.
//! Please DO NOT EDIT THIS FILE, instead use visitors
//! to add features, then rerun kinobi to update it.
//!
//! <https://github.com/kinobi-so/kinobi>

use borsh::{BorshDeserialize, BorshSerialize};

/// Accounts.
pub struct InitializeHolderRewardsPool {
    /// Holder rewards pool account.
    pub holder_rewards_pool: solana_program::pubkey::Pubkey,
    /// Transfer hook extra account metas account.
    pub extra_account_metas: solana_program::pubkey::Pubkey,
    /// Token mint.
    pub mint: solana_program::pubkey::Pubkey,
    /// Mint authority.
    pub mint_authority: solana_program::pubkey::Pubkey,
    /// System program.
    pub system_program: solana_program::pubkey::Pubkey,
}

impl InitializeHolderRewardsPool {
    pub fn instruction(&self) -> solana_program::instruction::Instruction {
        self.instruction_with_remaining_accounts(&[])
    }
    #[allow(clippy::vec_init_then_push)]
    pub fn instruction_with_remaining_accounts(
        &self,
        remaining_accounts: &[solana_program::instruction::AccountMeta],
    ) -> solana_program::instruction::Instruction {
        let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.holder_rewards_pool,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            self.extra_account_metas,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.mint, false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.mint_authority,
            true,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            self.system_program,
            false,
        ));
        accounts.extend_from_slice(remaining_accounts);
        let data = InitializeHolderRewardsPoolInstructionData::new()
            .try_to_vec()
            .unwrap();

        solana_program::instruction::Instruction {
            program_id: crate::REWARDS_ID,
            accounts,
            data,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InitializeHolderRewardsPoolInstructionData {
    discriminator: u8,
}

impl InitializeHolderRewardsPoolInstructionData {
    pub fn new() -> Self {
        Self { discriminator: 0 }
    }
}

impl Default for InitializeHolderRewardsPoolInstructionData {
    fn default() -> Self {
        Self::new()
    }
}

/// Instruction builder for `InitializeHolderRewardsPool`.
///
/// ### Accounts:
///
///   0. `[writable]` holder_rewards_pool
///   1. `[writable]` extra_account_metas
///   2. `[]` mint
///   3. `[signer]` mint_authority
///   4. `[optional]` system_program (default to
///      `11111111111111111111111111111111`)
#[derive(Clone, Debug, Default)]
pub struct InitializeHolderRewardsPoolBuilder {
    holder_rewards_pool: Option<solana_program::pubkey::Pubkey>,
    extra_account_metas: Option<solana_program::pubkey::Pubkey>,
    mint: Option<solana_program::pubkey::Pubkey>,
    mint_authority: Option<solana_program::pubkey::Pubkey>,
    system_program: Option<solana_program::pubkey::Pubkey>,
    __remaining_accounts: Vec<solana_program::instruction::AccountMeta>,
}

impl InitializeHolderRewardsPoolBuilder {
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
    /// Transfer hook extra account metas account.
    #[inline(always)]
    pub fn extra_account_metas(
        &mut self,
        extra_account_metas: solana_program::pubkey::Pubkey,
    ) -> &mut Self {
        self.extra_account_metas = Some(extra_account_metas);
        self
    }
    /// Token mint.
    #[inline(always)]
    pub fn mint(&mut self, mint: solana_program::pubkey::Pubkey) -> &mut Self {
        self.mint = Some(mint);
        self
    }
    /// Mint authority.
    #[inline(always)]
    pub fn mint_authority(&mut self, mint_authority: solana_program::pubkey::Pubkey) -> &mut Self {
        self.mint_authority = Some(mint_authority);
        self
    }
    /// `[optional account, default to '11111111111111111111111111111111']`
    /// System program.
    #[inline(always)]
    pub fn system_program(&mut self, system_program: solana_program::pubkey::Pubkey) -> &mut Self {
        self.system_program = Some(system_program);
        self
    }
    /// Add an aditional account to the instruction.
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
        let accounts = InitializeHolderRewardsPool {
            holder_rewards_pool: self
                .holder_rewards_pool
                .expect("holder_rewards_pool is not set"),
            extra_account_metas: self
                .extra_account_metas
                .expect("extra_account_metas is not set"),
            mint: self.mint.expect("mint is not set"),
            mint_authority: self.mint_authority.expect("mint_authority is not set"),
            system_program: self
                .system_program
                .unwrap_or(solana_program::pubkey!("11111111111111111111111111111111")),
        };

        accounts.instruction_with_remaining_accounts(&self.__remaining_accounts)
    }
}

/// `initialize_holder_rewards_pool` CPI accounts.
pub struct InitializeHolderRewardsPoolCpiAccounts<'a, 'b> {
    /// Holder rewards pool account.
    pub holder_rewards_pool: &'b solana_program::account_info::AccountInfo<'a>,
    /// Transfer hook extra account metas account.
    pub extra_account_metas: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token mint.
    pub mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// Mint authority.
    pub mint_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// System program.
    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
}

/// `initialize_holder_rewards_pool` CPI instruction.
pub struct InitializeHolderRewardsPoolCpi<'a, 'b> {
    /// The program to invoke.
    pub __program: &'b solana_program::account_info::AccountInfo<'a>,
    /// Holder rewards pool account.
    pub holder_rewards_pool: &'b solana_program::account_info::AccountInfo<'a>,
    /// Transfer hook extra account metas account.
    pub extra_account_metas: &'b solana_program::account_info::AccountInfo<'a>,
    /// Token mint.
    pub mint: &'b solana_program::account_info::AccountInfo<'a>,
    /// Mint authority.
    pub mint_authority: &'b solana_program::account_info::AccountInfo<'a>,
    /// System program.
    pub system_program: &'b solana_program::account_info::AccountInfo<'a>,
}

impl<'a, 'b> InitializeHolderRewardsPoolCpi<'a, 'b> {
    pub fn new(
        program: &'b solana_program::account_info::AccountInfo<'a>,
        accounts: InitializeHolderRewardsPoolCpiAccounts<'a, 'b>,
    ) -> Self {
        Self {
            __program: program,
            holder_rewards_pool: accounts.holder_rewards_pool,
            extra_account_metas: accounts.extra_account_metas,
            mint: accounts.mint,
            mint_authority: accounts.mint_authority,
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
        let mut accounts = Vec::with_capacity(5 + remaining_accounts.len());
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.holder_rewards_pool.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new(
            *self.extra_account_metas.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.mint.key,
            false,
        ));
        accounts.push(solana_program::instruction::AccountMeta::new_readonly(
            *self.mint_authority.key,
            true,
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
        let data = InitializeHolderRewardsPoolInstructionData::new()
            .try_to_vec()
            .unwrap();

        let instruction = solana_program::instruction::Instruction {
            program_id: crate::REWARDS_ID,
            accounts,
            data,
        };
        let mut account_infos = Vec::with_capacity(5 + 1 + remaining_accounts.len());
        account_infos.push(self.__program.clone());
        account_infos.push(self.holder_rewards_pool.clone());
        account_infos.push(self.extra_account_metas.clone());
        account_infos.push(self.mint.clone());
        account_infos.push(self.mint_authority.clone());
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

/// Instruction builder for `InitializeHolderRewardsPool` via CPI.
///
/// ### Accounts:
///
///   0. `[writable]` holder_rewards_pool
///   1. `[writable]` extra_account_metas
///   2. `[]` mint
///   3. `[signer]` mint_authority
///   4. `[]` system_program
#[derive(Clone, Debug)]
pub struct InitializeHolderRewardsPoolCpiBuilder<'a, 'b> {
    instruction: Box<InitializeHolderRewardsPoolCpiBuilderInstruction<'a, 'b>>,
}

impl<'a, 'b> InitializeHolderRewardsPoolCpiBuilder<'a, 'b> {
    pub fn new(program: &'b solana_program::account_info::AccountInfo<'a>) -> Self {
        let instruction = Box::new(InitializeHolderRewardsPoolCpiBuilderInstruction {
            __program: program,
            holder_rewards_pool: None,
            extra_account_metas: None,
            mint: None,
            mint_authority: None,
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
    /// Transfer hook extra account metas account.
    #[inline(always)]
    pub fn extra_account_metas(
        &mut self,
        extra_account_metas: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.extra_account_metas = Some(extra_account_metas);
        self
    }
    /// Token mint.
    #[inline(always)]
    pub fn mint(&mut self, mint: &'b solana_program::account_info::AccountInfo<'a>) -> &mut Self {
        self.instruction.mint = Some(mint);
        self
    }
    /// Mint authority.
    #[inline(always)]
    pub fn mint_authority(
        &mut self,
        mint_authority: &'b solana_program::account_info::AccountInfo<'a>,
    ) -> &mut Self {
        self.instruction.mint_authority = Some(mint_authority);
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
        let instruction = InitializeHolderRewardsPoolCpi {
            __program: self.instruction.__program,

            holder_rewards_pool: self
                .instruction
                .holder_rewards_pool
                .expect("holder_rewards_pool is not set"),

            extra_account_metas: self
                .instruction
                .extra_account_metas
                .expect("extra_account_metas is not set"),

            mint: self.instruction.mint.expect("mint is not set"),

            mint_authority: self
                .instruction
                .mint_authority
                .expect("mint_authority is not set"),

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
struct InitializeHolderRewardsPoolCpiBuilderInstruction<'a, 'b> {
    __program: &'b solana_program::account_info::AccountInfo<'a>,
    holder_rewards_pool: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    extra_account_metas: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    mint: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    mint_authority: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    system_program: Option<&'b solana_program::account_info::AccountInfo<'a>>,
    /// Additional instruction accounts `(AccountInfo, is_writable, is_signer)`.
    __remaining_accounts: Vec<(
        &'b solana_program::account_info::AccountInfo<'a>,
        bool,
        bool,
    )>,
}