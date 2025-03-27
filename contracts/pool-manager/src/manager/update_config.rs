use cosmwasm_std::{Coin, DepsMut, MessageInfo, Response};
use mantra_dex_std::pool_manager::FeatureToggle;

use crate::state::{get_pool_by_identifier, POOLS};
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

    let mut config = CONFIG.load(deps.storage)?;

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
        let mut pool_info =
            get_pool_by_identifier(&deps.as_ref(), &feature_toggle.pool_identifier)?;

        let mut pool_info_changed = false;

        if let Some(swaps_enabled) = feature_toggle.swaps_enabled {
            pool_info_changed = true;
            pool_info.status.swaps_enabled = swaps_enabled;
        }

        if let Some(deposits_enabled) = feature_toggle.deposits_enabled {
            pool_info_changed = true;
            pool_info.status.deposits_enabled = deposits_enabled;
        }

        if let Some(withdrawals_enabled) = feature_toggle.withdrawals_enabled {
            pool_info_changed = true;
            pool_info.status.withdrawals_enabled = withdrawals_enabled;
        }

        if pool_info_changed {
            POOLS.save(deps.storage, &pool_info.pool_identifier, &pool_info)?;
        }
    }

    CONFIG.save(deps.storage, &config)?;

    Ok(Response::default().add_attribute("action", "update_config"))
}
