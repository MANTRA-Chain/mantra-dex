use amm::pool_manager::{
    Config, FeatureToggle, PoolsResponse, ReverseSimulateSwapOperationsResponse,
    ReverseSimulationResponse, SimulateSwapOperationsResponse, SimulationResponse, SwapOperation,
};
use amm::pool_manager::{InstantiateMsg, PoolType};
use cosmwasm_std::testing::MockStorage;
use std::cell::RefCell;

use cosmwasm_std::{coin, Addr, Coin, Decimal, Empty, StdResult, Timestamp, Uint128, Uint64};
use cw_multi_test::{
    App, AppBuilder, AppResponse, BankKeeper, Contract, ContractWrapper, DistributionKeeper,
    Executor, FailingModule, GovFailingModule, IbcFailingModule, MockApiBech32, StakeKeeper,
    WasmKeeper,
};

use amm::constants::{LP_SYMBOL, MONTH_IN_SECONDS};
use amm::epoch_manager::EpochConfig;
use amm::farm_manager::PositionsResponse;
use amm::fee::PoolFee;
use common_testing::multi_test::stargate_mock::StargateMock;

/// Creates the pool manager contract
fn contract_pool_manager() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new_with_empty(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_reply(crate::contract::reply);

    Box::new(contract)
}

/// Creates the fee collector contract
pub fn fee_collector_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        fee_collector::contract::execute,
        fee_collector::contract::instantiate,
        fee_collector::contract::query,
    )
    .with_migrate(fee_collector::contract::migrate);

    Box::new(contract)
}

/// Creates the epoch manager contract
pub fn epoch_manager_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        epoch_manager::contract::execute,
        epoch_manager::contract::instantiate,
        epoch_manager::contract::query,
    )
    .with_migrate(epoch_manager::contract::migrate);

    Box::new(contract)
}

/// Creates the farm manager contract
pub fn farm_manager_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        farm_manager::contract::execute,
        farm_manager::contract::instantiate,
        farm_manager::contract::query,
    )
    .with_migrate(farm_manager::contract::migrate);

    Box::new(contract)
}

type OsmosisTokenFactoryApp = App<
    BankKeeper,
    MockApiBech32,
    MockStorage,
    FailingModule<Empty, Empty, Empty>,
    WasmKeeper<Empty, Empty>,
    StakeKeeper,
    DistributionKeeper,
    IbcFailingModule,
    GovFailingModule,
    StargateMock,
>;

pub struct TestingSuite {
    app: OsmosisTokenFactoryApp,
    pub senders: [Addr; 4],
    pub fee_collector_addr: Addr,
    pub pool_manager_addr: Addr,
    pub farm_manager_addr: Addr,
    pub epoch_manager_addr: Addr,
}

/// TestingSuite helpers
impl TestingSuite {
    pub(crate) fn creator(&mut self) -> Addr {
        self.senders.first().unwrap().clone()
    }

    pub(crate) fn set_time(&mut self, timestamp: Timestamp) -> &mut Self {
        let mut block_info = self.app.block_info();
        block_info.time = timestamp;
        self.app.set_block(block_info);

        self
    }
    pub(crate) fn add_one_day(&mut self) -> &mut Self {
        let mut block_info = self.app.block_info();
        block_info.time = block_info.time.plus_days(1);
        self.app.set_block(block_info);

        self
    }

    pub(crate) fn add_one_epoch(&mut self) -> &mut Self {
        self.add_one_day();
        self
    }

    pub(crate) fn get_lp_denom(&self, pool_identifier: String) -> String {
        format!(
            "factory/{}/{}.{}",
            self.pool_manager_addr, pool_identifier, LP_SYMBOL
        )
    }
}

/// Instantiate
impl TestingSuite {
    pub(crate) fn default_with_balances(
        initial_balance: Vec<Coin>,
        startgate_mock: StargateMock,
    ) -> Self {
        let sender_1 = Addr::unchecked("mantra15n2dapfyf7mzz70y0srycnduw5skp0s9u9g74e");
        let sender_2 = Addr::unchecked("mantra13cxr0w5tvczvte29r5n0mauejmrg83m4zxj4l2");
        let sender_3 = Addr::unchecked("mantra150qvkpleat9spklzs3mtwdxszjpeyjcssce49d");
        let sender_4 = Addr::unchecked("mantra15dzl255vgd8t4y2jdjkeyrjqjygv446nr58ltm");

        let bank = BankKeeper::new();

        let balances = vec![
            (sender_1.clone(), initial_balance.clone()),
            (sender_2.clone(), initial_balance.clone()),
            (sender_3.clone(), initial_balance.clone()),
            (sender_4.clone(), initial_balance.clone()),
        ];

        let app = AppBuilder::new()
            .with_api(MockApiBech32::new("mantra"))
            .with_wasm(WasmKeeper::default())
            .with_bank(bank)
            .with_stargate(startgate_mock)
            .build(|router, _api, storage| {
                balances.into_iter().for_each(|(account, amount)| {
                    router.bank.init_balance(storage, &account, amount).unwrap()
                });
            });

        Self {
            app,
            senders: [sender_1, sender_2, sender_3, sender_4],
            fee_collector_addr: Addr::unchecked(""),
            pool_manager_addr: Addr::unchecked(""),
            farm_manager_addr: Addr::unchecked(""),
            epoch_manager_addr: Addr::unchecked(""),
        }
    }

    #[track_caller]
    pub(crate) fn instantiate(
        &mut self,
        fee_collector_addr: String,
        farm_manager_addr: String,
    ) -> &mut Self {
        let msg = InstantiateMsg {
            fee_collector_addr,
            farm_manager_addr,
            pool_creation_fee: coin(1_000, "uusd"),
        };

        let pool_manager_id = self.app.store_code(contract_pool_manager());

        let creator = self.creator().clone();

        self.pool_manager_addr = self
            .app
            .instantiate_contract(
                pool_manager_id,
                creator.clone(),
                &msg,
                &[],
                "mock pool manager",
                Some(creator.clone().into_string()),
            )
            .unwrap();

        self
    }

    #[track_caller]
    pub(crate) fn instantiate_default(&mut self) -> &mut Self {
        self.create_epoch_manager();
        self.create_fee_collector();
        self.create_farm_manager();

        // 25 April 2024 15:00:00 UTC
        let timestamp = Timestamp::from_seconds(1_714_057_200);
        self.set_time(timestamp);

        let creator = self.creator().clone();

        self.instantiate(
            self.fee_collector_addr.to_string(),
            self.farm_manager_addr.to_string(),
        );

        self.update_farm_manager_config(&creator, self.pool_manager_addr.clone(), |res| {
            assert!(res.is_ok());
        })
    }

    #[track_caller]
    fn create_fee_collector(&mut self) {
        let fee_collector_contract = self.app.store_code(fee_collector_contract());

        // create fee collector
        let msg = amm::fee_collector::InstantiateMsg {};

        let creator = self.creator().clone();

        self.fee_collector_addr = self
            .app
            .instantiate_contract(
                fee_collector_contract,
                creator.clone(),
                &msg,
                &[],
                "Fee Collector".to_string(),
                Some(creator.to_string()),
            )
            .unwrap();
    }

    fn create_epoch_manager(&mut self) {
        let epoch_manager_id = self.app.store_code(epoch_manager_contract());

        let creator = self.creator().clone();

        let msg = amm::epoch_manager::InstantiateMsg {
            owner: creator.to_string(),
            epoch_config: EpochConfig {
                duration: Uint64::new(86_400),
                genesis_epoch: Uint64::new(1_714_057_200),
            },
        };

        let creator = self.creator().clone();

        self.epoch_manager_addr = self
            .app
            .instantiate_contract(
                epoch_manager_id,
                creator.clone(),
                &msg,
                &[],
                "Epoch Manager".to_string(),
                Some(creator.to_string()),
            )
            .unwrap();
    }

    fn create_farm_manager(&mut self) {
        let farm_manager_id = self.app.store_code(farm_manager_contract());

        let creator = self.creator().clone();
        let epoch_manager_addr = self.epoch_manager_addr.to_string();
        let fee_collector_addr = self.fee_collector_addr.to_string();

        let msg = amm::farm_manager::InstantiateMsg {
            owner: creator.clone().to_string(),
            epoch_manager_addr,
            fee_collector_addr,
            pool_manager_addr: "".to_string(),
            create_farm_fee: Coin {
                denom: "uwhale".to_string(),
                amount: Uint128::zero(),
            },
            max_concurrent_farms: 5,
            max_farm_epoch_buffer: 014,
            min_unlocking_duration: 86_400,
            max_unlocking_duration: 31_536_000,
            farm_expiration_time: MONTH_IN_SECONDS,
            emergency_unlock_penalty: Decimal::percent(10),
        };

        self.farm_manager_addr = self
            .app
            .instantiate_contract(
                farm_manager_id,
                creator.clone(),
                &msg,
                &[],
                "Farm Manager".to_string(),
                Some(creator.to_string()),
            )
            .unwrap();
    }
}

/// execute messages
impl TestingSuite {
    #[track_caller]
    pub(crate) fn update_ownership(
        &mut self,
        sender: &Addr,
        action: cw_ownable::Action,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        let msg = amm::pool_manager::ExecuteMsg::UpdateOwnership(action);

        result(self.app.execute_contract(
            sender.clone(),
            self.pool_manager_addr.clone(),
            &msg,
            &[],
        ));

        self
    }

    #[track_caller]
    pub(crate) fn provide_liquidity(
        &mut self,
        sender: &Addr,
        pool_identifier: String,
        unlocking_duration: Option<u64>,
        lock_position_identifier: Option<String>,
        max_spread: Option<Decimal>,
        receiver: Option<String>,
        funds: Vec<Coin>,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        let msg = amm::pool_manager::ExecuteMsg::ProvideLiquidity {
            pool_identifier,
            slippage_tolerance: None,
            max_spread,
            receiver,
            unlocking_duration,
            lock_position_identifier,
        };

        result(self.app.execute_contract(
            sender.clone(),
            self.pool_manager_addr.clone(),
            &msg,
            &funds,
        ));

        self
    }

    #[track_caller]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn swap(
        &mut self,
        sender: &Addr,
        ask_asset_denom: String,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
        receiver: Option<String>,
        pool_identifier: String,
        funds: Vec<Coin>,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        let msg = amm::pool_manager::ExecuteMsg::Swap {
            ask_asset_denom,
            belief_price,
            max_spread,
            receiver,
            pool_identifier,
        };

        result(self.app.execute_contract(
            sender.clone(),
            self.pool_manager_addr.clone(),
            &msg,
            &funds,
        ));

        self
    }

    #[track_caller]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn execute_swap_operations(
        &mut self,
        sender: &Addr,
        operations: Vec<SwapOperation>,
        minimum_receive: Option<Uint128>,
        receiver: Option<String>,
        max_spread: Option<Decimal>,
        funds: Vec<Coin>,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        let msg = amm::pool_manager::ExecuteMsg::ExecuteSwapOperations {
            operations,
            minimum_receive,
            receiver,
            max_spread,
        };

        result(self.app.execute_contract(
            sender.clone(),
            self.pool_manager_addr.clone(),
            &msg,
            &funds,
        ));

        self
    }

    #[track_caller]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn create_pool(
        &mut self,
        sender: &Addr,
        asset_denoms: Vec<String>,
        asset_decimals: Vec<u8>,
        pool_fees: PoolFee,
        pool_type: PoolType,
        pool_identifier: Option<String>,
        pool_creation_fee_funds: Vec<Coin>,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        let msg = amm::pool_manager::ExecuteMsg::CreatePool {
            asset_denoms,
            asset_decimals,
            pool_fees,
            pool_type,
            pool_identifier,
        };

        result(self.app.execute_contract(
            sender.clone(),
            self.pool_manager_addr.clone(),
            &msg,
            &pool_creation_fee_funds,
        ));

        self
    }

    #[track_caller]
    pub(crate) fn withdraw_liquidity(
        &mut self,
        sender: &Addr,
        pool_identifier: String,
        funds: Vec<Coin>,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        let msg = amm::pool_manager::ExecuteMsg::WithdrawLiquidity { pool_identifier };

        result(self.app.execute_contract(
            sender.clone(),
            self.pool_manager_addr.clone(),
            &msg,
            &funds,
        ));

        self
    }

    /// Updates the configuration of the contract.
    ///
    /// Any parameters which are set to `None` when passed will not update
    /// the current configuration.
    #[track_caller]
    pub(crate) fn update_config(
        &mut self,
        sender: &Addr,
        new_fee_collector_addr: Option<Addr>,
        new_farm_manager_addr: Option<Addr>,
        new_pool_creation_fee: Option<Coin>,
        new_feature_toggle: Option<FeatureToggle>,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        result(self.app.execute_contract(
            sender.clone(),
            self.pool_manager_addr.clone(),
            &amm::pool_manager::ExecuteMsg::UpdateConfig {
                fee_collector_addr: new_fee_collector_addr.map(|addr| addr.to_string()),
                farm_manager_addr: new_farm_manager_addr.map(|addr| addr.to_string()),
                pool_creation_fee: new_pool_creation_fee,
                feature_toggle: new_feature_toggle,
            },
            &[],
        ));

        self
    }

    /// Updates the configuration of the farm manager contract.
    ///
    /// Any parameters which are set to `None` when passed will not update
    /// the current configuration.
    #[track_caller]
    pub(crate) fn update_farm_manager_config(
        &mut self,
        sender: &Addr,
        new_pool_manager_addr: Addr,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        result(self.app.execute_contract(
            sender.clone(),
            self.farm_manager_addr.clone(),
            &amm::farm_manager::ExecuteMsg::UpdateConfig {
                fee_collector_addr: None,
                epoch_manager_addr: None,
                pool_manager_addr: Some(new_pool_manager_addr.to_string()),
                create_farm_fee: None,
                max_concurrent_farms: None,
                max_farm_epoch_buffer: None,
                min_unlocking_duration: None,
                max_unlocking_duration: None,
                farm_expiration_time: None,
                emergency_unlock_penalty: None,
            },
            &[],
        ));

        self
    }
}

/// queries
impl TestingSuite {
    pub(crate) fn query_ownership(
        &mut self,
        result: impl Fn(StdResult<cw_ownable::Ownership<String>>),
    ) -> &mut Self {
        let ownership_response: StdResult<cw_ownable::Ownership<String>> =
            self.app.wrap().query_wasm_smart(
                &self.pool_manager_addr,
                &amm::pool_manager::QueryMsg::Ownership {},
            );

        result(ownership_response);

        self
    }

    pub(crate) fn query_balance(
        &mut self,
        addr: &String,
        denom: impl Into<String>,
        result: impl Fn(StdResult<Coin>),
    ) -> &mut Self {
        let balance_resp: StdResult<Coin> = self.app.wrap().query_balance(addr, denom);

        result(balance_resp);

        self
    }

    pub(crate) fn query_all_balances(
        &mut self,
        addr: &String,
        result: impl Fn(StdResult<Vec<Coin>>),
    ) -> &mut Self {
        let balance_resp: StdResult<Vec<Coin>> = self.app.wrap().query_all_balances(addr);

        result(balance_resp);

        self
    }

    pub(crate) fn query_pools(
        &self,
        pool_identifier: Option<String>,
        start_after: Option<String>,
        limit: Option<u32>,
        result: impl Fn(StdResult<PoolsResponse>),
    ) -> &Self {
        let pools_response: StdResult<PoolsResponse> = self.app.wrap().query_wasm_smart(
            &self.pool_manager_addr,
            &amm::pool_manager::QueryMsg::Pools {
                pool_identifier,
                start_after,
                limit,
            },
        );

        result(pools_response);

        self
    }

    pub(crate) fn query_simulation(
        &mut self,
        pool_identifier: String,
        offer_asset: Coin,
        ask_asset_denom: String,
        result: impl Fn(StdResult<SimulationResponse>),
    ) -> &mut Self {
        let pool_info_response: StdResult<SimulationResponse> = self.app.wrap().query_wasm_smart(
            &self.pool_manager_addr,
            &amm::pool_manager::QueryMsg::Simulation {
                offer_asset,
                ask_asset_denom,
                pool_identifier,
            },
        );

        result(pool_info_response);

        self
    }

    pub(crate) fn query_reverse_simulation(
        &mut self,
        pool_identifier: String,
        ask_asset: Coin,
        offer_asset_denom: String,
        result: impl Fn(StdResult<ReverseSimulationResponse>),
    ) -> &mut Self {
        let pool_info_response: StdResult<ReverseSimulationResponse> =
            self.app.wrap().query_wasm_smart(
                &self.pool_manager_addr,
                &amm::pool_manager::QueryMsg::ReverseSimulation {
                    ask_asset,
                    offer_asset_denom,
                    pool_identifier,
                },
            );

        result(pool_info_response);

        self
    }

    pub(crate) fn query_simulate_swap_operations(
        &mut self,
        offer_amount: Uint128,
        operations: Vec<SwapOperation>,
        result: impl Fn(StdResult<SimulateSwapOperationsResponse>),
    ) -> &mut Self {
        let pool_info_response: StdResult<SimulateSwapOperationsResponse> =
            self.app.wrap().query_wasm_smart(
                &self.pool_manager_addr,
                &amm::pool_manager::QueryMsg::SimulateSwapOperations {
                    offer_amount,
                    operations,
                },
            );

        result(pool_info_response);

        self
    }

    pub(crate) fn query_reverse_simulate_swap_operations(
        &mut self,
        ask_amount: Uint128,
        operations: Vec<SwapOperation>,
        result: impl Fn(StdResult<ReverseSimulateSwapOperationsResponse>),
    ) -> &mut Self {
        let pool_info_response: StdResult<ReverseSimulateSwapOperationsResponse> =
            self.app.wrap().query_wasm_smart(
                &self.pool_manager_addr,
                &amm::pool_manager::QueryMsg::ReverseSimulateSwapOperations {
                    ask_amount,
                    operations,
                },
            );

        result(pool_info_response);

        self
    }

    pub(crate) fn query_amount_of_lp_token(
        &mut self,
        identifier: String,
        sender: &String,
        result: impl Fn(StdResult<Uint128>),
    ) -> &mut Self {
        // Get the LP token from Config
        let lp_token_response: PoolsResponse = self
            .app
            .wrap()
            .query_wasm_smart(
                &self.pool_manager_addr,
                &amm::pool_manager::QueryMsg::Pools {
                    pool_identifier: Some(identifier),
                    start_after: None,
                    limit: None,
                },
            )
            .unwrap();

        // Get balance of LP token, if native we can just query balance otherwise we need to go to cw20

        let balance: Uint128 = self
            .app
            .wrap()
            .query_balance(sender, &lp_token_response.pools[0].pool_info.lp_denom)
            .unwrap()
            .amount;

        result(Result::Ok(balance));
        self
    }

    /// Retrieves the current configuration of the pool manager contract.
    pub(crate) fn query_config(&mut self) -> Config {
        self.app
            .wrap()
            .query_wasm_smart(
                &self.pool_manager_addr,
                &amm::pool_manager::QueryMsg::Config {},
            )
            .unwrap()
    }

    #[track_caller]
    pub(crate) fn query_farm_positions(
        &mut self,
        filter_by: Option<amm::farm_manager::PositionsBy>,
        open_state: Option<bool>,
        start_after: Option<String>,
        limit: Option<u32>,
        result: impl Fn(StdResult<PositionsResponse>),
    ) -> &mut Self {
        let positions_response: StdResult<PositionsResponse> = self.app.wrap().query_wasm_smart(
            &self.farm_manager_addr,
            &amm::farm_manager::QueryMsg::Positions {
                filter_by,
                open_state,
                start_after,
                limit,
            },
        );

        result(positions_response);

        self
    }

    #[track_caller]
    pub(crate) fn query_lp_supply(
        &mut self,
        identifier: String,
        result: impl Fn(StdResult<Coin>),
    ) -> &mut Self {
        let lp_denom = RefCell::new("".to_string());

        self.query_pools(Some(identifier.clone()), None, None, |res| {
            let response = res.unwrap();
            *lp_denom.borrow_mut() = response.pools[0].pool_info.lp_denom.clone();
        });

        let supply_response = self.app.wrap().query_supply(lp_denom.into_inner());

        result(supply_response);

        self
    }
}
