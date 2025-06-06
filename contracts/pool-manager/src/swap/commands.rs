use cosmwasm_std::Decimal;
use cosmwasm_std::{ensure, Addr, BankMsg, CosmosMsg, DepsMut, MessageInfo, Response};

use mantra_dex_std::coin::burn_coin_msg;
use mantra_dex_std::common::validate_addr_or_default;

use crate::state::get_pool_by_identifier;
use crate::{state::CONFIG, ContractError};

use super::perform_swap::perform_swap;

pub fn swap(
    mut deps: DepsMut,
    info: MessageInfo,
    sender: Addr,
    ask_asset_denom: String,
    belief_price: Option<Decimal>,
    max_slippage: Option<Decimal>,
    receiver: Option<String>,
    pool_identifier: String,
) -> Result<Response, ContractError> {
    let pool = get_pool_by_identifier(&deps.as_ref(), &pool_identifier)?;

    // check if the swap feature is enabled
    ensure!(
        pool.status.swaps_enabled,
        ContractError::OperationDisabled("swap".to_string())
    );

    // ensure offer asset is not the same as ask asset
    let offer_asset = cw_utils::one_coin(&info)?;
    ensure!(
        offer_asset.denom != ask_asset_denom,
        ContractError::SameAsset
    );

    // verify that the assets sent match the ones from the pool
    ensure!(
        [ask_asset_denom.clone(), offer_asset.denom.clone()]
            .iter()
            .all(|asset| pool
                .assets
                .iter()
                .any(|pool_asset| pool_asset.denom == *asset)),
        ContractError::AssetMismatch
    );

    let swap_result = perform_swap(
        deps.branch(),
        offer_asset.clone(),
        ask_asset_denom,
        &pool_identifier,
        belief_price,
        max_slippage,
    )?;

    let mut messages: Vec<CosmosMsg> = vec![];

    let receiver = validate_addr_or_default(&deps.as_ref(), receiver, info.sender);

    if !swap_result.return_asset.amount.is_zero() {
        messages.push(CosmosMsg::Bank(BankMsg::Send {
            to_address: receiver.clone().into_string(),
            amount: vec![swap_result.return_asset.clone()],
        }));
    }

    if !swap_result.burn_fee_asset.amount.is_zero() {
        messages.push(burn_coin_msg(swap_result.burn_fee_asset.clone()));
    }

    if !swap_result.protocol_fee_asset.amount.is_zero() {
        let config = CONFIG.load(deps.storage)?;

        messages.push(
            BankMsg::Send {
                to_address: config.fee_collector_addr.to_string(),
                amount: vec![swap_result.protocol_fee_asset.clone()],
            }
            .into(),
        );
    }

    let pool_reserves: String = swap_result
        .pool_info
        .assets
        .iter()
        .map(|coin| format!("{coin}"))
        .collect::<Vec<_>>()
        .join(",");

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "swap".to_string()),
        ("sender", sender.into_string()),
        ("receiver", receiver.into_string()),
        ("offer_denom", offer_asset.denom),
        ("ask_denom", swap_result.return_asset.denom),
        ("offer_amount", offer_asset.amount.to_string()),
        ("return_amount", swap_result.return_asset.amount.to_string()),
        ("slippage_amount", swap_result.slippage_amount.to_string()),
        (
            "swap_fee_amount",
            swap_result.swap_fee_asset.amount.to_string(),
        ),
        (
            "protocol_fee_amount",
            swap_result.protocol_fee_asset.amount.to_string(),
        ),
        (
            "burn_fee_amount",
            swap_result.burn_fee_asset.amount.to_string(),
        ),
        (
            "extra_fees_amount",
            swap_result.extra_fees_asset.amount.to_string(),
        ),
        (
            "swap_type",
            swap_result.pool_info.pool_type.get_label().to_string(),
        ),
        ("pool_identifier", pool_identifier),
        ("pool_reserves", pool_reserves),
    ]))
}
