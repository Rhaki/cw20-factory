use cosmwasm_std::{testing::MockStorage, Addr, Coin, CosmosMsg, WasmMsg};
use cw20_base::msg::{InstantiateMsg as Cw20BaseInstantiateMsg, MigrateMsg};
use cw20_factory_pkg::{
    cw20_factory::{
        definitions::TransmuteIntoMsg,
        msgs::{
            ExecuteMsg, InstantiateMsg as FactoryInstantiateMsg, QueryMsg as FactoryQueryMsg,
            SupplyDetailsResponse,
        },
    },
    cw20_indexer::msgs::InstantiateMsg as IndexerInstantiateMsg,
};
use rhaki_cw_plus::{
    asset::AssetPrecisioned,
    cw_asset::AssetInfo,
    multi_test::{
        custom_app::CModuleWrapper,
        custom_chains::osmosis::{build_osmosis_app, OsmosisStargateModule},
        helper::{
            anyhow::Result as AnyResult,
            create_code,
            cw_multi_test::{
                addons::MockApiBech32, App, AppResponse, BankKeeper, DistributionKeeper, Executor,
                GovFailingModule, IbcFailingModule, StakeKeeper,
            },
            Bench32AppExt, DefaultWasmKeeper, FailingCustom,
        },
    },
    traits::Wrapper,
    wasm::WasmMsgBuilder,
};

pub struct Def {
    pub owner: Addr,
    pub code_id_cw20_factory: u64,
    pub code_id_cw20_base: u64,
    pub indexer_addr: Addr,
}

pub type OsmosisApp = App<
    BankKeeper,
    MockApiBech32,
    MockStorage,
    FailingCustom,
    DefaultWasmKeeper,
    StakeKeeper,
    DistributionKeeper,
    IbcFailingModule,
    GovFailingModule,
    OsmosisStargateModule,
>;

pub fn startup_osmosis() -> (OsmosisApp, CModuleWrapper, Def) {
    let (mut app, db) = build_osmosis_app();

    let code_id_cw20_base = app.store_code(Box::new(
        create_code(
            cw20_base::contract::instantiate,
            cw20_base::contract::execute,
            cw20_base::contract::query,
        )
        .with_migrate(cw20_base::contract::migrate),
    ));

    let code_id_cw20_factory = app.store_code(Box::new(
        create_code(
            cw20_factory_osmosis::instantiate,
            cw20_factory_osmosis::execute,
            cw20_factory_osmosis::query,
        )
        .with_migrate(cw20_factory_osmosis::migrate),
    ));

    let code_id_indexer = app.store_code(create_code(
        cw20_factory_indexer::contract::instantiate,
        cw20_factory_indexer::contract::execute,
        cw20_factory_indexer::contract::query,
    ));

    let owner = app.generate_addr("owner");

    let indexer_addr = app
        .instantiate_contract(
            code_id_indexer,
            owner.clone(),
            &IndexerInstantiateMsg {},
            &[],
            "indexer",
            owner.to_string().wrap_some(),
        )
        .unwrap();

    let def = Def {
        owner,
        code_id_cw20_factory,
        code_id_cw20_base,
        indexer_addr,
    };

    (app, db, def)
}

pub fn create_cw20_factory(
    app: &mut OsmosisApp,
    def: &Def,
    msg: FactoryInstantiateMsg,
    funds: Vec<Coin>,
) -> AnyResult<Addr> {
    let token_addr = app.instantiate_contract(
        def.code_id_cw20_factory,
        def.owner.clone(),
        &msg,
        &funds,
        "token",
        def.owner.to_string().wrap_some(),
    )?;

    Ok(token_addr)
}

pub fn create_cw20_base(
    app: &mut OsmosisApp,
    def: &Def,
    msg: Cw20BaseInstantiateMsg,
    funds: Vec<Coin>,
) -> AnyResult<Addr> {
    let token_addr = app.instantiate_contract(
        def.code_id_cw20_base,
        def.owner.clone(),
        &msg,
        &funds,
        "token",
        def.owner.to_string().wrap_some(),
    )?;

    Ok(token_addr)
}

pub fn migrate_to_factory(
    app: &mut OsmosisApp,
    def: &Def,
    cw20_addr: &Addr,
) -> AnyResult<AppResponse> {
    app.migrate_contract(
        def.owner.clone(),
        cw20_addr.clone(),
        &MigrateMsg {},
        def.code_id_cw20_factory,
    )
}

pub fn transmute(
    app: &mut OsmosisApp,
    sender: &Addr,
    token_addr: &Addr,
    amount: AssetPrecisioned,
) -> AnyResult<AppResponse> {
    let msg: CosmosMsg = match amount.info() {
        rhaki_cw_plus::cw_asset::AssetInfoBase::Native(_) => WasmMsg::build_execute(
            token_addr,
            ExecuteMsg::TransmuteInto(TransmuteIntoMsg::Cw20 {}),
            vec![amount.try_into().unwrap()],
        )
        .unwrap()
        .into(),
        rhaki_cw_plus::cw_asset::AssetInfoBase::Cw20(_) => WasmMsg::build_execute(
            token_addr,
            ExecuteMsg::TransmuteInto(TransmuteIntoMsg::Native {
                amount: amount.amount_raw(),
            }),
            vec![],
        )
        .unwrap()
        .into(),
        _ => todo!(),
    };

    app.execute(sender.clone(), msg)
}

pub fn qy_factory_denom(app: &OsmosisApp, cw20_addr: &Addr) -> String {
    app.wrap()
        .query_wasm_smart(cw20_addr, &FactoryQueryMsg::FactoryDenom {})
        .unwrap()
}

pub fn qy_supply(app: &OsmosisApp, cw20_addr: &Addr) -> SupplyDetailsResponse {
    app.wrap()
        .query_wasm_smart(cw20_addr, &FactoryQueryMsg::SupplyDetails {})
        .unwrap()
}

pub fn burn(
    app: &mut OsmosisApp,
    sender: &Addr,
    token_addr: &Addr,
    amount: AssetPrecisioned,
) -> AnyResult<AppResponse> {
    let msg: CosmosMsg = match amount.info() {
        rhaki_cw_plus::cw_asset::AssetInfoBase::Native(_) => WasmMsg::build_execute(
            token_addr,
            ExecuteMsg::Burn { amount: None },
            vec![amount.try_into().unwrap()],
        )
        .unwrap()
        .into(),
        rhaki_cw_plus::cw_asset::AssetInfoBase::Cw20(_) => WasmMsg::build_execute(
            token_addr,
            ExecuteMsg::Burn {
                amount: amount.amount_raw().wrap_some(),
            },
            vec![],
        )
        .unwrap()
        .into(),
        _ => todo!(),
    };

    app.execute(sender.clone(), msg)
}

#[allow(clippy::needless_bool)]
pub fn mint(
    app: &mut OsmosisApp,
    def: &Def,
    to: &Addr,
    token_addr: &Addr,
    amount: AssetPrecisioned,
) -> AnyResult<AppResponse> {
    let msg = WasmMsg::build_execute(
        token_addr,
        ExecuteMsg::Mint {
            recipient: to.to_string(),
            amount: amount.amount_raw(),
            as_native: if matches!(amount.info(), AssetInfo::Native(_)) {
                true
            } else {
                false
            }
            .wrap_some(),
        },
        vec![],
    )
    .unwrap();

    app.execute(def.owner.clone(), msg.into())
}

pub fn create_native(
    app: &mut OsmosisApp,
    sender: &Addr,
    cw20_addr: &Addr,
    funds: Vec<Coin>,
) -> AnyResult<AppResponse> {
    app.execute_contract(
        sender.clone(),
        cw20_addr.clone(),
        &ExecuteMsg::CreateNative {},
        &funds,
    )
}
