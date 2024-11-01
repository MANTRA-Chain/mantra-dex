use amm::constants::MONTH_IN_SECONDS;
use cosmwasm_std::testing::MockStorage;
use cosmwasm_std::{coin, Addr, Coin, Decimal, Empty, StdResult, Timestamp, Uint128, Uint64};
use cw_multi_test::{
    App, AppBuilder, AppResponse, BankKeeper, DistributionKeeper, Executor, FailingModule,
    GovFailingModule, IbcFailingModule, MockApiBech32, StakeKeeper, WasmKeeper,
};

use crate::common::suite_contracts::{
    epoch_manager_contract, farm_manager_contract, fee_collector_contract,
};
use crate::common::MOCK_CONTRACT_ADDR_1;
use amm::epoch_manager::{EpochConfig, EpochResponse};
use amm::farm_manager::{
    Config, FarmAction, FarmsBy, FarmsResponse, InstantiateMsg, LpWeightResponse, PositionAction,
    PositionsResponse, RewardsResponse,
};
use common_testing::multi_test::stargate_mock::StargateMock;

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
    pub farm_manager_addr: Addr,
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

    pub(crate) fn get_time(&mut self, result: impl Fn(Timestamp)) -> &mut Self {
        result(self.app.block_info().time);

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

    pub(crate) fn send_tokens(
        &mut self,
        sender: &Addr,
        receiver: &Addr,
        coins: &[Coin],
    ) -> &mut Self {
        self.app
            .send_tokens(sender.clone(), receiver.clone(), coins)
            .unwrap();
        self
    }
}

/// Instantiate
impl TestingSuite {
    pub(crate) fn default_with_balances(initial_balance: Vec<Coin>) -> Self {
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
            .with_stargate(StargateMock::new("uom".to_string(), "8888".to_string()))
            .build(|router, _api, storage| {
                balances.into_iter().for_each(|(account, amount)| {
                    router.bank.init_balance(storage, &account, amount).unwrap()
                });
            });

        Self {
            app,
            senders: [sender_1, sender_2, sender_3, sender_4],
            farm_manager_addr: Addr::unchecked(""),
            fee_collector_addr: Addr::unchecked(""),
            pool_manager_addr: Addr::unchecked(MOCK_CONTRACT_ADDR_1),
            epoch_manager_addr: Addr::unchecked(""),
        }
    }

    #[track_caller]
    pub(crate) fn instantiate_default(&mut self) -> &mut Self {
        self.create_epoch_manager();
        self.create_fee_collector();

        // April 4th 2024 15:00:00 UTC
        let timestamp = Timestamp::from_seconds(1_712_242_800u64);
        self.set_time(timestamp);

        // instantiates the farm manager contract
        self.instantiate(
            self.fee_collector_addr.to_string(),
            self.epoch_manager_addr.to_string(),
            self.pool_manager_addr.to_string(),
            Coin {
                denom: "uom".to_string(),
                amount: Uint128::new(1_000u128),
            },
            2,
            14,
            86_400,
            31_556_926u64,
            MONTH_IN_SECONDS,
            Decimal::percent(10), //10% penalty
        );

        self
    }

    #[allow(clippy::inconsistent_digit_grouping)]
    fn create_epoch_manager(&mut self) {
        let epoch_manager_contract = self.app.store_code(epoch_manager_contract());
        let creator = self.creator().clone();

        // create epoch manager
        let msg = amm::epoch_manager::InstantiateMsg {
            owner: creator.to_string(),
            epoch_config: EpochConfig {
                duration: Uint64::new(86_400u64),
                genesis_epoch: Uint64::new(1_712_242_800u64), // April 4th 2024 15:00:00 UTC
            },
        };

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
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn instantiate(
        &mut self,
        fee_collector_addr: String,
        epoch_manager_addr: String,
        pool_manager_addr: String,
        create_farm_fee: Coin,
        max_concurrent_farms: u32,
        max_farm_epoch_buffer: u32,
        min_unlocking_duration: u64,
        max_unlocking_duration: u64,
        farm_expiration_time: u64,
        emergency_unlock_penalty: Decimal,
    ) -> &mut Self {
        let msg = InstantiateMsg {
            owner: self.creator().to_string(),
            epoch_manager_addr,
            fee_collector_addr,
            pool_manager_addr,
            create_farm_fee,
            max_concurrent_farms,
            max_farm_epoch_buffer,
            min_unlocking_duration,
            max_unlocking_duration,
            farm_expiration_time,
            emergency_unlock_penalty,
        };

        let farm_manager_id = self.app.store_code(farm_manager_contract());

        let creator = self.creator().clone();

        self.farm_manager_addr = self
            .app
            .instantiate_contract(
                farm_manager_id,
                creator.clone(),
                &msg,
                &[],
                "Farm Manager",
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
        pool_manager_addr: String,
        create_farm_fee: Coin,
        max_concurrent_farms: u32,
        max_farm_epoch_buffer: u32,
        min_unlocking_duration: u64,
        max_unlocking_duration: u64,
        farm_expiration_time: u64,
        emergency_unlock_penalty: Decimal,
        result: impl Fn(anyhow::Result<Addr>),
    ) -> &mut Self {
        let msg = InstantiateMsg {
            owner: self.creator().to_string(),
            epoch_manager_addr,
            fee_collector_addr,
            pool_manager_addr,
            create_farm_fee,
            max_concurrent_farms,
            max_farm_epoch_buffer,
            min_unlocking_duration,
            max_unlocking_duration,
            farm_expiration_time,
            emergency_unlock_penalty,
        };

        let farm_manager_id = self.app.store_code(farm_manager_contract());

        let creator = self.creator().clone();

        result(self.app.instantiate_contract(
            farm_manager_id,
            creator.clone(),
            &msg,
            &[],
            "Farm Manager",
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
        let msg = amm::farm_manager::ExecuteMsg::UpdateOwnership(action);

        result(self.app.execute_contract(
            sender.clone(),
            self.farm_manager_addr.clone(),
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
        pool_manager_addr: Option<String>,
        create_farm_fee: Option<Coin>,
        max_concurrent_farms: Option<u32>,
        max_farm_epoch_buffer: Option<u32>,
        min_unlocking_duration: Option<u64>,
        max_unlocking_duration: Option<u64>,
        farm_expiration_time: Option<u64>,
        emergency_unlock_penalty: Option<Decimal>,
        funds: Vec<Coin>,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        let msg = amm::farm_manager::ExecuteMsg::UpdateConfig {
            fee_collector_addr,
            epoch_manager_addr,
            pool_manager_addr,
            create_farm_fee,
            max_concurrent_farms,
            max_farm_epoch_buffer,
            min_unlocking_duration,
            max_unlocking_duration,
            farm_expiration_time,
            emergency_unlock_penalty,
        };

        result(self.app.execute_contract(
            sender.clone(),
            self.farm_manager_addr.clone(),
            &msg,
            &funds,
        ));

        self
    }

    #[track_caller]
    pub(crate) fn manage_farm(
        &mut self,
        sender: &Addr,
        action: FarmAction,
        funds: Vec<Coin>,
        result: impl Fn(Result<AppResponse, anyhow::Error>),
    ) -> &mut Self {
        let msg = amm::farm_manager::ExecuteMsg::ManageFarm { action };

        result(self.app.execute_contract(
            sender.clone(),
            self.farm_manager_addr.clone(),
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
        let msg = amm::farm_manager::ExecuteMsg::ManagePosition { action };

        result(self.app.execute_contract(
            sender.clone(),
            self.farm_manager_addr.clone(),
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
        let msg = amm::farm_manager::ExecuteMsg::Claim {};

        result(self.app.execute_contract(
            sender.clone(),
            self.farm_manager_addr.clone(),
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
                &self.farm_manager_addr,
                &amm::farm_manager::QueryMsg::Ownership {},
            );

        result(ownership_response);

        self
    }

    #[track_caller]
    pub(crate) fn query_config(&mut self, result: impl Fn(StdResult<Config>)) -> &mut Self {
        let response: StdResult<Config> = self.app.wrap().query_wasm_smart(
            &self.farm_manager_addr,
            &amm::farm_manager::QueryMsg::Config {},
        );

        result(response);

        self
    }

    #[track_caller]
    pub(crate) fn query_farms(
        &mut self,
        filter_by: Option<FarmsBy>,
        start_after: Option<String>,
        limit: Option<u32>,
        result: impl Fn(StdResult<FarmsResponse>),
    ) -> &mut Self {
        let farms_response: StdResult<FarmsResponse> = self.app.wrap().query_wasm_smart(
            &self.farm_manager_addr,
            &amm::farm_manager::QueryMsg::Farms {
                filter_by,
                start_after,
                limit,
            },
        );

        result(farms_response);

        self
    }

    #[track_caller]
    pub(crate) fn query_positions(
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
    pub(crate) fn query_rewards(
        &mut self,
        address: &Addr,
        result: impl Fn(StdResult<RewardsResponse>),
    ) -> &mut Self {
        let rewards_response: StdResult<RewardsResponse> = self.app.wrap().query_wasm_smart(
            &self.farm_manager_addr,
            &amm::farm_manager::QueryMsg::Rewards {
                address: address.to_string(),
            },
        );

        result(rewards_response);

        self
    }

    #[track_caller]
    pub(crate) fn query_lp_weight(
        &mut self,
        address: &Addr,
        denom: &str,
        epoch_id: u64,
        result: impl Fn(StdResult<LpWeightResponse>),
    ) -> &mut Self {
        let rewards_response: StdResult<LpWeightResponse> = self.app.wrap().query_wasm_smart(
            &self.farm_manager_addr,
            &amm::farm_manager::QueryMsg::LpWeight {
                address: address.to_string(),
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
