use std::{cell::RefCell, rc::Rc};

use cosmwasm_std::{testing::MockStorage, Addr, Coin, CosmosMsg, WasmMsg};
use cw20_factory_pkg::{
    cw20_factory::{
        definitions::TransmuteInto,
        msgs::{ExecuteMsg, InstantiateMsg, QueryMsg},
    },
    cw20_indexer::msgs::InstantiateMsg as IndexerInstantiateMsg,
};
use rhaki_cw_plus::{
    asset::AssetPrecisioned,
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
    pub code_id_cw20: u64,
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

pub fn startup_osmosis() -> (OsmosisApp, Rc<RefCell<CModuleWrapper>>, Def) {
    let (mut app, db) = build_osmosis_app();

    let code_id_cw20 = app.store_code(create_code(
        cw20_factory_osmosis::instantiate,
        cw20_factory_osmosis::execute,
        cw20_factory_osmosis::query,
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
        code_id_cw20,
        indexer_addr,
    };

    (app, db, def)
}

pub fn create_token(
    app: &mut OsmosisApp,
    def: &Def,
    msg: InstantiateMsg,
    funds: Vec<Coin>,
) -> AnyResult<Addr> {
    let token_addr = app.instantiate_contract(
        def.code_id_cw20,
        def.owner.clone(),
        &msg,
        &funds,
        "token",
        def.owner.to_string().wrap_some(),
    )?;

    Ok(token_addr)
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
            ExecuteMsg::TransmuteInto(TransmuteInto::Cw20 {}),
            vec![amount.try_into().unwrap()],
        )
        .unwrap()
        .into(),
        rhaki_cw_plus::cw_asset::AssetInfoBase::Cw20(_) => WasmMsg::build_execute(
            token_addr,
            ExecuteMsg::TransmuteInto(TransmuteInto::Native {
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
        .query_wasm_smart(cw20_addr, &QueryMsg::FactoryDenom {})
        .unwrap()
}
