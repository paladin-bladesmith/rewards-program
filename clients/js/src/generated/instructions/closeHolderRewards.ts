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

export type CloseHolderRewardsInstruction<
  TProgram extends string = typeof PALADIN_REWARDS_PROGRAM_ADDRESS,
  TAccountHolderRewardsPool extends string | IAccountMeta<string> = string,
  TAccountHolderRewardsPoolTokenAccountInfo extends
    | string
    | IAccountMeta<string> = string,
  TAccountHolderRewards extends string | IAccountMeta<string> = string,
  TAccountTokenAccount extends string | IAccountMeta<string> = string,
  TAccountMint extends string | IAccountMeta<string> = string,
  TAccountOwner extends string | IAccountMeta<string> = string,
  TRemainingAccounts extends readonly IAccountMeta<string>[] = [],
> = IInstruction<TProgram> &
  IInstructionWithData<Uint8Array> &
  IInstructionWithAccounts<
    [
      TAccountHolderRewardsPool extends string
        ? WritableAccount<TAccountHolderRewardsPool>
        : TAccountHolderRewardsPool,
      TAccountHolderRewardsPoolTokenAccountInfo extends string
        ? WritableAccount<TAccountHolderRewardsPoolTokenAccountInfo>
        : TAccountHolderRewardsPoolTokenAccountInfo,
      TAccountHolderRewards extends string
        ? WritableAccount<TAccountHolderRewards>
        : TAccountHolderRewards,
      TAccountTokenAccount extends string
        ? ReadonlyAccount<TAccountTokenAccount>
        : TAccountTokenAccount,
      TAccountMint extends string
        ? ReadonlyAccount<TAccountMint>
        : TAccountMint,
      TAccountOwner extends string
        ? WritableAccount<TAccountOwner>
        : TAccountOwner,
      ...TRemainingAccounts,
    ]
  >;

export type CloseHolderRewardsInstructionData = { discriminator: number };

export type CloseHolderRewardsInstructionDataArgs = {};

export function getCloseHolderRewardsInstructionDataEncoder(): Encoder<CloseHolderRewardsInstructionDataArgs> {
  return transformEncoder(
    getStructEncoder([['discriminator', getU8Encoder()]]),
    (value) => ({ ...value, discriminator: 3 })
  );
}

export function getCloseHolderRewardsInstructionDataDecoder(): Decoder<CloseHolderRewardsInstructionData> {
  return getStructDecoder([['discriminator', getU8Decoder()]]);
}

export function getCloseHolderRewardsInstructionDataCodec(): Codec<
  CloseHolderRewardsInstructionDataArgs,
  CloseHolderRewardsInstructionData
> {
  return combineCodec(
    getCloseHolderRewardsInstructionDataEncoder(),
    getCloseHolderRewardsInstructionDataDecoder()
  );
}

export type CloseHolderRewardsInput<
  TAccountHolderRewardsPool extends string = string,
  TAccountHolderRewardsPoolTokenAccountInfo extends string = string,
  TAccountHolderRewards extends string = string,
  TAccountTokenAccount extends string = string,
  TAccountMint extends string = string,
  TAccountOwner extends string = string,
> = {
  /** Holder rewards pool account. */
  holderRewardsPool: Address<TAccountHolderRewardsPool>;
  /** Holder rewards pool token account. */
  holderRewardsPoolTokenAccountInfo: Address<TAccountHolderRewardsPoolTokenAccountInfo>;
  /** Holder rewards account. */
  holderRewards: Address<TAccountHolderRewards>;
  /** Token account. */
  tokenAccount: Address<TAccountTokenAccount>;
  /** Token mint. */
  mint: Address<TAccountMint>;
  /** Owner of the account. */
  owner: Address<TAccountOwner>;
};

export function getCloseHolderRewardsInstruction<
  TAccountHolderRewardsPool extends string,
  TAccountHolderRewardsPoolTokenAccountInfo extends string,
  TAccountHolderRewards extends string,
  TAccountTokenAccount extends string,
  TAccountMint extends string,
  TAccountOwner extends string,
>(
  input: CloseHolderRewardsInput<
    TAccountHolderRewardsPool,
    TAccountHolderRewardsPoolTokenAccountInfo,
    TAccountHolderRewards,
    TAccountTokenAccount,
    TAccountMint,
    TAccountOwner
  >
): CloseHolderRewardsInstruction<
  typeof PALADIN_REWARDS_PROGRAM_ADDRESS,
  TAccountHolderRewardsPool,
  TAccountHolderRewardsPoolTokenAccountInfo,
  TAccountHolderRewards,
  TAccountTokenAccount,
  TAccountMint,
  TAccountOwner
> {
  // Program address.
  const programAddress = PALADIN_REWARDS_PROGRAM_ADDRESS;

  // Original accounts.
  const originalAccounts = {
    holderRewardsPool: {
      value: input.holderRewardsPool ?? null,
      isWritable: true,
    },
    holderRewardsPoolTokenAccountInfo: {
      value: input.holderRewardsPoolTokenAccountInfo ?? null,
      isWritable: true,
    },
    holderRewards: { value: input.holderRewards ?? null, isWritable: true },
    tokenAccount: { value: input.tokenAccount ?? null, isWritable: false },
    mint: { value: input.mint ?? null, isWritable: false },
    owner: { value: input.owner ?? null, isWritable: true },
  };
  const accounts = originalAccounts as Record<
    keyof typeof originalAccounts,
    ResolvedAccount
  >;

  const getAccountMeta = getAccountMetaFactory(programAddress, 'programId');
  const instruction = {
    accounts: [
      getAccountMeta(accounts.holderRewardsPool),
      getAccountMeta(accounts.holderRewardsPoolTokenAccountInfo),
      getAccountMeta(accounts.holderRewards),
      getAccountMeta(accounts.tokenAccount),
      getAccountMeta(accounts.mint),
      getAccountMeta(accounts.owner),
    ],
    programAddress,
    data: getCloseHolderRewardsInstructionDataEncoder().encode({}),
  } as CloseHolderRewardsInstruction<
    typeof PALADIN_REWARDS_PROGRAM_ADDRESS,
    TAccountHolderRewardsPool,
    TAccountHolderRewardsPoolTokenAccountInfo,
    TAccountHolderRewards,
    TAccountTokenAccount,
    TAccountMint,
    TAccountOwner
  >;

  return instruction;
}

export type ParsedCloseHolderRewardsInstruction<
  TProgram extends string = typeof PALADIN_REWARDS_PROGRAM_ADDRESS,
  TAccountMetas extends readonly IAccountMeta[] = readonly IAccountMeta[],
> = {
  programAddress: Address<TProgram>;
  accounts: {
    /** Holder rewards pool account. */
    holderRewardsPool: TAccountMetas[0];
    /** Holder rewards pool token account. */
    holderRewardsPoolTokenAccountInfo: TAccountMetas[1];
    /** Holder rewards account. */
    holderRewards: TAccountMetas[2];
    /** Token account. */
    tokenAccount: TAccountMetas[3];
    /** Token mint. */
    mint: TAccountMetas[4];
    /** Owner of the account. */
    owner: TAccountMetas[5];
  };
  data: CloseHolderRewardsInstructionData;
};

export function parseCloseHolderRewardsInstruction<
  TProgram extends string,
  TAccountMetas extends readonly IAccountMeta[],
>(
  instruction: IInstruction<TProgram> &
    IInstructionWithAccounts<TAccountMetas> &
    IInstructionWithData<Uint8Array>
): ParsedCloseHolderRewardsInstruction<TProgram, TAccountMetas> {
  if (instruction.accounts.length < 6) {
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
      holderRewardsPoolTokenAccountInfo: getNextAccount(),
      holderRewards: getNextAccount(),
      tokenAccount: getNextAccount(),
      mint: getNextAccount(),
      owner: getNextAccount(),
    },
    data: getCloseHolderRewardsInstructionDataDecoder().decode(
      instruction.data
    ),
  };
}
