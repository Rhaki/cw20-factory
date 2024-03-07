use std::marker::PhantomData;

use cosmwasm_std::{
    attr, Addr, Binary, Coin, CosmosMsg, CustomQuery, Deps, DepsMut, Empty, Env, MessageInfo,
    Response, StdResult, Storage, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use cw20_base::{msg::QueryMsg as Cw20QueryMsg, state::BALANCES};

use cw20_factory_pkg::{
    cw20_factory::{
        definitions::TransmuteInto,
        interface::TokenFactoryInterface,
        msgs::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg},
        traits::IntoCustom,
        ContractResponse, ContractResult, Cw20FactoryError,
    },
    cw20_indexer::msgs::RegisterDenomMsg,
};
use rhaki_cw_plus::{
    traits::{FromBinary, IntoAddr, IntoBinary, IntoBinaryResult, Wrapper},
    wasm::WasmMsgBuilder,
};

use crate::state::FACTORY_DENOM;

pub struct Cw20FactoryBase<CQ: CustomQuery, I: TokenFactoryInterface<CQ, CM>, CM = Empty> {
    pub custom_query: PhantomData<CQ>,
    pub interface: PhantomData<I>,
    pub custom_msg: PhantomData<CM>,
}

// Entry points

impl<CQ, I, CM> Cw20FactoryBase<CQ, I, CM>
where
    CQ: CustomQuery,
    I: TokenFactoryInterface<CQ, CM>,
{
    pub fn instantiate(
        mut deps: DepsMut<CQ>,
        env: Env,
        info: MessageInfo,
        msg: InstantiateMsg,
    ) -> ContractResponse<CM> {
        let base_response = cw20_base::contract::instantiate(
            deps.branch().into_empty(),
            env.clone(),
            info.clone(),
            msg.clone().into(),
        )?;

        let interface_response = I::instantiate(deps.branch(), &env, info, msg.clone())?;

        let indexer_msg = if let Some(indexer) = msg.indexer {
            let msg: CosmosMsg = WasmMsg::build_execute(
                indexer.into_addr(deps.api)?,
                cw20_factory_pkg::cw20_indexer::msgs::ExecuteMsg::RegisterDenom(RegisterDenomMsg {
                    denom: interface_response.factory_denom.clone(),
                }),
                vec![],
            )?
            .into();
            vec![msg.to_custom()]
        } else {
            vec![]
        };

        FACTORY_DENOM.save(deps.storage, &interface_response.factory_denom)?;

        Response::<CM>::new()
            .add_attributes(base_response.attributes)
            .add_attributes(interface_response.attributes)
            .add_attribute("factory_denom", interface_response.factory_denom)
            .add_submessages(base_response.messages.to_custom())
            .add_messages(interface_response.messages)
            .add_messages(indexer_msg)
            .wrap_ok()
    }

    pub fn execute(
        deps: DepsMut<CQ>,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> ContractResponse<CM> {
        match msg {
            ExecuteMsg::TransmuteInto(into) => Self::run_transmute(deps, env, info, into),
            _ => {
                let base: Cw20ExecuteMsg = msg.into_binary()?.des_into()?;

                cw20_base::contract::execute(deps.into_empty(), env.clone(), info.clone(), base)?
                    .to_custom()
                    .wrap_ok()
            }
        }
    }

    pub fn query(deps: Deps<CQ>, env: Env, msg: QueryMsg) -> StdResult<Binary> {
        match msg {
            QueryMsg::FactoryDenom {} => FACTORY_DENOM.load(deps.storage).into_binary(),
            _ => {
                let base: Cw20QueryMsg = msg.into_binary()?.des_into()?;
                cw20_base::contract::query(deps.into_empty(), env.clone(), base)
            }
        }
    }

    pub fn migrate(deps: DepsMut<CQ>, env: Env, msg: MigrateMsg) -> ContractResponse<CM> {
        let base: cw20_base::msg::MigrateMsg = msg.into_binary()?.des_into()?;
        cw20_base::contract::migrate(deps.into_empty(), env.clone(), base)?
            .to_custom()
            .wrap_ok()
    }
}

// Execute

impl<CQ, I, CM> Cw20FactoryBase<CQ, I, CM>
where
    CQ: CustomQuery,
    I: TokenFactoryInterface<CQ, CM>,
{
    pub fn run_transmute(
        deps: DepsMut<CQ>,
        env: Env,
        info: MessageInfo,
        into: TransmuteInto,
    ) -> ContractResponse<CM> {
        let (msgs, attrs) = match into {
            TransmuteInto::Native { amount } => {
                reduce_cw20_balance(deps.storage, &info.sender, amount)?;
                let amount = Coin::new(amount.u128(), FACTORY_DENOM.load(deps.storage)?);
                let msgs_mint = I::mint(deps, &env, info, &amount)?;
                (
                    msgs_mint,
                    vec![
                        attr("action", "transmuted_cw20"),
                        attr("amount", amount.amount),
                    ],
                )
            }
            TransmuteInto::Cw20 {} => {
                let received_coin = rhaki_cw_plus::asset::only_one_coin(&info.funds, None)?;
                assert_denom(deps.storage, &received_coin)?;
                increase_cw20_balance(deps.storage, &info.sender, received_coin.amount)?;
                let msg_burn = I::burn(deps, &env, info, &received_coin)?;
                (
                    msg_burn,
                    vec![
                        attr("action", "transmuted_native"),
                        attr("amount", received_coin.amount),
                    ],
                )
            }
        };

        Response::<CM>::new()
            .add_attributes(attrs)
            .add_messages(msgs)
            .wrap_ok()
    }
}

fn assert_denom(storage: &dyn Storage, coin: &Coin) -> ContractResult<()> {
    let denom = FACTORY_DENOM.load(storage)?;

    if denom != coin.denom {
        Err(Cw20FactoryError::InvalidDenom {
            expected: denom,
            received: coin.denom.clone(),
        })
    } else {
        Ok(())
    }
}

fn reduce_cw20_balance(
    storage: &mut dyn Storage,
    user: &Addr,
    amount: Uint128,
) -> ContractResult<Uint128> {
    BALANCES.update(storage, user, |balance| {
        Ok(balance
            .unwrap_or_default()
            .checked_sub(amount)
            .map_err(|_| Cw20FactoryError::InsufficientCw20Balance {
                current: balance.unwrap_or_default(),
                requested: amount,
            })?)
    })
}

fn increase_cw20_balance(
    storage: &mut dyn Storage,
    user: &Addr,
    amount: Uint128,
) -> ContractResult<Uint128> {
    BALANCES.update(storage, user, |balance| {
        Ok(balance.unwrap_or_default() + amount)
    })
}
