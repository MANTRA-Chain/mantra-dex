use std::clone::Clone;
use std::string::ToString;

use cosmwasm_std::{Addr, Order, StdResult, Storage, Uint128};
use cw_storage_plus::{Bound, Index, IndexList, IndexedMap, Item, Map, MultiIndex};

use amm::farm_manager::{Config, EpochId, Farm, Position};

use crate::ContractError;

/// Contract's config
pub const CONFIG: Item<Config> = Item::new("config");

/// A monotonically increasing counter to generate unique position identifiers.
pub const POSITION_ID_COUNTER: Item<u64> = Item::new("position_id_counter");

/// The positions that a user has. Positions can be open or closed.
/// The key is the position identifier
pub const POSITIONS: IndexedMap<&str, Position, PositionIndexes> = IndexedMap::new(
    "positions",
    PositionIndexes {
        receiver: MultiIndex::new(
            |_pk, p| p.receiver.to_string(),
            "positions",
            "positions__receiver",
        ),
        open_state_by_receiver: MultiIndex::new(
            |_pk, p| (p.receiver.as_bytes().to_vec(), p.open.into()),
            "positions",
            "positions__open_state_by_receiver",
        ),
    },
);

pub struct PositionIndexes<'a> {
    pub receiver: MultiIndex<'a, String, Position, String>,
    pub open_state_by_receiver: MultiIndex<'a, (Vec<u8>, u8), Position, String>,
}

impl<'a> IndexList<Position> for PositionIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Position>> + '_> {
        let v: Vec<&dyn Index<Position>> = vec![&self.receiver, &self.open_state_by_receiver];
        Box::new(v.into_iter())
    }
}

/// The last epoch an address claimed rewards
pub const LAST_CLAIMED_EPOCH: Map<&Addr, EpochId> = Map::new("last_claimed_epoch");

/// The lp weight history for addresses, including the contract. i.e. how much lp weight an address
/// or contract has at a given epoch.
/// Key is a tuple of (address, lp_denom, epoch_id), value is the lp weight.
pub const LP_WEIGHT_HISTORY: Map<(&Addr, &str, EpochId), Uint128> = Map::new("lp_weight_history");

/// A monotonically increasing counter to generate unique farm identifiers.
pub const FARM_COUNTER: Item<u64> = Item::new("farm_counter");

/// Farms map
pub const FARMS: IndexedMap<&str, Farm, FarmIndexes> = IndexedMap::new(
    "farms",
    FarmIndexes {
        lp_denom: MultiIndex::new(|_pk, f| f.lp_denom.to_string(), "farms", "farms__lp_asset"),
        farm_asset: MultiIndex::new(
            |_pk, f| f.farm_asset.denom.clone(),
            "farms",
            "farms__farm_asset",
        ),
    },
);

pub struct FarmIndexes<'a> {
    pub lp_denom: MultiIndex<'a, String, Farm, String>,
    pub farm_asset: MultiIndex<'a, String, Farm, String>,
}

impl<'a> IndexList<Farm> for FarmIndexes<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Farm>> + '_> {
        let v: Vec<&dyn Index<Farm>> = vec![&self.lp_denom, &self.farm_asset];
        Box::new(v.into_iter())
    }
}

// settings for pagination
// MAX_ITEMS_LIMIT in the case of positions, is the maximum number of positions that a user can have
// open or closed at a given time, i.e. there can be at most MAX_ITEMS_LIMIT open positions and
// MAX_POSITIONS_LIMIT closed positions.
// For farms, the MAX_ITEMS_LIMIT is the maximum number of farms that can be queried at a given time.
pub const MAX_ITEMS_LIMIT: u32 = 100;
const DEFAULT_LIMIT: u32 = 10;

/// Gets the farms in the contract
pub fn get_farms(
    storage: &dyn Storage,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<Farm>, ContractError> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_ITEMS_LIMIT) as usize;
    let start = cw_utils::calc_range_start_string(start_after).map(Bound::ExclusiveRaw);

    FARMS
        .range(storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, farm) = item?;

            Ok(farm)
        })
        .collect()
}

/// Gets farms given an lp denom.
pub fn get_farms_by_lp_denom(
    storage: &dyn Storage,
    lp_denom: &str,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<Farm>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_ITEMS_LIMIT) as usize;
    let start = cw_utils::calc_range_start_string(start_after).map(Bound::ExclusiveRaw);

    FARMS
        .idx
        .lp_denom
        .prefix(lp_denom.to_owned())
        .range(storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, farm) = item?;

            Ok(farm)
        })
        .collect()
}

/// Gets all the farms that are offering the given [farm_asset] as a reward.
pub fn get_farms_by_farm_asset(
    storage: &dyn Storage,
    farm_asset: &str,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<Farm>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_ITEMS_LIMIT) as usize;
    let start = cw_utils::calc_range_start_string(start_after).map(Bound::ExclusiveRaw);

    FARMS
        .idx
        .farm_asset
        .prefix(farm_asset.to_owned())
        .range(storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, farm) = item?;

            Ok(farm)
        })
        .collect()
}

/// Gets the farm given its identifier
pub fn get_farm_by_identifier(
    storage: &dyn Storage,
    farm_identifier: &String,
) -> Result<Farm, ContractError> {
    FARMS
        .may_load(storage, farm_identifier)?
        .ok_or(ContractError::NonExistentFarm)
}

/// Gets positions
pub(crate) fn get_positions(
    storage: &dyn Storage,
    start_after: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<Position>, ContractError> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_ITEMS_LIMIT) as usize;
    let start = cw_utils::calc_range_start_string(start_after).map(Bound::ExclusiveRaw);

    POSITIONS
        .range(storage, start, None, Order::Ascending)
        .take(limit)
        .map(|item| {
            let (_, farm) = item?;

            Ok(farm)
        })
        .collect()
}

/// Gets a position given its identifier. If the position is not found with the given identifier, it returns None.
pub fn get_position(
    storage: &dyn Storage,
    identifier: Option<String>,
) -> StdResult<Option<Position>> {
    if let Some(identifier) = identifier {
        // there is a position
        POSITIONS.may_load(storage, &identifier)
    } else {
        // there is no position
        Ok(None)
    }
}

/// Gets all the positions of the given receiver.
pub fn get_positions_by_receiver(
    storage: &dyn Storage,
    receiver: &str,
    open_state: Option<bool>,
    start_after: Option<String>,
    limit: Option<u32>,
) -> StdResult<Vec<Position>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_ITEMS_LIMIT) as usize;
    let start = cw_utils::calc_range_start_string(start_after).map(Bound::ExclusiveRaw);

    let index = if let Some(open_state) = open_state {
        // if open_state is provided, filter by open state
        POSITIONS
            .idx
            .open_state_by_receiver
            .prefix((receiver.as_bytes().to_vec(), open_state.into()))
    } else {
        // otherwise get all positions, no matter if they are open or closed
        POSITIONS.idx.receiver.prefix(receiver.to_string())
    };

    let positions_by_receiver = index
        .range(storage, start, None, Order::Ascending)
        // take only the first `limit` positions. If filtering by open state, it means the user
        // at most have MAX_POSITION_LIMIT open and MAX_POSITION_LIMIT close positions, as they
        // are validated when creating/closing a position.
        .take(limit)
        .map(|item| {
            let (_, position) = item?;
            Ok(position)
        })
        .collect::<StdResult<Vec<Position>>>()?;

    Ok(positions_by_receiver)
}

/// Gets the earliest entry of an address in the address lp weight history.
/// If the address has no open positions, it returns an error.
pub fn get_earliest_address_lp_weight(
    storage: &dyn Storage,
    address: &Addr,
    lp_denom: &str,
) -> Result<(EpochId, Uint128), ContractError> {
    let earliest_weight_history_result = LP_WEIGHT_HISTORY
        .prefix((address, lp_denom))
        .range(storage, None, None, Order::Ascending)
        .next()
        .transpose();

    match earliest_weight_history_result {
        Ok(Some(item)) => Ok(item),
        Ok(None) => Err(ContractError::NoOpenPositions),
        Err(std_err) => Err(std_err.into()),
    }
}

/// Checks if a user has any LP weight for the given LP denom.
pub fn has_any_lp_weight(
    storage: &dyn Storage,
    address: &Addr,
    lp_denom: &str,
) -> Result<bool, ContractError> {
    let lp_weight_history_result = LP_WEIGHT_HISTORY
        .prefix((address, lp_denom))
        .range(storage, None, None, Order::Ascending)
        .next()
        .transpose();

    match lp_weight_history_result {
        Ok(Some(_)) => Ok(true),
        Ok(None) => Ok(false),
        Err(std_err) => Err(std_err.into()),
    }
}

/// Gets the latest entry of an address in the address lp weight history.
/// If the address has no open positions, returns 0 for the weight.
pub fn get_latest_address_lp_weight(
    storage: &dyn Storage,
    address: &Addr,
    lp_denom: &str,
    epoch_id: &EpochId,
) -> Result<(EpochId, Uint128), ContractError> {
    let latest_weight_history_result = LP_WEIGHT_HISTORY
        .prefix((address, lp_denom))
        .range(storage, None, None, Order::Descending)
        .next()
        .transpose();

    match latest_weight_history_result {
        Ok(Some(item)) => Ok(item),
        Ok(None) => Ok((epoch_id.to_owned(), Uint128::zero())),
        Err(std_err) => Err(std_err.into()),
    }
}
