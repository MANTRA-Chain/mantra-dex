use cosmwasm_std::{DepsMut, MessageInfo, Response};

use amm::epoch_manager::EpochConfig;

use crate::state::{ADMIN, CONFIG};
use crate::ContractError;

/// Updates the config of the contract.
pub fn update_config(
    mut deps: DepsMut,
    info: &MessageInfo,
    owner: Option<String>,
    epoch_config: Option<EpochConfig>,
) -> Result<Response, ContractError> {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    if let Some(owner) = owner.clone() {
        let new_admin = deps.api.addr_validate(owner.as_str())?;
        ADMIN.set(deps.branch(), Some(new_admin))?;
    }

    let mut config = CONFIG.load(deps.storage)?;

    if let Some(epoch_config) = epoch_config.clone() {
        config.epoch_config = epoch_config;
        CONFIG.save(deps.storage, &config)?;
    }

    Ok(Response::default().add_attributes(vec![
        ("action", "update_config".to_string()),
        ("owner", owner.unwrap_or_else(|| info.sender.to_string())),
        (
            "epoch_config",
            epoch_config.unwrap_or(config.epoch_config).to_string(),
        ),
    ]))
}
