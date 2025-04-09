use cosmwasm_std::{Decimal256, Uint256, Uint512};

use crate::error::ContractError;

pub trait Decimal256Helper {
    fn decimal_with_precision(
        value: impl Into<Uint256>,
        precision: u8,
    ) -> Result<Decimal256, ContractError>;

    fn checked_multiply_ratio(
        &self,
        numerator: Decimal256,
        denominator: Decimal256,
    ) -> Result<Decimal256, ContractError>;

    fn to_uint256_with_precision(&self, precision: u32) -> Result<Uint256, ContractError>;
    fn to_uint512_with_precision(&self, precision: u32) -> Result<Uint512, ContractError>;
}

impl Decimal256Helper for Decimal256 {
    fn decimal_with_precision(
        value: impl Into<Uint256>,
        precision: u8,
    ) -> Result<Decimal256, ContractError> {
        Decimal256::from_atomics(value, u32::from(precision))
            .map_err(|_| ContractError::DecimalOverflow)
    }

    fn checked_multiply_ratio(
        &self,
        numerator: Decimal256,
        denominator: Decimal256,
    ) -> Result<Decimal256, ContractError> {
        Ok(Decimal256::new(self.atomics().checked_multiply_ratio(
            numerator.atomics(),
            denominator.atomics(),
        )?))
    }

    fn to_uint256_with_precision(&self, precision: u32) -> Result<Uint256, ContractError> {
        let value = self.atomics();

        Ok(value.checked_div(10u128.pow(self.decimal_places() - precision).into())?)
    }

    fn to_uint512_with_precision(&self, precision: u32) -> Result<Uint512, ContractError> {
        let value = Uint512::from(self.atomics());

        Ok(value.checked_div(10u128.pow(self.decimal_places() - precision).into())?)
    }
}
