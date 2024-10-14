use std::collections::HashMap;
use std::fmt::Display;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal, Uint128};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

/// The instantiation message
#[cw_serde]
pub struct InstantiateMsg {
    /// The owner of the contract
    pub owner: String,
    /// The epoch manager address, where the epochs are managed
    pub epoch_manager_addr: String,
    /// The fee collector address, where protocol fees are stored
    pub fee_collector_addr: String,
    /// The pool manager address, where pools are created
    pub pool_manager_addr: String,
    /// The fee that must be paid to create a farm.
    pub create_farm_fee: Coin,
    /// The maximum amount of farms that can exist for a single LP token at a time.
    pub max_concurrent_farms: u32,
    /// New farms are allowed to start up to `current_epoch + start_epoch_buffer` into the future.
    pub max_farm_epoch_buffer: u32,
    /// The minimum amount of time that a user can lock their tokens for. In seconds.
    pub min_unlocking_duration: u64,
    /// The maximum amount of time that a user can lock their tokens for. In seconds.
    pub max_unlocking_duration: u64,
    /// The amount of time after which a farm is considered to be expired after it ended. In seconds.
    /// Once a farm is expired it cannot be expanded, and expired farms can be closed
    pub farm_expiration_time: u64,
    /// The penalty for unlocking a position before the unlocking duration finishes. In percentage.
    pub emergency_unlock_penalty: Decimal,
}

/// The execution messages
#[cw_ownable_execute]
#[cw_serde]
pub enum ExecuteMsg {
    /// Manages a farm based on the action, which can be:
    /// - Fill: Creates or expands a farm.
    /// - Close: Closes an existing farm.
    ManageFarm { action: FarmAction },
    /// Manages a position based on the action, which can be:
    /// - Fill: Creates or expands a position.
    /// - Close: Closes an existing position.
    ManagePosition { action: PositionAction },
    /// Claims the rewards for the user
    Claim {},
    /// Updates the config of the contract
    UpdateConfig {
        /// The fee collector address, where protocol fees are stored
        fee_collector_addr: Option<String>,
        /// The epoch manager address, where the epochs are managed
        epoch_manager_addr: Option<String>,
        /// The pool manager address, where pools are created
        pool_manager_addr: Option<String>,
        /// The fee that must be paid to create a farm.
        create_farm_fee: Option<Coin>,
        /// The maximum amount of farms that can exist for a single LP token at a time.
        max_concurrent_farms: Option<u32>,
        /// The maximum amount of epochs in the future a new farm is allowed to start in.
        max_farm_epoch_buffer: Option<u32>,
        /// The minimum amount of time that a user can lock their tokens for. In seconds.
        min_unlocking_duration: Option<u64>,
        /// The maximum amount of time that a user can lock their tokens for. In seconds.
        max_unlocking_duration: Option<u64>,
        /// The amount of time after which a farm is considered to be expired after it ended. In seconds.
        /// Once a farm is expired it cannot be expanded, and expired farms can be closed
        farm_expiration_time: Option<u64>,
        /// The penalty for unlocking a position before the unlocking duration finishes. In percentage.
        emergency_unlock_penalty: Option<Decimal>,
    },
}

/// The migrate message
#[cw_serde]
pub struct MigrateMsg {}

/// The query messages
#[cw_ownable_query]
#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    /// Retrieves the configuration of the manager.
    #[returns(Config)]
    Config {},
    /// Retrieves farms in the contract. It is possible to filter by [FarmsBy] and to paginate the results.
    #[returns(FarmsResponse)]
    Farms {
        /// An optional parameter specifying what to filter farms by.
        /// Can be either the farm identifier, lp denom or the farm asset.
        filter_by: Option<FarmsBy>,
        /// An optional parameter specifying what farm (identifier) to start searching after.
        start_after: Option<String>,
        /// The amount of farms to return.
        /// If unspecified, will default to a value specified by the contract.
        limit: Option<u32>,
    },
    /// Retrieves the positions for an address.
    #[returns(PositionsResponse)]
    Positions {
        /// An optional parameter specifying what to filter positions by.
        filter_by: Option<PositionsBy>,
        /// An optional parameter specifying to return only positions that match the given open state.
        /// if true, it will return open positions. If false, it will return closed positions.
        open_state: Option<bool>,
        /// An optional parameter specifying what position (identifier) to start searching after.
        start_after: Option<String>,
        /// The amount of positions to return.
        /// If unspecified, will default to a value specified by the contract.
        limit: Option<u32>,
    },
    /// Retrieves the rewards for an address.
    #[returns(RewardsResponse)]
    Rewards {
        /// The address to get all the farm rewards for.
        address: String,
    },
    /// Retrieves the total LP weight in the contract for a given denom on a given epoch.
    #[returns(LpWeightResponse)]
    LpWeight {
        /// The address to get the LP weight for.
        address: String,
        /// The denom to get the total LP weight for.
        denom: String,
        /// The epoch id to get the LP weight for.
        epoch_id: EpochId,
    },
}

/// Enum to filter farms by identifier, lp denom or the farm asset. Used in the farms query.
#[cw_serde]
pub enum FarmsBy {
    Identifier(String),
    LpDenom(String),
    FarmAsset(String),
}

/// Enum to filter positions by identifier or receiver. Used in the positions query.
#[cw_serde]
pub enum PositionsBy {
    Identifier(String),
    Receiver(String),
}

/// Configuration for the contract (manager)
#[cw_serde]
pub struct Config {
    /// The fee collector address, where protocol fees are stored
    pub fee_collector_addr: Addr,
    /// The epoch manager address, where the epochs are managed
    pub epoch_manager_addr: Addr,
    /// The pool manager address, where pools are created
    pub pool_manager_addr: Addr,
    /// The fee that must be paid to create a farm.
    pub create_farm_fee: Coin,
    /// The maximum amount of farms that can exist for a single LP token at a time.
    pub max_concurrent_farms: u32,
    /// The maximum amount of epochs in the future a new farm is allowed to start in.
    pub max_farm_epoch_buffer: u32,
    /// The minimum amount of time that a user can lock their tokens for. In seconds.
    pub min_unlocking_duration: u64,
    /// The maximum amount of time that a user can lock their tokens for. In seconds.
    pub max_unlocking_duration: u64,
    /// The amount of time after which a farm is considered to be expired after it ended. In seconds.
    /// Once a farm is expired it cannot be expanded, and expired farms can be closed
    pub farm_expiration_time: u64,
    /// The penalty for unlocking a position before the unlocking duration finishes. In percentage.
    pub emergency_unlock_penalty: Decimal,
}

/// Parameters for creating farms
#[cw_serde]
pub struct FarmParams {
    /// The LP asset denom to create the farm for.
    pub lp_denom: String,
    /// The epoch at which the farm will start. If unspecified, it will start at the
    /// current epoch.
    pub start_epoch: Option<u64>,
    /// The epoch at which the farm should preliminarily end (if it's not expanded). If
    /// unspecified, the farm will default to end at 14 epochs from the current one.
    pub preliminary_end_epoch: Option<u64>,
    /// The type of distribution curve. If unspecified, the distribution will be linear.
    pub curve: Option<Curve>,
    /// The asset to be distributed in this farm.
    pub farm_asset: Coin,
    /// If set, it  will be used to identify the farm.
    pub farm_identifier: Option<String>,
}

#[cw_serde]
pub enum FarmAction {
    /// Fills a farm. If the farm doesn't exist, it creates a new one. If it exists already,
    /// it expands it given the sender created the original farm and the params are correct.
    Fill {
        /// The parameters for the farm to fill.
        params: FarmParams,
    },
    //// Closes a farm with the given identifier. If the farm has expired, anyone can
    // close it. Otherwise, only the farm creator or the owner of the contract can close a farm.
    Close {
        /// The farm identifier to close.
        farm_identifier: String,
    },
}

#[cw_serde]
pub enum PositionAction {
    /// Creates a position.
    Create {
        /// The identifier of the position.
        identifier: Option<String>,
        /// The time it takes in seconds to unlock this position. This is used to identify the position to fill.
        unlocking_duration: u64,
        /// The receiver for the position.
        /// If left empty, defaults to the message sender.
        receiver: Option<String>,
    },
    /// Expands a position.
    Expand {
        /// The identifier of the position.
        identifier: String,
    },
    /// Closes an existing position. The position stops earning farm rewards.
    Close {
        /// The identifier of the position.
        identifier: String,
        /// The asset to add to the position. If not set, the position will be closed in full. If not, it could be partially closed.
        lp_asset: Option<Coin>,
    },
    /// Withdraws the LP tokens from a position after the position has been closed and the unlocking duration has passed.
    Withdraw {
        /// The identifier of the position.
        identifier: String,
        /// Whether to unlock the position in an emergency. If set to true, the position will be
        /// unlocked immediately. If the position has not expired, it will pay a penalty.
        emergency_unlock: Option<bool>,
    },
}

// type for the epoch id
pub type EpochId = u64;

/// Represents a farm.
#[cw_serde]
pub struct Farm {
    /// The ID of the farm.
    pub identifier: String,
    /// The account which opened the farm and can manage it.
    pub owner: Addr,
    /// The LP asset denom to create the farm for.
    pub lp_denom: String,
    /// The asset the farm was created to distribute.
    pub farm_asset: Coin,
    /// The amount of the `farm_asset` that has been claimed so far.
    pub claimed_amount: Uint128,
    /// The amount of the `farm_asset` that is to be distributed every epoch.
    pub emission_rate: Uint128,
    /// The type of curve the farm has.
    pub curve: Curve,
    /// The epoch at which the farm starts.
    pub start_epoch: EpochId,
    /// The epoch at which the farm will preliminary end (in case it's not expanded).
    pub preliminary_end_epoch: EpochId,
    /// The last epoch this farm was claimed.
    pub last_epoch_claimed: EpochId,
}

#[cw_serde]
pub enum Curve {
    /// A linear curve that releases assets uniformly over time.
    Linear,
}

impl std::fmt::Display for Curve {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Curve::Linear => write!(f, "linear"),
        }
    }
}

/// Represents an LP position.
#[cw_serde]
pub struct Position {
    /// The identifier of the position.
    pub identifier: String,
    /// The amount of LP tokens that are put up to farm rewards.
    pub lp_asset: Coin,
    /// Represents the amount of time in seconds the user must wait after unlocking for the LP tokens to be released.
    pub unlocking_duration: u64,
    /// If true, the position is open. If false, the position is closed.
    pub open: bool,
    /// The block height at which the position, after being closed, can be withdrawn.
    pub expiring_at: Option<u64>,
    /// The owner of the position.
    pub receiver: Addr,
}

impl Position {
    pub fn is_expired(&self, current_time: u64) -> bool {
        self.expiring_at.is_some() && self.expiring_at.unwrap() <= current_time
    }
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Position: {} - LP Asset: {} - Unlocking Duration: {} - Open: {} - Receiver: {} - Expiring At: {}",
            self.identifier, self.lp_asset, self.unlocking_duration, self.open, self.receiver, self.expiring_at.unwrap_or(u64::MAX)
        )
    }
}

#[cw_serde]
pub enum RewardsResponse {
    /// The rewards response
    RewardsResponse {
        /// The rewards that is available to a user if they executed the `claim` function at this point.
        total_rewards: Vec<Coin>,
        /// The rewards per LP denom that is available to a user if they executed the `claim` function at this point.
        rewards_per_lp_denom: Vec<(String, Vec<Coin>)>,
    },
    /// Rewards response used internally when querying the rewards
    QueryRewardsResponse {
        /// The rewards that is available to a user if they executed the `claim` function at this point.
        rewards: Vec<Coin>,
    },
    /// Returned when claiming rewards
    ClaimRewards {
        /// The rewards that is available to a user if they executed the `claim` function at this point.
        rewards: Vec<Coin>,
        /// The rewards that were claimed on each farm, if any.
        modified_farms: HashMap<String, Uint128>,
    },
}

/// Minimum amount of an asset to create a farm with
pub const MIN_FARM_AMOUNT: Uint128 = Uint128::new(1_000u128);

/// Default farm duration in epochs
pub const DEFAULT_FARM_DURATION: u64 = 14u64;

/// The response for the farms query
#[cw_serde]
pub struct FarmsResponse {
    /// The list of farms
    pub farms: Vec<Farm>,
}

#[cw_serde]
pub struct PositionsResponse {
    /// All the positions a user has.
    pub positions: Vec<Position>,
}

/// The response for the LP weight query
#[cw_serde]
pub struct LpWeightResponse {
    /// The total lp weight in the contract
    pub lp_weight: Uint128,
    /// The epoch id corresponding to the lp weight in the contract
    pub epoch_id: EpochId,
}
