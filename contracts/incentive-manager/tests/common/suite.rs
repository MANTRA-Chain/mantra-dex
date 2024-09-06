use cosmwasm_std::testing::MockStorage;
use cosmwasm_std::{coin, Addr, Coin, Decimal, Empty, StdResult, Timestamp, Uint128, Uint64};
use cw_multi_test::addons::{MockAddressGenerator, MockApiBech32};
use cw_multi_test::{
    App, AppBuilder, AppResponse, BankKeeper, DistributionKeeper, Executor, FailingModule,
    GovFailingModule, IbcFailingModule, StakeKeeper, WasmKeeper,
};

use amm::epoch_manager::{Epoch, EpochChangedHookMsg, EpochConfig, EpochResponse};
use amm::incentive_manager::{
    Config, IncentiveAction, IncentivesBy, IncentivesResponse, InstantiateMsg, LpWeightResponse,
    PositionAction, PositionsResponse, RewardsResponse,
};
use common_testing::multi_test::stargate_mock::StargateMock;

use crate::common::suite_contracts::{
    epoch_manager_contract, fee_collector_contract, incentive_manager_contract,
    pool_manager_contract,
};

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
    pub senders: [Addr; 3],
    pub incentive_manager_addr: Addr,
    pub fee_collector_addr: Addr,
    pub pool_manager_addr: Addr,
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
        let creator = self.creator();

        self.add_one_day().create_epoch(&creator, |res| {
            res.unwrap();
        });

        self
    }
}

/// Instantiate
impl TestingSuite {
    pub(crate) fn default_with_balances(initial_balance: Vec<Coin>) -> Self {
        let sender_1 = Addr::unchecked("mantra15n2dapfyf7mzz70y0srycnduw5skp0s9u9g74e");
        let sender_2 = Addr::unchecked("mantra13cxr0w5tvczvte29r5n0mauejmrg83m4zxj4l2");
        let sender_3 = Addr::unchecked("mantra150qvkpleat9spklzs3mtwdxszjpeyjcssce49d");

        let bank = BankKeeper::new();

        let balances = vec![
            (sender_1.clone(), initial_balance.clone()),
            (sender_2.clone(), initial_balance.clone()),
            (sender_3.clone(), initial_balance.clone()),
        ];

        let app = AppBuilder::new()
            .with_api(MockApiBech32::new("mantra"))
            .with_wasm(WasmKeeper::default().with_address_generator(MockAddressGenerator))
            .with_bank(bank)
            .with_stargate(StargateMock {})
            .build(|router, _api, storage| {
                balances.into_iter().for_each(|(account, amount)| {
                    router.bank.init_balance(storage, &account, amount).unwrap()
                });
            });

        Self {
            app,
            senders: [sender_1, sender_2, sender_3],
            incentive_manager_addr: Addr::unchecked(""),
            fee_collector_addr: Addr::unchecked(""),
            pool_manager_addr: Addr::unchecked(""),
            epoch_manager_addr: Addr::unchecked(""),
        }
    }

    #[track_caller]
    pub(crate) fn instantiate_default(&mut self) -> &mut Self {
        self.create_epoch_manager();
        self.create_fee_collector();

        // April 4th 2024 15:00:00 UTC
        let timestamp = Timestamp::from_seconds(1712242800u64);
        self.set_time(timestamp);

        // instantiates the incentive manager contract
        self.instantiate(
            self.fee_collector_addr.to_string(),
            self.epoch_manager_addr.to_string(),
            Coin {
                denom: "uom".to_string(),
                amount: Uint128::new(1_000u128),
            },
            2,
            14,
            86_400,
            31_536_000,
            Decimal::percent(10), //10% penalty
        );

        self.create_pool_manager();

        self
    }

    #[allow(clippy::inconsistent_digit_grouping)]
    fn create_epoch_manager(&mut self) {
        let epoch_manager_contract = self.app.store_code(epoch_manager_contract());

        // create epoch manager
        let msg = amm::epoch_manager::InstantiateMsg {
            start_epoch: Epoch {
                id: 10,
                start_time: Timestamp::from_nanos(1712242800_000000000u64),
            },
            epoch_config: EpochConfig {
                duration: Uint64::new(86400_000000000u64),
                genesis_epoch: Uint64::new(1712242800_000000000u64), // April 4th 2024 15:00:00 UTC
            },
        };

        let creator = self.creator().clone();

        self.epoch_manager_addr = self
            .app
            .instantiate_contract(
                epoch_manager_contract,
                creator.clone(),
                &msg,
                &[],
                "Epoch Manager".to_string(),
                Some(creator.to_string()),
            )
            .unwrap();
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

    #[track_caller]
    fn create_pool_manager(&mut self) {
        let pool_manager_contract = self.app.store_code(pool_manager_contract());

        // create pool manager
        let msg = amm::pool_manager::InstantiateMsg {
            fee_collector_addr: self.fee_collector_addr.to_string(),
            incentive_manager_addr: self.incentive_manager_addr.to_string(),
            pool_creation_fee: Coin {
                denom: "uom".to_string(),
                amount: Uint128::new(1_000u128),
            },
        };

        let creator = self.creator().clone();

        self.pool_manager_addr = self
            .app
            .instantiate_contract(
                pool_manager_contract,
                creator.clone(),
                &msg,
                &[],
                "Pool Manager".to_string(),
                Some(creator.to_string()),
            )
            .unwrap();
    }

    #[track_caller]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn instantiate(
        &mut self,
        fee_collector_addr: String,
        epoch_manager_addr: String,
        create_incentive_fee: Coin,
        max_concurrent_incentives: u32,
        max_incentive_epoch_buffer: u32,
        min_unlocking_duration: u64,
        max_unlocking_duration: u64,
        emergency_unlock_penalty: Decimal,
    ) -> &mut Self {
        let msg = InstantiateMsg {
            owner: self.creator().to_string(),
            epoch_manager_addr,
            fee_collector_addr,
            create_incentive_fee,
            max_concurrent_incentives,
            max_incentive_epoch_buffer,
            min_unlocking_duration,
            max_unlocking_duration,
            emergency_unlock_penalty,
        };

        let incentive_manager_id = self.app.store_code(incentive_manager_contract());

        let creator = self.creator().clone();

        self.incentive_manager_addr = self
            .app
            .instantiate_contract(
                incentive_manager_id,
                creator.clone(),
                &msg,
                &[],
                "Incentive Manager",
                Some(creator.into_string()),
            )
            .unwrap();
        self
    }

    #[track_caller]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn instantiate_err(
        &mut self,
        fee_collector_addr: String,
        epoch_manager_addr: String,
        create_incentive_fee: Coin,
        max_concurrent_incentives: u32,
        max_incentive_epoch_buffer: u32,
        min_unlocking_duration: u64,
        max_unlocking_duration: u64,
        emergency_unlock_penalty: Decimal,
        result: impl Fn(anyhow::Result<Addr>),
    ) -> &mut Self {
        let msg = InstantiateMsg {
            owner: self.creator().to_string(),
            epoch_manager_addr,
            fee_collector_addr,
            create_incentive_fee,
            max_concurrent_incentives,
            max_incentive_epoch_buffer,
            min_unlocking_duration,
            max_unlocking_duration,
            emergency_unlock_penalty,
        };

        let incentive_manager_id = self.app.store_code(incentive_manager_contract());

        let creator = self.creator().clone();

        result(self.app.instantiate_contract(
            incentive_manager_id,
            creator.clone(),
            &msg,
            &[],
            "Incentive Manager",
            Some(creator.into_string()),
        ));

        self
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
        let msg = amm::incentive_manager::ExecuteMsg::UpdateOwnership(action);

        result(self.app.execute_contract(
            sender.clone(),
            self.incentive_manager_addr.clone(),
            &msg,
            &[],
        ));

        self
    }

    #[track_caller]
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update_config(
        &mut self,
        sender: &Addr,
        fee_collector_addr: Option<String>,
        epoch_manager_addr: Option<String>,
        create_incentive_fee: Option<Coin>,
        max_concurrent_incentives: Option<u32>,
        max_incentive_epoch_buffer: Option<u32>,
        min_unlocking_duration: Option<u64>,
        max_unlocking_duration: Option<u64>,
        emergency_unlock_penalty: Option<Decimal>,
        funds: Vec<Coin>,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        let msg = amm::incentive_manager::ExecuteMsg::UpdateConfig {
            fee_collector_addr,
            epoch_manager_addr,
            create_incentive_fee,
            max_concurrent_incentives,
            max_incentive_epoch_buffer,
            min_unlocking_duration,
            max_unlocking_duration,
            emergency_unlock_penalty,
        };

        result(self.app.execute_contract(
            sender.clone(),
            self.incentive_manager_addr.clone(),
            &msg,
            &funds,
        ));

        self
    }

    #[track_caller]
    pub(crate) fn manage_incentive(
        &mut self,
        sender: &Addr,
        action: IncentiveAction,
        funds: Vec<Coin>,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        let msg = amm::incentive_manager::ExecuteMsg::ManageIncentive { action };

        result(self.app.execute_contract(
            sender.clone(),
            self.incentive_manager_addr.clone(),
            &msg,
            &funds,
        ));

        self
    }

    #[track_caller]
    pub(crate) fn manage_position(
        &mut self,
        sender: &Addr,
        action: PositionAction,
        funds: Vec<Coin>,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        let msg = amm::incentive_manager::ExecuteMsg::ManagePosition { action };

        result(self.app.execute_contract(
            sender.clone(),
            self.incentive_manager_addr.clone(),
            &msg,
            &funds,
        ));

        self
    }

    #[track_caller]
    pub(crate) fn claim(
        &mut self,
        sender: &Addr,
        funds: Vec<Coin>,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        let msg = amm::incentive_manager::ExecuteMsg::Claim;

        result(self.app.execute_contract(
            sender.clone(),
            self.incentive_manager_addr.clone(),
            &msg,
            &funds,
        ));

        self
    }

    #[track_caller]
    pub(crate) fn on_epoch_changed(
        &mut self,
        sender: &Addr,
        funds: Vec<Coin>,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        let msg = amm::incentive_manager::ExecuteMsg::EpochChangedHook(EpochChangedHookMsg {
            current_epoch: Epoch {
                id: 0,
                start_time: Default::default(),
            },
        });

        result(self.app.execute_contract(
            sender.clone(),
            self.incentive_manager_addr.clone(),
            &msg,
            &funds,
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
                &self.incentive_manager_addr,
                &amm::incentive_manager::QueryMsg::Ownership {},
            );

        result(ownership_response);

        self
    }

    #[track_caller]
    pub(crate) fn query_config(&mut self, result: impl Fn(StdResult<Config>)) -> &mut Self {
        let response: StdResult<Config> = self.app.wrap().query_wasm_smart(
            &self.incentive_manager_addr,
            &amm::incentive_manager::QueryMsg::Config {},
        );

        result(response);

        self
    }

    #[track_caller]
    pub(crate) fn query_incentives(
        &mut self,
        filter_by: Option<IncentivesBy>,
        start_after: Option<String>,
        limit: Option<u32>,
        result: impl Fn(StdResult<IncentivesResponse>),
    ) -> &mut Self {
        let incentives_response: StdResult<IncentivesResponse> = self.app.wrap().query_wasm_smart(
            &self.incentive_manager_addr,
            &amm::incentive_manager::QueryMsg::Incentives {
                filter_by,
                start_after,
                limit,
            },
        );

        result(incentives_response);

        self
    }

    #[track_caller]
    pub(crate) fn query_positions(
        &mut self,
        address: &Addr,
        open_state: Option<bool>,
        result: impl Fn(StdResult<PositionsResponse>),
    ) -> &mut Self {
        let positions_response: StdResult<PositionsResponse> = self.app.wrap().query_wasm_smart(
            &self.incentive_manager_addr,
            &amm::incentive_manager::QueryMsg::Positions {
                address: address.to_string(),
                open_state,
            },
        );

        result(positions_response);

        self
    }
    #[track_caller]
    pub(crate) fn query_rewards(
        &mut self,
        address: &Addr,
        result: impl Fn(StdResult<RewardsResponse>),
    ) -> &mut Self {
        let rewards_response: StdResult<RewardsResponse> = self.app.wrap().query_wasm_smart(
            &self.incentive_manager_addr,
            &amm::incentive_manager::QueryMsg::Rewards {
                address: address.to_string(),
            },
        );

        result(rewards_response);

        self
    }

    #[track_caller]
    pub(crate) fn query_lp_weight(
        &mut self,
        denom: &str,
        epoch_id: u64,
        result: impl Fn(StdResult<LpWeightResponse>),
    ) -> &mut Self {
        let rewards_response: StdResult<LpWeightResponse> = self.app.wrap().query_wasm_smart(
            &self.incentive_manager_addr,
            &amm::incentive_manager::QueryMsg::LPWeight {
                address: self.incentive_manager_addr.to_string(),
                denom: denom.to_string(),
                epoch_id,
            },
        );

        result(rewards_response);

        self
    }

    #[track_caller]
    pub(crate) fn query_balance(
        &mut self,
        denom: String,
        address: &Addr,
        result: impl Fn(Uint128),
    ) -> &mut Self {
        let balance_response = self.app.wrap().query_balance(address, denom.clone());
        result(balance_response.unwrap_or(coin(0, denom)).amount);

        self
    }
}

/// Epoch manager actions
impl TestingSuite {
    #[track_caller]
    pub(crate) fn create_epoch(
        &mut self,
        sender: &Addr,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        let msg = amm::epoch_manager::ExecuteMsg::CreateEpoch {};

        result(self.app.execute_contract(
            sender.clone(),
            self.epoch_manager_addr.clone(),
            &msg,
            &[],
        ));

        self
    }

    #[track_caller]
    pub(crate) fn add_hook(
        &mut self,
        sender: &Addr,
        contract_addr: &Addr,
        funds: Vec<Coin>,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        let msg = amm::epoch_manager::ExecuteMsg::AddHook {
            contract_addr: contract_addr.to_string(),
        };

        result(self.app.execute_contract(
            sender.clone(),
            self.epoch_manager_addr.clone(),
            &msg,
            &funds,
        ));

        self
    }

    #[track_caller]
    pub(crate) fn query_current_epoch(
        &mut self,
        mut result: impl FnMut(StdResult<EpochResponse>),
    ) -> &mut Self {
        let current_epoch_response: StdResult<EpochResponse> = self.app.wrap().query_wasm_smart(
            &self.epoch_manager_addr,
            &amm::epoch_manager::QueryMsg::CurrentEpoch {},
        );

        result(current_epoch_response);

        self
    }
}
