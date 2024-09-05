use cosmwasm_std::{Addr, Deps, StdError, StdResult, Storage};
use cw_storage_plus::Item;

/// Validates that the given address matches the address stored in the given `owner_item`.
pub fn validate_owner(
    storage: &dyn Storage,
    owner_item: Item<Addr>,
    address: Addr,
) -> StdResult<()> {
    let owner = owner_item.load(storage)?;

    // verify owner
    if owner != address {
        return Err(StdError::generic_err("Unauthorized"));
    }

    Ok(())
}

/// Validates a [String] address or returns the default address if the validation fails.
pub fn validate_addr_or_default(deps: &Deps, unvalidated: Option<String>, default: Addr) -> Addr {
    unvalidated
        .map_or_else(
            || Some(default.clone()),
            |recv| match deps.api.addr_validate(&recv) {
                Ok(validated) => Some(validated),
                Err(_) => None,
            },
        )
        .unwrap_or(default)
}
