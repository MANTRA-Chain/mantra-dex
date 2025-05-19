extern crate core;

use std::cell::RefCell;

use cosmwasm_std::{coin, coins, Addr, Coin, Decimal, StdResult, Timestamp, Uint128};
use cw_utils::PaymentError;
use farm_manager::state::{MAX_FARMS_LIMIT, MAX_POSITIONS_LIMIT};
use farm_manager::ContractError;
use mantra_dex_std::constants::{LP_SYMBOL, MONTH_IN_SECONDS};
use mantra_dex_std::farm_manager::{
    Config, Curve, Farm, FarmAction, FarmParams, FarmsBy, LpWeightResponse, Position,
    PositionAction, PositionsBy, PositionsResponse, RewardsResponse,
};

use crate::common::suite::TestingSuite;
use crate::common::{MOCK_CONTRACT_ADDR_1, MOCK_CONTRACT_ADDR_2};

mod common;
