use cosmwasm_schema::cw_serde;
use cosmwasm_std::{AnyMsg, CosmosMsg, StdResult};

#[cw_serde]
enum Protocol {
    Osmosis,
}

impl Protocol {
    #![allow(dead_code)]
    fn from_features() -> Self {
        Self::Osmosis
    }
    #[allow(unused_assignments)]
    fn as_str(&self) -> &'static str {
        match self {
            Self::Osmosis => "osmosis",
        }
    }
}

#[allow(dead_code)]
pub(crate) enum MsgTypes {
    CreateDenom,
    Mint,
    Burn,
}

impl MsgTypes {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CreateDenom => "MsgCreateDenom",
            Self::Mint => "MsgMint",
            Self::Burn => "MsgBurn",
        }
    }
}

pub trait EncodeMessage {
    /// Encodes the data as a proto doc
    fn encode(data: Self) -> Vec<u8>;

    /// Decodes the data from a proto doc. Only used for tests.
    fn decode(data: Vec<u8>) -> StdResult<Self>
    where
        Self: Sized;
}

#[allow(dead_code)]
pub(crate) fn create_msg<M: EncodeMessage>(message_data: M, msg_type: &str) -> CosmosMsg {
    CosmosMsg::Any(AnyMsg {
        type_url: format!(
            "/{}.tokenfactory.v1beta1.{}",
            Protocol::from_features().as_str(),
            msg_type
        ),
        value: M::encode(message_data).into(),
    })
}
