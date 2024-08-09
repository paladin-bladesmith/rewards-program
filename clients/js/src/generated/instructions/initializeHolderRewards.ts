/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  combineCodec,
  getStructDecoder,
  getStructEncoder,
  getU8Decoder,
  getU8Encoder,
  transformEncoder,
  type Address,
  type Codec,
  type Decoder,
  type Encoder,
  type IAccountMeta,
  type IInstruction,
  type IInstructionWithAccounts,
  type IInstructionWithData,
  type ReadonlyAccount,
  type WritableAccount,
} from '@solana/web3.js';
import { PALADIN_REWARDS_PROGRAM_ADDRESS } from '../programs';
import { getAccountMetaFactory, type ResolvedAccount } from '../shared';

export type InitializeHolderRewardsInstruction<
  TProgram extends string = typeof PALADIN_REWARDS_PROGRAM_ADDRESS,
  TAccountHolderRewardsPool extends string | IAccountMeta<string> = string,
  TAccountHolderRewards extends string | IAccountMeta<string> = string,
  TAccountTokenAccount extends string | IAccountMeta<string> = string,
  TAccountMint extends string | IAccountMeta<string> = string,
  TAccountSystemProgram extends
    | string
    | IAccountMeta<string> = '11111111111111111111111111111111',
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountHolderRewardsPool extends string
        ? ReadonlyAccount<TAccountHolderRewardsPool>
        : TAccountHolderRewardsPool,
      TAccountHolderRewards extends string
        ? WritableAccount<TAccountHolderRewards>
        : TAccountHolderRewards,
      TAccountTokenAccount extends string
        ? ReadonlyAccount<TAccountTokenAccount>
        : TAccountTokenAccount,
      TAccountMint extends string
        ? ReadonlyAccount<TAccountMint>
        : TAccountMint,
      TAccountSystemProgram extends string
        ? ReadonlyAccount<TAccountSystemProgram>
        : TAccountSystemProgram,
      ...TRemainingAccounts,
    ]
  >;

export type InitializeHolderRewardsInstructionData = { discriminator: number };

export type InitializeHolderRewardsInstructionDataArgs = {};

export function getInitializeHolderRewardsInstructionDataEncoder(): Encoder<InitializeHolderRewardsInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({ ...value, discriminator: 2 })
  );
}

export function getInitializeHolderRewardsInstructionDataDecoder(): Decoder<InitializeHolderRewardsInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getInitializeHolderRewardsInstructionDataCodec(): Codec<
  InitializeHolderRewardsInstructionDataArgs,
  InitializeHolderRewardsInstructionData
> {
  return combineCodec(
    getInitializeHolderRewardsInstructionDataEncoder(),
    getInitializeHolderRewardsInstructionDataDecoder()
  );
}

export type InitializeHolderRewardsInput<
  TAccountHolderRewardsPool extends string = string,
  TAccountHolderRewards extends string = string,
  TAccountTokenAccount extends string = string,
  TAccountMint extends string = string,
  TAccountSystemProgram extends string = string,
> = {
  /** Holder rewards pool account. */
  holderRewardsPool: Address<TAccountHolderRewardsPool>;
  /** Holder rewards account. */
  holderRewards: Address<TAccountHolderRewards>;
  /** Token account. */
  tokenAccount: Address<TAccountTokenAccount>;
  /** Token mint. */
  mint: Address<TAccountMint>;
  /** System program. */
  systemProgram?: Address<TAccountSystemProgram>;
};

export function getInitializeHolderRewardsInstruction<
  TAccountHolderRewardsPool extends string,
  TAccountHolderRewards extends string,
  TAccountTokenAccount extends string,
  TAccountMint extends string,
  TAccountSystemProgram extends string,
>(
  input: InitializeHolderRewardsInput<
    TAccountHolderRewardsPool,
    TAccountHolderRewards,
    TAccountTokenAccount,
    TAccountMint,
    TAccountSystemProgram
  >
): InitializeHolderRewardsInstruction<
  typeof PALADIN_REWARDS_PROGRAM_ADDRESS,
  TAccountHolderRewardsPool,
  TAccountHolderRewards,
  TAccountTokenAccount,
  TAccountMint,
  TAccountSystemProgram
> {
  // Program address.
  const programAddress = PALADIN_REWARDS_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    holderRewardsPool: {
      value: input.holderRewardsPool ?? null,
      isWritable: false,
    },
    holderRewards: { value: input.holderRewards ?? null, isWritable: true },
    tokenAccount: { value: input.tokenAccount ?? null, isWritable: false },
    mint: { value: input.mint ?? null, isWritable: false },
    systemProgram: { value: input.systemProgram ?? null, isWritable: false },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  // Resolve default values.
  if (!accounts.systemProgram.value) {
    accounts.systemProgram.value =
      '11111111111111111111111111111111' as Address<'11111111111111111111111111111111'>;
  }

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.holderRewardsPool),
      getAccountMeta(accounts.holderRewards),
      getAccountMeta(accounts.tokenAccount),
      getAccountMeta(accounts.mint),
      getAccountMeta(accounts.systemProgram),
    ],
    programAddress,
    data: getInitializeHolderRewardsInstructionDataEncoder().encode({}),
  } as InitializeHolderRewardsInstruction<
    typeof PALADIN_REWARDS_PROGRAM_ADDRESS,
    TAccountHolderRewardsPool,
    TAccountHolderRewards,
    TAccountTokenAccount,
    TAccountMint,
    TAccountSystemProgram
  >;

  return instruction;
}

export type ParsedInitializeHolderRewardsInstruction<
  TProgram extends string = typeof PALADIN_REWARDS_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Holder rewards pool account. */
    holderRewardsPool: TAccountMetas[0];
    /** Holder rewards account. */
    holderRewards: TAccountMetas[1];
    /** Token account. */
    tokenAccount: TAccountMetas[2];
    /** Token mint. */
    mint: TAccountMetas[3];
    /** System program. */
    systemProgram: TAccountMetas[4];
  };
  data: InitializeHolderRewardsInstructionData;
};

export function parseInitializeHolderRewardsInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedInitializeHolderRewardsInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 5) {
    // TODO: Coded error.
    throw new Error('Not enough accounts');
  }
  let accountIndex = 0;
  const getNextAccount = () => {
    const accountMeta = instruction.accounts![accountIndex]!;
    accountIndex += 1;
    return accountMeta;
  };
  return {
    programAddress: instruction.programAddress,
    accounts: {
      holderRewardsPool: getNextAccount(),
      holderRewards: getNextAccount(),
      tokenAccount: getNextAccount(),
      mint: getNextAccount(),
      systemProgram: getNextAccount(),
    },
    data: getInitializeHolderRewardsInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
