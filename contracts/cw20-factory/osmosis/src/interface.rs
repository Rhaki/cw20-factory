use cosmwasm_std::{Addr, Coin, CosmosMsg, DepsMut, Empty, Env, MessageInfo};
use cw20_factory_pkg::cw20_factory::{
    interface::{InterfaceInstantiateResponse, TokenFactoryInterface},
    msgs::InstantiateMsg,
    ContractResult,
};
use osmosis_std::types::{
    cosmos::base::v1beta1::Coin as CosmosCoin,
    osmosis::tokenfactory::v1beta1::{MsgBurn, MsgCreateDenom, MsgMint},
};
use rhaki_cw_plus::traits::Wrapper;

pub struct OsmosisTokenFactoryInterface {}

impl TokenFactoryInterface for OsmosisTokenFactoryInterface {
    fn instantiate(
        _deps: DepsMut<Empty>,
        env: &Env,
        _info: MessageInfo,
        msg: InstantiateMsg,
    ) -> ContractResult<InterfaceInstantiateResponse<Empty>> {
        let subdenom = msg.symbol.to_lowercase();
        let msg = MsgCreateDenom {
            sender: env.contract.address.to_string(),
            subdenom: subdenom.clone(),
        }
        .to_any();

        let msg = CosmosMsg::Stargate {
            type_url: MsgCreateDenom::TYPE_URL.to_string(),
            value: msg.value.into(),
        };

        Ok(InterfaceInstantiateResponse {
            attributes: vec![],
            messages: vec![msg],
            factory_denom: denom(&env.contract.address, subdenom),
        })
    }

    fn burn(
        _deps: DepsMut<Empty>,
        env: &Env,
        _info: &MessageInfo,
        amount: &Coin,
    ) -> ContractResult<Vec<CosmosMsg<Empty>>> {
        let msg = MsgBurn {
            sender: env.contract.address.to_string(),
            amount: amount.clone().to_cosmos_coin().wrap_some(),
            burn_from_address: env.contract.address.to_string(),
        }
        .to_any();

        vec![CosmosMsg::Stargate {
            type_url: MsgBurn::TYPE_URL.to_string(),
            value: msg.value.into(),
        }]
        .wrap_ok()
    }

    fn mint(
        _deps: DepsMut<Empty>,
        env: &Env,
        _info: &MessageInfo,
        to: &Addr,
        amount: &Coin,
    ) -> ContractResult<Vec<CosmosMsg<Empty>>> {
        let msg = MsgMint {
            sender: env.contract.address.to_string(),
            amount: amount.clone().to_cosmos_coin().wrap_some(),
            mint_to_address: to.to_string(),
        }
        .to_any();

        vec![CosmosMsg::Stargate {
            type_url: MsgMint::TYPE_URL.to_string(),
            value: msg.value.into(),
        }]
        .wrap_ok()
    }
}

fn denom(contract: &Addr, subdenom: String) -> String {
    format!("factory/{}/{}", contract, subdenom)
}

trait IntoCosmosCoin {
    fn to_cosmos_coin(self) -> CosmosCoin;
}

impl IntoCosmosCoin for Coin {
    fn to_cosmos_coin(self) -> CosmosCoin {
        CosmosCoin {
            denom: self.denom,
            amount: self.amount.to_string(),
        }
    }
}
