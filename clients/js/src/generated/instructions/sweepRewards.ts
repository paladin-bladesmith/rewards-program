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

export type SweepRewardsInstruction<
  TProgram extends string = typeof PALADIN_REWARDS_PROGRAM_ADDRESS,
  TAccountSweep extends string | IAccountMeta<string> = string,
  TAccountHolderRewardsPool extends string | IAccountMeta<string> = string,
  TAccountMint extends string | IAccountMeta<string> = string,
  TAccountSystemProgram extends
    | string
    | IAccountMeta<string> = '11111111111111111111111111111111',
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountSweep extends string
        ? WritableAccount<TAccountSweep>
        : TAccountSweep,
      TAccountHolderRewardsPool extends string
        ? WritableAccount<TAccountHolderRewardsPool>
        : TAccountHolderRewardsPool,
      TAccountMint extends string
        ? ReadonlyAccount<TAccountMint>
        : TAccountMint,
      TAccountSystemProgram extends string
        ? ReadonlyAccount<TAccountSystemProgram>
        : TAccountSystemProgram,
      ...TRemainingAccounts,
    ]
  >;

export type SweepRewardsInstructionData = { discriminator: number };

export type SweepRewardsInstructionDataArgs = {};

export function getSweepRewardsInstructionDataEncoder(): Encoder<SweepRewardsInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({ ...value, discriminator: 4 })
  );
}

export function getSweepRewardsInstructionDataDecoder(): Decoder<SweepRewardsInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getSweepRewardsInstructionDataCodec(): Codec<
  SweepRewardsInstructionDataArgs,
  SweepRewardsInstructionData
> {
  return combineCodec(
    getSweepRewardsInstructionDataEncoder(),
    getSweepRewardsInstructionDataDecoder()
  );
}

export type SweepRewardsInput<
  TAccountSweep extends string = string,
  TAccountHolderRewardsPool extends string = string,
  TAccountMint extends string = string,
  TAccountSystemProgram extends string = string,
> = {
  /** Sweep account. */
  sweep: Address<TAccountSweep>;
  /** Holder rewards pool account. */
  holderRewardsPool: Address<TAccountHolderRewardsPool>;
  /** Token mint. */
  mint: Address<TAccountMint>;
  /** System program. */
  systemProgram?: Address<TAccountSystemProgram>;
};

export function getSweepRewardsInstruction<
  TAccountSweep extends string,
  TAccountHolderRewardsPool extends string,
  TAccountMint extends string,
  TAccountSystemProgram extends string,
>(
  input: SweepRewardsInput<
    TAccountSweep,
    TAccountHolderRewardsPool,
    TAccountMint,
    TAccountSystemProgram
  >
): SweepRewardsInstruction<
  typeof PALADIN_REWARDS_PROGRAM_ADDRESS,
  TAccountSweep,
  TAccountHolderRewardsPool,
  TAccountMint,
  TAccountSystemProgram
> {
  // Program address.
  const programAddress = PALADIN_REWARDS_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    sweep: { value: input.sweep ?? null, isWritable: true },
    holderRewardsPool: {
      value: input.holderRewardsPool ?? null,
      isWritable: true,
    },
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
      getAccountMeta(accounts.sweep),
      getAccountMeta(accounts.holderRewardsPool),
      getAccountMeta(accounts.mint),
      getAccountMeta(accounts.systemProgram),
    ],
    programAddress,
    data: getSweepRewardsInstructionDataEncoder().encode({}),
  } as SweepRewardsInstruction<
    typeof PALADIN_REWARDS_PROGRAM_ADDRESS,
    TAccountSweep,
    TAccountHolderRewardsPool,
    TAccountMint,
    TAccountSystemProgram
  >;

  return instruction;
}

export type ParsedSweepRewardsInstruction<
  TProgram extends string = typeof PALADIN_REWARDS_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Sweep account. */
    sweep: TAccountMetas[0];
    /** Holder rewards pool account. */
    holderRewardsPool: TAccountMetas[1];
    /** Token mint. */
    mint: TAccountMetas[2];
    /** System program. */
    systemProgram: TAccountMetas[3];
  };
  data: SweepRewardsInstructionData;
};

export function parseSweepRewardsInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedSweepRewardsInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 4) {
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
      sweep: getNextAccount(),
      holderRewardsPool: getNextAccount(),
      mint: getNextAccount(),
      systemProgram: getNextAccount(),
    },
    data: getSweepRewardsInstructionDataDecoder().decode(instruction.data),
  };
}