use cosmwasm_std::{CosmosMsg, Empty, Response, SubMsg};

pub trait IntoCustom<T> {
    type Output;

    fn to_custom(self) -> Self::Output;
}

impl<T> IntoCustom<T> for Vec<CosmosMsg<Empty>> {
    type Output = Vec<CosmosMsg<T>>;
    fn to_custom(self) -> Vec<CosmosMsg<T>> {
        self.into_iter().map(|msg| msg.to_custom()).collect()
    }
}

impl<T> IntoCustom<T> for Vec<SubMsg<Empty>> {
    type Output = Vec<SubMsg<T>>;
    fn to_custom(self) -> Vec<SubMsg<T>> {
        self.into_iter()
            .map(|msg| SubMsg {
                id: msg.id,
                msg: msg.msg.to_custom(),
                gas_limit: msg.gas_limit,
                reply_on: msg.reply_on,
            })
            .collect()
    }
}

impl<T> IntoCustom<T> for CosmosMsg<Empty> {
    type Output = CosmosMsg<T>;

    fn to_custom(self) -> CosmosMsg<T> {
        match self {
            CosmosMsg::Bank(msg) => CosmosMsg::<T>::Bank(msg),
            CosmosMsg::Wasm(msg) => CosmosMsg::<T>::Wasm(msg),
            CosmosMsg::Staking(msg) => CosmosMsg::<T>::Staking(msg),
            CosmosMsg::Distribution(msg) => CosmosMsg::<T>::Distribution(msg),
            CosmosMsg::Stargate { type_url, value } => CosmosMsg::<T>::Stargate { type_url, value },
            CosmosMsg::Ibc(msg) => CosmosMsg::<T>::Ibc(msg),
            CosmosMsg::Gov(msg) => CosmosMsg::<T>::Gov(msg),
            _ => unimplemented!("Unsupported CosmosMsg type"),
        }
    }
}

impl<T> IntoCustom<T> for Response<Empty> {
    type Output = Response<T>;

    fn to_custom(self) -> Self::Output {
        Response::<T>::new()
            .add_attributes(self.attributes)
            .add_messages(self.messages.into_iter().map(|val| val.msg.to_custom()))
            .add_events(self.events)
            .set_data(self.data.unwrap_or_default())
    }
}
