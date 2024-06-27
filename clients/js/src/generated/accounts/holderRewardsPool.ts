/**
 * This code was AUTOGENERATED using the kinobi library.
 * Please DO NOT EDIT THIS FILE, instead use visitors
 * to add features, then rerun kinobi to update it.
 *
 * @see https://github.com/kinobi-so/kinobi
 */

import {
  assertAccountExists,
  assertAccountsExist,
  combineCodec,
  decodeAccount,
  fetchEncodedAccount,
  fetchEncodedAccounts,
  getStructDecoder,
  getStructEncoder,
  getU128Decoder,
  getU128Encoder,
  type Account,
  type Address,
  type Codec,
  type Decoder,
  type EncodedAccount,
  type Encoder,
  type FetchAccountConfig,
  type FetchAccountsConfig,
  type MaybeAccount,
  type MaybeEncodedAccount,
} from '@solana/web3.js';

export type HolderRewardsPool = { rewardsPerToken: bigint };

export type HolderRewardsPoolArgs = { rewardsPerToken: number | bigint };

export function getHolderRewardsPoolEncoder(): Encoder<HolderRewardsPoolArgs> {
  return getStructEncoder([['rewardsPerToken', getU128Encoder()]]);
}

export function getHolderRewardsPoolDecoder(): Decoder<HolderRewardsPool> {
  return getStructDecoder([['rewardsPerToken', getU128Decoder()]]);
}

export function getHolderRewardsPoolCodec(): Codec<
  HolderRewardsPoolArgs,
  HolderRewardsPool
> {
  return combineCodec(
    getHolderRewardsPoolEncoder(),
    getHolderRewardsPoolDecoder()
  );
}

export function decodeHolderRewardsPool<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress>
): Account<HolderRewardsPool, TAddress>;
export function decodeHolderRewardsPool<TAddress extends string = string>(
  encodedAccount: MaybeEncodedAccount<TAddress>
): MaybeAccount<HolderRewardsPool, TAddress>;
export function decodeHolderRewardsPool<TAddress extends string = string>(
  encodedAccount: EncodedAccount<TAddress> | MaybeEncodedAccount<TAddress>
):
  | Account<HolderRewardsPool, TAddress>
  | MaybeAccount<HolderRewardsPool, TAddress> {
  return decodeAccount(
    encodedAccount as MaybeEncodedAccount<TAddress>,
    getHolderRewardsPoolDecoder()
  );
}

export async function fetchHolderRewardsPool<TAddress extends string = string>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<Account<HolderRewardsPool, TAddress>> {
  const maybeAccount = await fetchMaybeHolderRewardsPool(rpc, address, config);
  assertAccountExists(maybeAccount);
  return maybeAccount;
}

export async function fetchMaybeHolderRewardsPool<
  TAddress extends string = string,
>(
  rpc: Parameters<typeof fetchEncodedAccount>[0],
  address: Address<TAddress>,
  config?: FetchAccountConfig
): Promise<MaybeAccount<HolderRewardsPool, TAddress>> {
  const maybeAccount = await fetchEncodedAccount(rpc, address, config);
  return decodeHolderRewardsPool(maybeAccount);
}

export async function fetchAllHolderRewardsPool(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<Account<HolderRewardsPool>[]> {
  const maybeAccounts = await fetchAllMaybeHolderRewardsPool(
    rpc,
    addresses,
    config
  );
  assertAccountsExist(maybeAccounts);
  return maybeAccounts;
}

export async function fetchAllMaybeHolderRewardsPool(
  rpc: Parameters<typeof fetchEncodedAccounts>[0],
  addresses: Array<Address>,
  config?: FetchAccountsConfig
): Promise<MaybeAccount<HolderRewardsPool>[]> {
  const maybeAccounts = await fetchEncodedAccounts(rpc, addresses, config);
  return maybeAccounts.map((maybeAccount) =>
    decodeHolderRewardsPool(maybeAccount)
  );
}

export function getHolderRewardsPoolSize(): number {
  return 16;
}
