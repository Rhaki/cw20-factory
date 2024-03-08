# CW20 FACTORY

The `cw20-factory` derived from the standard `CW20` implementation, designed to streamline the management of tokens both in the `CW20` and `native` version through the use of a `Token Factory`. During contract initialization, the `native` version of the `token` is also produced via the `Token Factory`. This implementation retains the original `entry points` of the `cw20-base`, yet it is enhanced with the `ExecuteMsg::TransmuteInto` (feature that allows users to convert `native` tokens into `CW20` formats and vice versa) and others functionality.

In parallel, an auxiliary contract named `Indexer` is introduced, tasked with mapping between the `denom` associated with the `Token Factory` and the `CW20` address. This association process can occurs automatically during the `cw20-factory` initialization if optional indexer field is provided (backwards compatible).
It is possible to register a `cw20-factory` with multiple `Indexer`, allowing the management of specific token subsets based on needs (For example, a protocol can create its own indexer for the tokens it manages). Registration can be requested from the `cw20-factory` in a permissionless manner.

To ensure proper management of the various types of `Token Factories` across different blockchains, the contract requires, at the compilation level, the use of a structure that implements the `TokenFactoryInterface` `trait`. This `trait` serves as a communicative bridge with the specific `TokenFactory` module of each blockchain, maintaining the base code's independence from interchain variations.

## ExecuteMsg implementation

The new ExecuteMsg implementation is full backwards compatible with contracts that interact with the basic version of cw20

```rust
#[cw_serde]
pub enum ExecuteMsg {
    /// Transmute `cw20` into `native` token or vice versa
    TransmuteInto(TransmuteIntoMsg),
    /// Register this contract into an indexer
    RegisterToIndexer {
        indexer_addr: String,
    },
    /// Create native token after a migration from cw20-base
    CreateNative {},
    Burn {
        /// Amount is now optional:
        /// - Burn native: amount field is not used (info.funds will be checked).
        /// - Burn cw20: need to specify amount (backwards compatible)
        amount: Option<Uint128>,
    },
    Mint {
        recipient: String,
        amount: Uint128,
        /// New field.
        /// If not provided, mint will happens as cw20 (backwards compatible)
        as_native: Option<bool>,
    },
    ...
}
```

## QueryMsg implementation

```rust
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Returns the factory denom for this token
    #[returns(String)]
    FactoryDenom {},
    /// Returns the supply share between cw20 and native .
    #[returns(SupplyDetailsResponse)]
    SupplyDetails {},
    ...
}
```

`TokenInfoResponse.total_supply` return the sum of `cw20` and `native` supply

## Migration from existing cw20-base

It is possible to migrate an existing token from `cw20-base` to `cw20-factory`. Once the migration has taken place, to generate the native token is needed to execute `ExecuteMsg::CreateNative` message. Until this message is executed, the token will continue to function as a `cw20-base.`
