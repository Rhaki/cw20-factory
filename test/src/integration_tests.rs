use cosmwasm_std::Uint128;
use cw20::MinterResponse;
use cw20_base::msg::InstantiateMsg as Cw20BaseInstantiateMsg;
use cw20_factory_pkg::cw20_factory::msgs::InstantiateMsg as FactoryInstantiateMsg;
use rhaki_cw_plus::{
    asset::{AssetInfoPrecisioned, AssetPrecisioned},
    cw_asset::AssetInfo,
    math::IntoDecimal,
    multi_test::{
        custom_modules::token_factory::CTokenFactoryFee,
        helper::{AppExt, Bench32AppExt, UnwrapError},
    },
    traits::Wrapper,
};

use crate::helper::{
    burn, create_cw20_base, create_cw20_factory, create_native, migrate_to_factory, mint,
    qy_factory_denom, qy_supply, startup_osmosis, transmute,
};

#[test]
#[rustfmt::skip]
fn t1() {
    let (mut app, db, def) = startup_osmosis();

    let msg_init = FactoryInstantiateMsg {
        name: "Token Foo".to_string(),
        symbol: "FOO".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: MinterResponse{ minter: def.owner.to_string(), cap: None }.wrap_some(),
        marketing: None,
        indexer: def.indexer_addr.to_string().wrap_some(),
    };

    let foo_addr = create_cw20_factory(&mut app, &def, msg_init, vec![]).unwrap();

    let user_1 = app.generate_addr("user_1");

    let foo_cw20 = AssetInfoPrecisioned::cw20(&foo_addr, 6);

    let foo_native = AssetInfoPrecisioned::native(qy_factory_denom(&app, &foo_addr), 6);

    app.mint(&user_1, foo_cw20.to_asset(100_u128.into_decimal()));

    let balance_cw20 = app.qy_balance(&user_1, &foo_cw20).unwrap();

    assert_eq!(balance_cw20, foo_cw20.to_asset(100_u128.into_decimal()));

    transmute(&mut app, &user_1, &foo_addr, foo_cw20.to_asset(50_u128.into_decimal())).unwrap();

    let balance_cw20 = app.qy_balance(&user_1, &foo_cw20).unwrap();
    let balance_native = app.qy_balance(&user_1, &foo_native).unwrap();
    assert_eq!(balance_cw20, foo_cw20.to_asset(50_u128.into_decimal()));
    assert_eq!(balance_native, foo_native.to_asset(50_u128.into_decimal()));

    transmute(&mut app, &user_1, &foo_addr, foo_native.to_asset(25_u128.into_decimal())).unwrap();

    let balance_cw20 = app.qy_balance(&user_1, &foo_cw20).unwrap();
    let balance_native = app.qy_balance(&user_1, &foo_native).unwrap();
    assert_eq!(balance_cw20, foo_cw20.to_asset(75_u128.into_decimal()));
    assert_eq!(balance_native, foo_native.to_asset(25_u128.into_decimal()));

    transmute(&mut app, &user_1, &foo_addr, foo_cw20.to_asset(76_u128.into_decimal())).unwrap_err_contains("Insufficient cw20 balance");

    // Set fee for token creation

    let tf_fee_collector = app.generate_addr("tf_fee_collector");

    let fee_token_creation = AssetPrecisioned::new_super(AssetInfo::native("uosmo"), 6, 100_u128.into_decimal());

    db.borrow_mut().token_factory.fee_creation = CTokenFactoryFee { fee: vec![fee_token_creation.clone().try_into().unwrap()], fee_collector: tf_fee_collector }.wrap_some();

    let msg_init = FactoryInstantiateMsg {
        name: "Token Bar".to_string(),
        symbol: "BAR".to_string(),
        decimals: 18,
        initial_balances: vec![],
        mint: MinterResponse{ minter: def.owner.to_string(), cap: None }.wrap_some(),
        marketing: None,
        indexer: def.indexer_addr.to_string().wrap_some(),
    };

   create_cw20_factory(&mut app, &def, msg_init.clone(), vec![]).unwrap_err_contains("Error on gather fee for denom creation");

   app.mint(&def.owner, fee_token_creation.clone());
   db.borrow_mut().token_factory.supplies.remove("factory/osmo1nc5tatafv6eyq7llkr2gv50ff9e22mnf70qgjlv737ktmt4eswrqvlx82r/bar");

   create_cw20_factory(&mut app, &def, msg_init, vec![fee_token_creation.try_into().unwrap()]).unwrap();

}

#[test]
#[rustfmt::skip]
fn t2_migration() {
    let (mut app, db, def) = startup_osmosis();

    let msg_init = Cw20BaseInstantiateMsg {
        name: "Token Foo".to_string(),
        symbol: "FOO".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: MinterResponse {
            minter: def.owner.to_string(),
            cap: None,
        }
        .wrap_some(),
        marketing: None,
    };

    let foo_addr = create_cw20_base(&mut app, &def, msg_init, vec![]).unwrap();

    let user_1 = app.generate_addr("user_1");

    let foo_cw20 = AssetInfoPrecisioned::cw20(&foo_addr, 6);

    let foo_native = AssetInfoPrecisioned::native(format!("factory/{}/{}", foo_addr, "foo"), 6);

    app.mint(&user_1, foo_cw20.to_asset(100_u128.into_decimal()));

    migrate_to_factory(&mut app, &def, &foo_addr).unwrap();

    app.mint(&user_1, foo_cw20.to_asset(100_u128.into_decimal()));

    let supply = qy_supply(&app, &foo_addr);
    assert_eq!(supply.cw20_supply, foo_cw20.to_asset(200_u128.into_decimal()).amount_raw());
    assert_eq!(supply.native_supply, Uint128::zero());
    assert_eq!(supply.total_supply, supply.cw20_supply);

    mint(&mut app, &def, &user_1, &foo_addr, foo_cw20.to_asset(100_u128.into_decimal())).unwrap();

    let supply = qy_supply(&app, &foo_addr);
    assert_eq!(supply.cw20_supply, foo_cw20.to_asset(300_u128.into_decimal()).amount_raw());
    assert_eq!(supply.native_supply, Uint128::zero());
    assert_eq!(supply.total_supply, supply.cw20_supply);

    mint(&mut app, &def, &user_1, &foo_addr, foo_native.to_asset(100_u128.into_decimal())).unwrap_err_contains("Item factory_denom on contract cw20_factory can't be loaded");

    let fee_token_creation = AssetPrecisioned::new_super(AssetInfo::native("uosmo"), 6, 100_u128.into_decimal());
    let tf_fee_collector = app.generate_addr("tf_fee_collector");

    db.borrow_mut().token_factory.fee_creation = CTokenFactoryFee { fee: vec![fee_token_creation.clone().try_into().unwrap()], fee_collector: tf_fee_collector }.wrap_some();

    create_native(&mut app, &user_1, &foo_addr, vec![]).unwrap_err_contains("Error on gather fee for denom creation");

    app.mint(&user_1, fee_token_creation.clone());

    db.borrow_mut().token_factory.supplies.remove("factory/osmo14hj2tavq8fpesdwxxcu44rty3hh90vhujrvcmstl4zr3txmfvw9sq2r9g9/foo");

    create_native(&mut app, &user_1, &foo_addr, vec![fee_token_creation.try_into().unwrap()]).unwrap();

    mint(&mut app, &def, &user_1, &foo_addr, foo_native.to_asset(100_u128.into_decimal())).unwrap();

    let supply = qy_supply(&app, &foo_addr);
    assert_eq!(supply.cw20_supply, foo_cw20.to_asset(300_u128.into_decimal()).amount_raw());
    assert_eq!(supply.native_supply, foo_native.to_asset(100_u128.into_decimal()).amount_raw());
    assert_eq!(supply.total_supply, supply.cw20_supply + supply.native_supply);

    burn(&mut app, &user_1, &foo_addr, foo_cw20.to_asset(200_u128.into_decimal())).unwrap();

    burn(&mut app, &user_1, &foo_addr, foo_native.to_asset(50_u128.into_decimal())).unwrap();

    let supply = qy_supply(&app, &foo_addr);
    assert_eq!(supply.cw20_supply, foo_cw20.to_asset(100_u128.into_decimal()).amount_raw());
    assert_eq!(supply.native_supply, foo_native.to_asset(50_u128.into_decimal()).amount_raw());
    assert_eq!(supply.total_supply, supply.cw20_supply + supply.native_supply);

}
