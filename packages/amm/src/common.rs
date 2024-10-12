use cosmwasm_std::{Addr, Deps};

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
