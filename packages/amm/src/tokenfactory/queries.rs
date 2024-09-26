use cosmwasm_schema::cw_serde;
use cosmwasm_std::{to_json_binary, Coin, GrpcQuery, QueryRequest, StdResult};

use crate::tokenfactory::common::MsgTypes;

/// Queries the token factory params via a grpc query.
pub fn query_token_factory_params() -> StdResult<QueryRequest> {
    Ok(QueryRequest::Grpc(GrpcQuery {
        path: format!(
            "/{}.tokenfactory.v1beta1.{}",
            crate::tokenfactory::common::Protocol::from_features().as_str(),
            MsgTypes::QueryParams.as_str()
        ),
        data: to_json_binary(&QueryParamsRequest {})?,
    }))
}

///QueryParamsRequest is the type for the token factory params query.
#[cw_serde]
pub struct QueryParamsRequest {}

///QueryParamsResponse is the response type for the Query/Params RPC method.
#[cw_serde]
pub struct QueryParamsResponse {
    pub params: Option<Params>,
}

/// Params defines the parameters for the token factory module.
#[cw_serde]
pub struct Params {
    pub denom_creation_fee: Vec<Coin>,
    pub denom_creation_gas_consume: u64,
    pub fee_collector_address: String,
}
