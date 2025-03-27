use crate::state::POOLS;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Coin, DepsMut, Order, StdError};
use cw_storage_plus::{Index, IndexList, IndexedMap, UniqueIndex};
use mantra_dex_std::fee::PoolFee;
use mantra_dex_std::pool_manager::{PoolInfo, PoolStatus, PoolType};

/// Migrates to v1.3.0, which adds status to the PoolInfo struct
pub fn migrate_to_v130(deps: DepsMut) -> Result<(), StdError> {
    // recreate the old structure
    #[cw_serde]
    struct OldPoolInfo {
        /// The identifier for the pool.
        pub pool_identifier: String,
        /// The asset denoms for the pool.
        pub asset_denoms: Vec<String>,
        /// The LP denom of the pool.
        pub lp_denom: String,
        /// The decimals for the given asset denoms, provided in the same order as asset_denoms.
        pub asset_decimals: Vec<u8>,
        /// The total amount of assets in the pool.
        pub assets: Vec<Coin>,
        /// The type of pool to create.
        pub pool_type: PoolType,
        /// The fees for the pool.
        pub pool_fees: PoolFee,
    }

    struct OldPoolIndexes<'a> {
        pub lp_asset: UniqueIndex<'a, String, OldPoolInfo, String>,
    }

    impl IndexList<OldPoolInfo> for OldPoolIndexes<'_> {
        fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<OldPoolInfo>> + '_> {
            let v: Vec<&dyn Index<OldPoolInfo>> = vec![&self.lp_asset];
            Box::new(v.into_iter())
        }
    }

    const OLD_POOLS: IndexedMap<&str, OldPoolInfo, OldPoolIndexes> = IndexedMap::new(
        "pools",
        OldPoolIndexes {
            lp_asset: UniqueIndex::new(|v| v.lp_denom.to_string(), "pools__lp_asset"),
        },
    );

    let old_values = OLD_POOLS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<Result<Vec<_>, _>>()?;

    old_values
        .into_iter()
        .try_for_each(|(key, old_pool_info)| -> Result<(), StdError> {
            let pool_status = if old_pool_info.pool_identifier == *"o.ausdy.uusdc" {
                // disable the broken o.ausdy.uusdc pool
                PoolStatus {
                    swaps_enabled: false,
                    deposits_enabled: false,
                    withdrawals_enabled: true,
                }
            } else {
                PoolStatus::default()
            };

            OLD_POOLS.remove(deps.storage, &key)?;

            POOLS.save(
                deps.storage,
                &key,
                &PoolInfo {
                    pool_identifier: old_pool_info.pool_identifier,
                    asset_denoms: old_pool_info.asset_denoms,
                    lp_denom: old_pool_info.lp_denom,
                    asset_decimals: old_pool_info.asset_decimals,
                    assets: old_pool_info.assets,
                    pool_type: old_pool_info.pool_type,
                    pool_fees: old_pool_info.pool_fees,
                    status: pool_status,
                },
            )?;

            Ok(())
        })?;

    Ok(())
}
