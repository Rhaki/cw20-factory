use cw20::MinterResponse;
use cw20_factory_pkg::cw20_factory::msgs::InstantiateMsg;
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

use crate::helper::{create_token, qy_factory_denom, startup_osmosis, transmute};

#[test]
#[rustfmt::skip]
fn t1() {
    let (mut app, db, def) = startup_osmosis();

    let minter_foo = app.generate_addr("minter_foo");

    let msg_init = InstantiateMsg {
        name: "Token Foo".to_string(),
        symbol: "FOO".to_string(),
        decimals: 6,
        initial_balances: vec![],
        mint: MinterResponse{ minter: minter_foo.to_string(), cap: None }.wrap_some(),
        marketing: None,
        indexer: def.indexer_addr.to_string().wrap_some(),
    };

    let foo_addr = create_token(&mut app, &def, msg_init, vec![]).unwrap();

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

    let msg_init = InstantiateMsg {
        name: "Token Bar".to_string(),
        symbol: "BAR".to_string(),
        decimals: 18,
        initial_balances: vec![],
        mint: MinterResponse{ minter: minter_foo.to_string(), cap: None }.wrap_some(),
        marketing: None,
        indexer: def.indexer_addr.to_string().wrap_some(),
    };

   create_token(&mut app, &def, msg_init.clone(), vec![]).unwrap_err_contains("Error on gather fee for denom creation");

   app.mint(&def.owner, fee_token_creation.clone());
   db.borrow_mut().token_factory.supplies.remove("factory/osmo1suhgf5svhu4usrurvxzlgn54ksxmn8gljarjtxqnapv8kjnp4nrsll0sqv/bar");

   create_token(&mut app, &def, msg_init, vec![fee_token_creation.try_into().unwrap()]).unwrap();

}
