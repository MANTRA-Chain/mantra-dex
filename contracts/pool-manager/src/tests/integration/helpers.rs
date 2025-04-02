use cosmwasm_std::Coin;
use std::cell::RefCell;
use std::str::FromStr;

pub fn extract_pool_reserves(
    attribute: &cosmwasm_std::Attribute,
    expected_pool_reserves: &RefCell<Vec<Vec<Coin>>>,
) {
    let mut pool_reserves = vec![];
    for reserve in attribute.value.split(",") {
        pool_reserves.push(Coin::from_str(reserve).unwrap());
    }
    pool_reserves.sort_by(|a, b| a.denom.cmp(&b.denom));
    expected_pool_reserves.borrow_mut().push(pool_reserves);
}
