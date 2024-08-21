# Paladin Rewards Program

<a href="https://github.com/paladin-bladesmith/rewards-program/actions/workflows/main.yml"><img src="https://img.shields.io/github/actions/workflow/status/paladin-bladesmith/rewards-program/main.yml?logo=GitHub" /></a>
<a href="https://explorer.solana.com/address/Dbf7u6x15DhjMrBMunY3XoRWdByrCCt2dbyoPrCXN6SQ"><img src="https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Fraw.githubusercontent.com%2Fpaladin-bladesmith%2Frewards-program%2Fmain%2Fprogram%2Fidl.json&query=%24.version&label=program&logo=data:image/svg%2bxml;base64,PHN2ZyB3aWR0aD0iMzEzIiBoZWlnaHQ9IjI4MSIgdmlld0JveD0iMCAwIDMxMyAyODEiIGZpbGw9Im5vbmUiIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyI+CjxnIGNsaXAtcGF0aD0idXJsKCNjbGlwMF80NzZfMjQzMCkiPgo8cGF0aCBkPSJNMzExLjMxOCAyMjEuMDU3TDI1OS42NiAyNzYuNTU4QzI1OC41MzcgMjc3Ljc2NCAyNTcuMTc4IDI3OC43MjUgMjU1LjY2OSAyNzkuMzgyQzI1NC4xNTkgMjgwLjAzOSAyNTIuNTMgMjgwLjM3OCAyNTAuODg0IDI4MC4zNzdINS45OTcxOUM0LjgyODcgMjgwLjM3NyAzLjY4NTY4IDI4MC4wMzUgMi43MDg1NSAyNzkuMzkzQzEuNzMxNDMgMjc4Ljc1MSAwLjk2Mjc3MSAyNzcuODM3IDAuNDk3MDIgMjc2Ljc2NEMwLjAzMTI2OTEgMjc1LjY5IC0wLjExMTI4NiAyNzQuNTA0IDAuMDg2ODcxMiAyNzMuMzVDMC4yODUwMjggMjcyLjE5NiAwLjgxNTI2NSAyNzEuMTI2IDEuNjEyNDMgMjcwLjI3TDUzLjMwOTkgMjE0Ljc2OUM1NC40Mjk5IDIxMy41NjYgNTUuNzg0MyAyMTIuNjA3IDU3LjI4OTMgMjExLjk1QzU4Ljc5NDMgMjExLjI5MyA2MC40MTc4IDIxMC45NTMgNjIuMDU5NSAyMTAuOTVIMzA2LjkzM0MzMDguMTAxIDIxMC45NSAzMDkuMjQ0IDIxMS4yOTIgMzEwLjIyMSAyMTEuOTM0QzMxMS4xOTkgMjEyLjU3NiAzMTEuOTY3IDIxMy40OSAzMTIuNDMzIDIxNC41NjRDMzEyLjg5OSAyMTUuNjM3IDMxMy4wNDEgMjE2LjgyNCAzMTIuODQzIDIxNy45NzdDMzEyLjY0NSAyMTkuMTMxIDMxMi4xMTUgMjIwLjIwMSAzMTEuMzE4IDIyMS4wNTdaTTI1OS42NiAxMDkuMjk0QzI1OC41MzcgMTA4LjA4OCAyNTcuMTc4IDEwNy4xMjcgMjU1LjY2OSAxMDYuNDdDMjU0LjE1OSAxMDUuODEzIDI1Mi41MyAxMDUuNDc0IDI1MC44ODQgMTA1LjQ3NUg1Ljk5NzE5QzQuODI4NyAxMDUuNDc1IDMuNjg1NjggMTA1LjgxNyAyLjcwODU1IDEwNi40NTlDMS43MzE0MyAxMDcuMTAxIDAuOTYyNzcxIDEwOC4wMTUgMC40OTcwMiAxMDkuMDg4QzAuMDMxMjY5MSAxMTAuMTYyIC0wLjExMTI4NiAxMTEuMzQ4IDAuMDg2ODcxMiAxMTIuNTAyQzAuMjg1MDI4IDExMy42NTYgMC44MTUyNjUgMTE0LjcyNiAxLjYxMjQzIDExNS41ODJMNTMuMzA5OSAxNzEuMDgzQzU0LjQyOTkgMTcyLjI4NiA1NS43ODQzIDE3My4yNDUgNTcuMjg5MyAxNzMuOTAyQzU4Ljc5NDMgMTc0LjU1OSA2MC40MTc4IDE3NC44OTkgNjIuMDU5NSAxNzQuOTAySDMwNi45MzNDMzA4LjEwMSAxNzQuOTAyIDMwOS4yNDQgMTc0LjU2IDMxMC4yMjEgMTczLjkxOEMzMTEuMTk5IDE3My4yNzYgMzExLjk2NyAxNzIuMzYyIDMxMi40MzMgMTcxLjI4OEMzMTIuODk5IDE3MC4yMTUgMzEzLjA0MSAxNjkuMDI4IDMxMi44NDMgMTY3Ljg3NUMzMTIuNjQ1IDE2Ni43MjEgMzEyLjExNSAxNjUuNjUxIDMxMS4zMTggMTY0Ljc5NUwyNTkuNjYgMTA5LjI5NFpNNS45OTcxOSA2OS40MjY3SDI1MC44ODRDMjUyLjUzIDY5LjQyNzUgMjU0LjE1OSA2OS4wODkgMjU1LjY2OSA2OC40MzJDMjU3LjE3OCA2Ny43NzUxIDI1OC41MzcgNjYuODEzOSAyNTkuNjYgNjUuNjA4MkwzMTEuMzE4IDEwLjEwNjlDMzEyLjExNSA5LjI1MTA3IDMxMi42NDUgOC4xODA1NiAzMTIuODQzIDcuMDI2OTVDMzEzLjA0MSA1Ljg3MzM0IDMxMi44OTkgNC42ODY4NiAzMTIuNDMzIDMuNjEzM0MzMTEuOTY3IDIuNTM5NzQgMzExLjE5OSAxLjYyNTg2IDMxMC4yMjEgMC45ODM5NDFDMzA5LjI0NCAwLjM0MjAyNiAzMDguMTAxIDMuOTUzMTRlLTA1IDMwNi45MzMgMEw2Mi4wNTk1IDBDNjAuNDE3OCAwLjAwMjc5ODY2IDU4Ljc5NDMgMC4zNDMxNCA1Ny4yODkzIDAuOTk5OTUzQzU1Ljc4NDMgMS42NTY3NyA1NC40Mjk5IDIuNjE2MDcgNTMuMzA5OSAzLjgxODQ3TDEuNjI1NzYgNTkuMzE5N0MwLjgyOTM2MSA2MC4xNzQ4IDAuMjk5MzU5IDYxLjI0NCAwLjEwMDc1MiA2Mi4zOTY0Qy0wLjA5Nzg1MzkgNjMuNTQ4OCAwLjA0MzU2OTggNjQuNzM0MiAwLjUwNzY3OSA2NS44MDczQzAuOTcxNzg5IDY2Ljg4MDMgMS43Mzg0MSA2Ny43OTQzIDIuNzEzNTIgNjguNDM3MkMzLjY4ODYzIDY5LjA4MDIgNC44Mjk4NCA2OS40MjQgNS45OTcxOSA2OS40MjY3WiIgZmlsbD0idXJsKCNwYWludDBfbGluZWFyXzQ3Nl8yNDMwKSIvPgo8L2c+CjxkZWZzPgo8bGluZWFyR3JhZGllbnQgaWQ9InBhaW50MF9saW5lYXJfNDc2XzI0MzAiIHgxPSIyNi40MTUiIHkxPSIyODcuMDU5IiB4Mj0iMjgzLjczNSIgeTI9Ii0yLjQ5NTc0IiBncmFkaWVudFVuaXRzPSJ1c2VyU3BhY2VPblVzZSI+CjxzdG9wIG9mZnNldD0iMC4wOCIgc3RvcC1jb2xvcj0iIzk5NDVGRiIvPgo8c3RvcCBvZmZzZXQ9IjAuMyIgc3RvcC1jb2xvcj0iIzg3NTJGMyIvPgo8c3RvcCBvZmZzZXQ9IjAuNSIgc3RvcC1jb2xvcj0iIzU0OTdENSIvPgo8c3RvcCBvZmZzZXQ9IjAuNiIgc3RvcC1jb2xvcj0iIzQzQjRDQSIvPgo8c3RvcCBvZmZzZXQ9IjAuNzIiIHN0b3AtY29sb3I9IiMyOEUwQjkiLz4KPHN0b3Agb2Zmc2V0PSIwLjk3IiBzdG9wLWNvbG9yPSIjMTlGQjlCIi8+CjwvbGluZWFyR3JhZGllbnQ+CjxjbGlwUGF0aCBpZD0iY2xpcDBfNDc2XzI0MzAiPgo8cmVjdCB3aWR0aD0iMzEyLjkzIiBoZWlnaHQ9IjI4MC4zNzciIGZpbGw9IndoaXRlIi8+CjwvY2xpcFBhdGg+CjwvZGVmcz4KPC9zdmc+Cg==&color=9945FF" /></a>
<a href="https://www.npmjs.com/package/@paladin-bladesmith/rewards"><img src="https://img.shields.io/npm/v/%40paladin-bladesmith%2Frewards?logo=npm&color=377CC0" /></a>
<a href="https://crates.io/crates/paladin-rewards-program-client"><img src="https://img.shields.io/crates/v/paladin-rewards-program-client?logo=rust" /></a>

![](./docs/paladin_rewards_program.jpg)

The Paladin Rewards program manages the distribution of rewards to token
holders based on their share of the total token supply. Holders earn shares of
rewards proportional to their share of token supply.

## Architecture

The program makes use of two types of accounts to manage the tracking of
eligible rewards - Holder Rewards Pool and Holder Rewards.

The Holder Rewards Pool is a global pool of reward funds that maintains a
governing rate of "rewards per token" for all holders to claim against.
Rewards are deposited directly into the pool using the `DistributeRewards`
instruction and can be claimed by holders.

```rust
struct HolderRewardsPool {
    /// The current rewards per token exchange rate.
    ///
    /// Stored as a `u128`, which includes a scaling factor of `1e18` to
    /// represent the exchange rate with 18 decimal places of precision.
    pub accumulated_rewards_per_token: u128,
}
```

A Holder Rewards account is assigned to each token holder and, together with
the Holder Rewards Pool, tracks the rewards a particular holder can claim.

```rust
struct HolderRewards {
    /// The rewards per token exchange rate when this holder last harvested.
    ///
    /// Stored as a `u128`, which includes a scaling factor of `1e18` to
    /// represent the exchange rate with 18 decimal places of precision.
    pub last_accumulated_rewards_per_token: u128,
    /// The amount of unharvested rewards currently stored in the holder
    /// rewards account that can be harvested by the holder.
    pub unharvested_rewards: u64,
    _padding: u64,
}
```

## Reward Calculation

The pool's `accumulated_rewards_per_token` is a constantly-increasing rate that
determines how many rewards each holder can claim on each token they hold. As
holders claim rewards, their Holder Rewards account stores the pool's rate at
their last claim. This is done to maintain proper accounting across changing
token balances, including transferring and burning.

The `accumulated_rewards_per_token` is calculated against the mint's total
supply, simply by dividing newly deposited rewards by the token's total
supply and adding that "marginal rate" to the current rate.

For example, consider a pool whose current rate is 5 rewards per token and
Alice has 100 tokens. She hasn't claimed before, so she's entitled to claim
500 rewards. She claims those rewards, and her Holder Rewards account stores
the rate of 5. Now, more rewards are deposited into the system, bringing the
rate up to 8 rewards per token. Since Alice last harvested 5 rewards per token,
and her token balance has not changed, she is now eligible for 3 rewards per
token - the difference between the current accumulated rate, and the rate she
last harvested at.

## Transfer Hook

The Paladin Rewards program also implements the SPL Transfer Hook interface,
and this hook functionality is paramount to preserving proper rewards
accounting.

During transfer, the hook implementation will essentially calculate each
holder's eligible rewards for their token balance _before the transfer_, move
those rewards into each Holder Rewards account's `unharvested_rewards`, then
update each account's `last_accumulated_rewards_per_token`.

This allows the program to basically "spy on" changes in token balances to
ensure transfers of tokens don't create opportunities to game the system.
