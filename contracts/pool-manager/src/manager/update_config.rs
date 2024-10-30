use amm::pool_manager::{Config, FeatureToggle};
use cosmwasm_std::{Coin, DepsMut, MessageInfo, Response};

use crate::{state::CONFIG, ContractError};

pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    fee_collector_addr: Option<String>,
    farm_manager_addr: Option<String>,
    pool_creation_fee: Option<Coin>,
    feature_toggle: Option<FeatureToggle>,
) -> Result<Response, ContractError> {
    // permission check
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    CONFIG.update(deps.storage, |mut config| {
        if let Some(new_fee_collector_addr) = fee_collector_addr {
            let fee_collector_addr = deps.api.addr_validate(&new_fee_collector_addr)?;
            config.fee_collector_addr = fee_collector_addr;
        }

        if let Some(new_farm_manager_addr) = farm_manager_addr {
            let farm_manager_addr = deps.api.addr_validate(&new_farm_manager_addr)?;
            config.farm_manager_addr = farm_manager_addr;
        }

        if let Some(pool_creation_fee) = pool_creation_fee {
            config.pool_creation_fee = pool_creation_fee;
        }

        if let Some(feature_toggle) = feature_toggle {
            config.feature_toggle = feature_toggle;
        }
        Ok::<Config, ContractError>(config)
    })?;

    Ok(Response::default().add_attribute("action", "update_config"))
}
