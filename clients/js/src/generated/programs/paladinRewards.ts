/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  containsBytes,
  getU8Encoder,
  type Address,
  type ReadonlyUint8Array,
} from '@solana/web3.js';
import {
  type ParsedDistributeRewardsInstruction,
  type ParsedHarvestRewardsInstruction,
  type ParsedInitializeHolderRewardsInstruction,
  type ParsedInitializeHolderRewardsPoolInstruction,
} from '../instructions';

export const PALADIN_REWARDS_PROGRAM_ADDRESS =
  '2XNqZeXtemZ1FjrkVsssPPE9AVAVjTSoaqb53EnNQ1fe' as Address<'2XNqZeXtemZ1FjrkVsssPPE9AVAVjTSoaqb53EnNQ1fe'>;

export enum PaladinRewardsAccount {
  HolderRewards,
  HolderRewardsPool,
}

export enum PaladinRewardsInstruction {
  InitializeHolderRewardsPool,
  DistributeRewards,
  InitializeHolderRewards,
  HarvestRewards,
}

export function identifyPaladinRewardsInstruction(
  instruction: { data: ReadonlyUint8Array } | ReadonlyUint8Array
): PaladinRewardsInstruction {
  const data = 'data' in instruction ? instruction.data : instruction;
  if (containsBytes(data, getU8Encoder().encode(0), 0)) {
    return PaladinRewardsInstruction.InitializeHolderRewardsPool;
  }
  if (containsBytes(data, getU8Encoder().encode(1), 0)) {
    return PaladinRewardsInstruction.DistributeRewards;
  }
  if (containsBytes(data, getU8Encoder().encode(2), 0)) {
    return PaladinRewardsInstruction.InitializeHolderRewards;
  }
  if (containsBytes(data, getU8Encoder().encode(3), 0)) {
    return PaladinRewardsInstruction.HarvestRewards;
  }
  throw new Error(
    'The provided instruction could not be identified as a paladinRewards instruction.'
  );
}

export type ParsedPaladinRewardsInstruction<
  TProgram extends string = '2XNqZeXtemZ1FjrkVsssPPE9AVAVjTSoaqb53EnNQ1fe',
> =
  | ({
      instructionType: PaladinRewardsInstruction.InitializeHolderRewardsPool;
    } & ParsedInitializeHolderRewardsPoolInstruction<TProgram>)
  | ({
      instructionType: PaladinRewardsInstruction.DistributeRewards;
    } & ParsedDistributeRewardsInstruction<TProgram>)
  | ({
      instructionType: PaladinRewardsInstruction.InitializeHolderRewards;
    } & ParsedInitializeHolderRewardsInstruction<TProgram>)
  | ({
      instructionType: PaladinRewardsInstruction.HarvestRewards;
    } & ParsedHarvestRewardsInstruction<TProgram>);
