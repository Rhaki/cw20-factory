# CW20 FACTORY

`cw20-factory` is an implementation of the `cw20-base` that manages a token both as `cw20` and as `native` through the use of the `TokenFactory`.

During the contract's initialization, the `native` version is also generated through the `TokenFactory`. The entry points are the same as those of the `cw20-base`, with the addition of the `ExecuteMsg::TransmuteInto` implementation that allows a user to transform `native tokens` into `cw20` and vice versa.

There is a second `contract`, called `Indexer`, that tracks the link between the token factory's `denom` and the `cw20` address. Registration occurs automatically during the `cw20-factory` initialization.

For managing different types of `TokenFactory` across various chains, the contract requires, during compilation, a structure that implements the `TokenFactoryInterface` trait. This trait acts as an `interface` for communication with the `TokenFactory` module of a specific chain, keeping the base code free from these differences.
