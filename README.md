# CW20 FACTORY

The `cw20-factory` derived from the standard `CW20` implementation, designed to streamline the management of tokens both in the `CW20` and native formats through the use of a `Token Factory`. During the initial phase of contract initialization, the `native` version of the `token` is also produced via the `Token Factory`. This implementation retains the original `entry points` of the `cw20-base`, yet it is enhanced with the `ExecuteMsg::TransmuteInto` and others functionality. This feature allows users to convert `native` tokens into `CW20` formats and vice versa.

In parallel, an auxiliary contract named `Indexer` is introduced, tasked with mapping between the `denom` associated with the `Token Factory` and the `CW20` address. This association process occurs automatically during the `cw20-factory` initialization.

To ensure proper management of the various types of `Token Factories` across different blockchains, the contract requires, at the compilation level, the use of a structure that implements the `TokenFactoryInterface` trait. This trait serves as a communicative bridge with the specific `TokenFactory` module of each blockchain, maintaining the base code's independence from interchain variations.

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
        /// Amount is now optional
        /// If want to burn native token, amount field is not used (info.funds will be checked).
        /// If want to burn cw20, specify amount (backwards compatible)
        amount: Option<Uint128>,
    },
    Mint {
        recipient: String,
        amount: Uint128,
        /// New field.
        /// If not provided, it will mint as cw20 (backwards compatible)
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
