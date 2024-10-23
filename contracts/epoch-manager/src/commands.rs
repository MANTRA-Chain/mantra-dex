use cosmwasm_std::{DepsMut, MessageInfo, Response};

use amm::epoch_manager::EpochConfig;

use crate::helpers::validate_epoch_duration;
use crate::state::CONFIG;
use crate::ContractError;

/// Updates the config of the contract.
pub fn update_config(
    deps: DepsMut,
    info: &MessageInfo,
    epoch_config: Option<EpochConfig>,
) -> Result<Response, ContractError> {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let mut config = CONFIG.load(deps.storage)?;

    if let Some(epoch_config) = epoch_config.clone() {
        validate_epoch_duration(epoch_config.duration)?;
        config.epoch_config = epoch_config;
        CONFIG.save(deps.storage, &config)?;
    }

    Ok(Response::default().add_attributes(vec![
        ("action", "update_config".to_string()),
        (
            "epoch_config",
            epoch_config.unwrap_or(config.epoch_config).to_string(),
        ),
    ]))
}
