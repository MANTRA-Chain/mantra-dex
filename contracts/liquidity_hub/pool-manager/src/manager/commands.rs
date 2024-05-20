use cosmwasm_std::{
    attr, Attribute, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, Uint128,
};

use white_whale_std::fee::PoolFee;

use crate::state::{get_pool_by_identifier, POOL_COUNTER};
use crate::{
    state::{Config, CONFIG, POOLS},
    ContractError,
};

use white_whale_std::lp_common::LP_SYMBOL;
use white_whale_std::pool_manager::{PoolInfo, PoolType};

pub const MAX_ASSETS_PER_POOL: usize = 4;

/// Creates a pool with 2, 3, or N assets. The function dynamically handles different numbers of assets,
/// allowing for the creation of pools with varying configurations. The maximum number of assets per pool is defined by
/// the constant `MAX_ASSETS_PER_POOL`.
///
/// # Example
///
/// ```rust
/// # use cosmwasm_std::{DepsMut, Decimal, Env, MessageInfo, Response, CosmosMsg, WasmMsg, to_json_binary};
/// # use white_whale_std::fee::PoolFee;
/// # use white_whale_std::fee::Fee;
/// # use pool_manager::error::ContractError;
/// # use pool_manager::manager::commands::MAX_ASSETS_PER_POOL;
/// # use pool_manager::manager::commands::create_pool;
/// # use std::convert::TryInto;
/// # use white_whale_std::pool_manager::PoolType;
/// #
/// # fn example(deps: DepsMut, env: Env, info: MessageInfo) -> Result<Response, ContractError> {
/// let asset_infos = vec![
///     "uatom".into(),
///     "uscrt".into(),
/// ];
/// let asset_decimals = vec![6, 6];
/// #[cfg(not(feature = "osmosis"))]
/// let pool_fees = PoolFee {
///     protocol_fee: Fee {
///         share: Decimal::percent(5u64),
///     },
///     swap_fee: Fee {
///         share: Decimal::percent(7u64),
///     },
///     burn_fee: Fee {
///         share: Decimal::zero(),
///     },
///    extra_fees: vec![],
/// };
///
/// #[cfg(feature = "osmosis")]
/// let pool_fees = PoolFee {
///     protocol_fee: Fee {
///         share: Decimal::percent(5u64),
///     },
///     swap_fee: Fee {
///         share: Decimal::percent(7u64),
///     },
///     burn_fee: Fee {
///         share: Decimal::zero(),
///     },
///     osmosis_fee: Fee {
///         share: Decimal::zero(),
///     },
///     extra_fees: vec![],
/// };
/// let pool_type = PoolType::ConstantProduct;
/// let token_factory_lp = false;
///
/// let response = create_pool(deps, env, info, asset_infos, asset_decimals, pool_fees, pool_type, None)?;
/// # Ok(response)
/// # }
/// ```
#[allow(unreachable_code)]
#[allow(clippy::too_many_arguments)]
pub fn create_pool(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    asset_denoms: Vec<String>,
    asset_decimals: Vec<u8>,
    pool_fees: PoolFee,
    pool_type: PoolType,
    pool_identifier: Option<String>,
) -> Result<Response, ContractError> {
    // Load config for pool creation fee
    let config: Config = CONFIG.load(deps.storage)?;

    // Check if fee was provided and is sufficient
    if !config.pool_creation_fee.amount.is_zero() {
        // verify fee payment
        let amount = cw_utils::must_pay(&info, &config.pool_creation_fee.denom)?;
        if amount != config.pool_creation_fee.amount {
            return Err(ContractError::InvalidPoolCreationFee {
                amount,
                expected: config.pool_creation_fee.amount,
            });
        }
    }

    // Prepare the sending of pool creation fee
    let mut messages: Vec<CosmosMsg> = vec![];

    // send pool creation fee to whale lair
    let creation_fee = vec![config.pool_creation_fee];

    // send pool creation fee to the bonding manager
    messages.push(white_whale_std::bonding_manager::fill_rewards_msg(
        config.bonding_manager_addr.into_string(),
        creation_fee,
    )?);

    // Check if the asset infos are the same
    if asset_denoms
        .iter()
        .any(|asset| asset_denoms.iter().filter(|&a| a == asset).count() > 1)
    {
        return Err(ContractError::SameAsset);
    }

    // Verify pool fees
    pool_fees.is_valid()?;

    let pool_id = POOL_COUNTER.load(deps.storage)?;
    // if no identifier is provided, use the pool counter (id) as identifier
    let identifier = pool_identifier.unwrap_or(pool_id.to_string());

    // check if there is an existing pool with the given identifier
    let pool = get_pool_by_identifier(&deps.as_ref(), &identifier);
    if pool.is_ok() {
        return Err(ContractError::PoolExists {
            asset_infos: asset_denoms
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(", "),
            identifier,
        });
    }

    // prepare labels for creating the pool token with a meaningful name
    let pool_label = asset_denoms.join("-");

    let mut attributes = Vec::<Attribute>::new();

    // Convert all asset_infos into assets with 0 balances
    let assets = asset_denoms
        .iter()
        .map(|asset_info| Coin {
            denom: asset_info.clone(),
            amount: Uint128::zero(),
        })
        .collect::<Vec<_>>();

    let lp_symbol = format!("{pool_label}.pool.{identifier}.{LP_SYMBOL}");
    let lp_asset = format!("{}/{}/{}", "factory", env.contract.address, lp_symbol);

    #[allow(clippy::redundant_clone)]
    POOLS.save(
        deps.storage,
        &identifier,
        &PoolInfo {
            asset_denoms,
            pool_type: pool_type.clone(),
            lp_denom: lp_asset.clone(),
            asset_decimals,
            pool_fees,
            assets,
        },
    )?;

    attributes.push(attr("lp_asset", lp_asset));

    messages.push(white_whale_std::tokenfactory::create_denom::create_denom(
        env.contract.address,
        lp_symbol,
    ));

    // increase pool counter
    POOL_COUNTER.update(deps.storage, |mut counter| -> Result<_, ContractError> {
        counter += 1;
        Ok(counter)
    })?;

    attributes.push(attr("action", "create_pool"));
    attributes.push(attr("pool", &pool_label));
    attributes.push(attr("pool_label", pool_label.as_str()));
    attributes.push(attr("pool_type", pool_type.get_label()));
    attributes.push(attr("pool_identifier", identifier.as_str()));

    Ok(Response::new()
        .add_attributes(attributes)
        .add_messages(messages))
}
