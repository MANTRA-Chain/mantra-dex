/// Validates the contract version and name
#[macro_export]
macro_rules! validate_contract {
    ($deps:expr, $contract_name:expr, $contract_version:expr) => {{
        let stored_contract_name = cw2::CONTRACT.load($deps.storage)?.contract;
        cosmwasm_std::ensure!(
            stored_contract_name == $contract_name,
            cosmwasm_std::StdError::generic_err("Contract name mismatch")
        );

        let version: semver::Version = $contract_version.parse()?;
        let storage_version: semver::Version =
            cw2::get_contract_version($deps.storage)?.version.parse()?;

        cosmwasm_std::ensure!(
            storage_version < version,
            ContractError::MigrateInvalidVersion {
                current_version: storage_version,
                new_version: version,
            }
        );
    }};
}
