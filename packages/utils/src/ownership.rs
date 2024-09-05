use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw_ownable::{Action, OwnershipError};

/// Updates the ownership of a contract using the cw_ownable package, which needs to be implemented by the contract.
pub fn update_ownership(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    action: Action,
) -> Result<Response, OwnershipError> {
    cw_ownable::update_ownership(deps, &env.block, &info.sender, action).map(|ownership| {
        Response::default()
            .add_attribute("action", "update_ownership")
            .add_attributes(ownership.into_attributes())
    })
}
