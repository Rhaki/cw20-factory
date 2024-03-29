use std::marker::PhantomData;

use cosmwasm_std::{
    attr, Addr, Binary, Coin, CosmosMsg, CustomQuery, Deps, DepsMut, Empty, Env, Int256,
    MessageInfo, Response, StdResult, Storage, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, TokenInfoResponse};
use cw20_base::{
    msg::QueryMsg as Cw20QueryMsg,
    state::{BALANCES, TOKEN_INFO},
    ContractError as Cw20BaseError,
};

use cw20_factory_pkg::{
    cw20_factory::{
        definitions::TransmuteIntoMsg,
        interface::TokenFactoryInterface,
        msgs::{
            ExecuteMsg, InitNativeDetails, InstantiateMsg, MigrateMsg, QueryMsg,
            SupplyDetailsResponse,
        },
        traits::IntoCustom,
        ContractResponse, ContractResult, Cw20FactoryError,
    },
    cw20_indexer::msgs::RegisterDenomMsg,
};
use rhaki_cw_plus::{
    storage::interfaces::ItemInterface,
    traits::{FromBinary, IntoAddr, IntoBinary, IntoBinaryResult, IntoStdResult, Wrapper},
    wasm::WasmMsgBuilder,
};

use crate::state::FactoryDenom;

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

        let mut response = Response::new()
            .add_attributes(base_response.attributes)
            .add_submessages(base_response.messages.to_custom());

        if let Some(init_native) = msg.init_native {
            let interface_response = I::instantiate(deps.branch(), &env, info, msg.symbol)?;

            response = response.add_messages(interface_response.messages);

            if let InitNativeDetails::WithIndexer(indexer) = init_native {
                let indexer_msg: CosmosMsg<CM> = WasmMsg::build_execute(
                    indexer.into_addr(deps.api)?,
                    cw20_factory_pkg::cw20_indexer::msgs::ExecuteMsg::RegisterDenom(
                        RegisterDenomMsg {
                            denom: interface_response.factory_denom.clone(),
                        },
                    ),
                    vec![],
                )?
                .into();
                response = response.add_message(indexer_msg);
            }

            let factory_denom = FactoryDenom::new(interface_response.factory_denom.clone());
            factory_denom.save(deps.storage)?;

            response = response
                .add_attributes(interface_response.attributes)
                .add_attribute("factory_denom", interface_response.factory_denom);
        }

        response.wrap_ok()
    }

    pub fn execute(
        deps: DepsMut<CQ>,
        env: Env,
        info: MessageInfo,
        msg: ExecuteMsg,
    ) -> ContractResponse<CM> {
        match msg {
            ExecuteMsg::TransmuteInto(into) => Self::run_transmute(deps, env, info, into),
            ExecuteMsg::Burn { amount } => Self::run_burn(deps, env, info, amount),
            ExecuteMsg::Mint {
                recipient,
                amount,
                as_native,
            } => Self::run_mint(deps, env, info, recipient, amount, as_native),
            ExecuteMsg::RegisterToIndexer { indexer_addr } => {
                Self::run_register_into_indexer(deps, indexer_addr)
            }
            ExecuteMsg::CreateNative {} => Self::run_create_native(deps, env, info),
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
            QueryMsg::FactoryDenom {} => FactoryDenom::load(deps.storage).into_binary(),
            QueryMsg::SupplyDetails {} => Self::qy_supply_details(deps.into_empty())
                .into_std_result()
                .into_binary(),
            QueryMsg::TokenInfo {} => Self::qy_token_info(deps.into_empty())
                .into_std_result()
                .into_binary(),
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
        mut deps: DepsMut<CQ>,
        env: Env,
        info: MessageInfo,
        into: TransmuteIntoMsg,
    ) -> ContractResponse<CM> {
        let (msgs, attrs) = match into {
            TransmuteIntoMsg::Native { amount } => {
                Self::burn_cw20(deps.branch().into_empty(), &info.sender, amount)?;
                let mint_coin = Coin::new(amount.u128(), FactoryDenom::load(deps.storage)?.inner());
                (
                    I::mint(deps.branch(), &env, &info, &info.sender, &mint_coin)?,
                    vec![
                        attr("action", "transumte_into_native"),
                        attr("amount", mint_coin.amount),
                    ],
                )
            }
            TransmuteIntoMsg::Cw20 {} => {
                let brun_coin = rhaki_cw_plus::asset::only_one_coin(&info.funds, None)?;
                Self::assert_denom(deps.storage, &brun_coin)?;
                Self::mint_cw20(deps.branch().into_empty(), &info.sender, brun_coin.amount)?;
                (
                    I::burn(deps.branch(), &env, &info, &brun_coin)?,
                    vec![
                        attr("action", "transumte_into_cw20"),
                        attr("amount", brun_coin.amount),
                    ],
                )
            }
        };

        Response::new()
            .add_attributes(attrs)
            .add_messages(msgs)
            .wrap_ok()
    }

    pub fn run_mint(
        mut deps: DepsMut<CQ>,
        env: Env,
        info: MessageInfo,
        recipient: String,
        amount: Uint128,
        as_native: Option<bool>,
    ) -> ContractResponse<CM> {
        Self::assert_minter(deps.as_ref().into_empty(), &info.sender)?;
        let recipient = recipient.into_addr(deps.api)?;

        let (msgs, action) = match as_native.unwrap_or(false) {
            true => {
                let mint_coin = Coin::new(amount.u128(), FactoryDenom::load(deps.storage)?.inner());
                Self::validate_max_supply(deps.as_ref().into_empty(), amount.wrap_some())?;
                (
                    I::mint(deps.branch(), &env, &info, &recipient, &mint_coin)?,
                    "mint_native",
                )
            }
            false => {
                Self::mint_cw20(deps.branch().into_empty(), &recipient, amount)?;
                Self::validate_max_supply(deps.as_ref().into_empty(), None)?;
                (vec![], "mint_cw20")
            }
        };

        Response::new()
            .add_attribute("action", action)
            .add_attribute("amount", amount)
            .add_messages(msgs)
            .wrap_ok()
    }

    pub fn run_burn(
        deps: DepsMut<CQ>,
        env: Env,
        info: MessageInfo,
        amount: Option<Uint128>,
    ) -> ContractResponse<CM> {
        let (msgs, attrs) = if info.funds.is_empty() {
            let amount = amount.ok_or(Cw20FactoryError::InvalidZeroBurnamount {})?;
            Self::burn_cw20(deps.into_empty(), &info.sender, amount)?;
            (
                vec![],
                vec![attr("action", "burn_cw20"), attr("amount", amount)],
            )
        } else {
            let denom = FactoryDenom::load(deps.storage)?.inner();
            let burn_coin = rhaki_cw_plus::asset::only_one_coin(&info.funds, Some(denom))?;
            (
                I::burn(deps, &env, &info, &burn_coin)?,
                vec![
                    attr("action", "burn_native"),
                    attr("amount", burn_coin.amount),
                ],
            )
        };

        Response::new()
            .add_attributes(attrs)
            .add_messages(msgs)
            .wrap_ok()
    }

    pub fn run_register_into_indexer(deps: DepsMut<CQ>, indexer: String) -> ContractResponse<CM> {
        let denom = FactoryDenom::load(deps.storage)?.inner();

        Response::new()
            .add_attribute("action", "register_into_indexer")
            .add_attribute("indexer", &indexer)
            .add_message(WasmMsg::build_execute(
                indexer.clone().into_addr(deps.api)?,
                cw20_factory_pkg::cw20_indexer::msgs::ExecuteMsg::RegisterDenom(RegisterDenomMsg {
                    denom,
                }),
                vec![],
            )?)
            .wrap_ok()
    }

    pub fn run_create_native(
        mut deps: DepsMut<CQ>,
        env: Env,
        info: MessageInfo,
    ) -> ContractResponse<CM> {
        let token_info = TOKEN_INFO.load(deps.storage)?;
        if FactoryDenom::load(deps.storage).is_ok() {
            return Err(Cw20FactoryError::NativeTokenAlredyCreated {});
        }
        let interface_response =
            I::instantiate(deps.branch(), &env, info, token_info.symbol.clone())?;

        let factory_denom = FactoryDenom::new(interface_response.factory_denom.clone());
        factory_denom.save(deps.storage)?;

        Response::new()
            .add_attribute("action", "create_native")
            .add_attributes(interface_response.attributes)
            .add_attribute("factory_denom", interface_response.factory_denom)
            .add_messages(interface_response.messages)
            .wrap_ok()
    }
}

// fn

impl<CQ, I, CM> Cw20FactoryBase<CQ, I, CM>
where
    CQ: CustomQuery,
    I: TokenFactoryInterface<CQ, CM>,
{
    fn mint_cw20(deps: DepsMut, to: &Addr, amount: Uint128) -> ContractResult<()> {
        Self::modify_cw20_balance(deps.storage, to, amount.into())?;
        Self::modify_cw20_supply(deps, amount.into())
    }

    fn burn_cw20(deps: DepsMut, user: &Addr, amount: Uint128) -> ContractResult<()> {
        let amount = -Into::<Int256>::into(amount);
        Self::modify_cw20_balance(deps.storage, user, amount)?;
        Self::modify_cw20_supply(deps, amount)
    }

    fn assert_denom(storage: &dyn Storage, coin: &Coin) -> ContractResult<()> {
        let denom = FactoryDenom::load(storage)?.inner();

        if denom != coin.denom {
            Err(Cw20FactoryError::InvalidDenom {
                expected: denom,
                received: coin.denom.clone(),
            })
        } else {
            Ok(())
        }
    }

    fn modify_cw20_balance(
        storage: &mut dyn Storage,
        user: &Addr,
        amount: Int256,
    ) -> ContractResult<Uint128> {
        BALANCES
            .update(storage, user, |balance| -> ContractResult<Uint128> {
                let mut current: Int256 = (balance.unwrap_or_default()).into();
                current += amount;
                current
                    .try_into()
                    .map_err(|_| Cw20FactoryError::InsufficientCw20Balance {
                        current: current + amount,
                        requested: amount,
                    })
            })?
            .wrap_ok()
    }

    fn modify_cw20_supply(deps: DepsMut, amount: Int256) -> ContractResult<()> {
        TOKEN_INFO.update(deps.storage, |mut info| -> StdResult<_> {
            let mut supply: Int256 = info.total_supply.into();
            supply += amount;
            info.total_supply = supply.try_into()?;
            Ok(info)
        })?;

        Ok(())
    }

    fn validate_max_supply(deps: Deps, extra_amount: Option<Uint128>) -> ContractResult<()> {
        let token_info = TOKEN_INFO.load(deps.storage)?;
        if let Some(minter) = token_info.mint {
            if let Some(cap) = minter.cap {
                let supply = Self::qy_supply_details(deps)?.total_supply;
                if supply + extra_amount.unwrap_or_default() > cap {
                    return Err(Cw20FactoryError::Base(Cw20BaseError::CannotExceedCap {}));
                }
            }
        }
        Ok(())
    }

    fn assert_minter(deps: Deps, sender: &Addr) -> ContractResult<()> {
        let token_info = TOKEN_INFO.load(deps.storage)?;

        if token_info
            .mint
            .as_ref()
            .ok_or(Cw20BaseError::Unauthorized {})?
            .minter
            != sender
        {
            Err(Cw20BaseError::Unauthorized {}.into())
        } else {
            Ok(())
        }
    }

    fn qy_token_info(deps: Deps) -> ContractResult<TokenInfoResponse> {
        let info = TOKEN_INFO.load(deps.storage)?;
        let supply = Self::qy_supply_details(deps)?.total_supply;
        Ok(TokenInfoResponse {
            name: info.name,
            symbol: info.symbol,
            decimals: info.decimals,
            total_supply: supply,
        })
    }

    fn qy_supply_details(deps: Deps) -> ContractResult<SupplyDetailsResponse> {
        let cw20_supply = TOKEN_INFO.load(deps.storage)?.total_supply;
        let native_supply = if let Ok(denom) = FactoryDenom::load(deps.storage) {
            deps.querier.query_supply(denom.inner())?.amount
        } else {
            Uint128::zero()
        };

        SupplyDetailsResponse {
            cw20_supply,
            native_supply,
            total_supply: cw20_supply + native_supply,
        }
        .wrap_ok()
    }
}
