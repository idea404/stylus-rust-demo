#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::{string::{String, ToString}, vec::Vec};
use core::str::FromStr;
use rust_decimal::Decimal;
use stylus_sdk::{alloy_primitives::Address, prelude::*};

sol_storage! {
    #[entrypoint]
    pub struct DataValidator {
        /// Stores the last valid decimal submitted by each user.
        mapping(address => string) last_valid_submissions;
    }
}

#[public]
impl DataValidator {
    /// Submits a string, validates it as a decimal, and stores it for the caller.
    pub fn submit_data(&mut self, value: String) -> Result<(), Vec<u8>> {
        match Decimal::from_str(&value) {
            Ok(decimal) => {
                if decimal > Decimal::new(1_000_000_000, 0) {
                    return Err("Decimal value too large".to_string().into_bytes());
                }
                if decimal < Decimal::new(-1_000_000_000, 0) {
                    return Err("Decimal value too small".to_string().into_bytes());
                }
            }
            Err(_) => {
                return Err("Invalid decimal format".to_string().into_bytes());
            }
        }

        let caller = self.vm().msg_sender();
        self.last_valid_submissions.setter(caller).set_str(&value);
        Ok(())
    }

    /// Retrieves the last valid submission for a specific user.
    pub fn get_last_submission(&self, user: Address) -> String {
        self.last_valid_submissions.getter(user).get_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use stylus_sdk::testing::*;

    fn setup() -> (TestVM, DataValidator, Address) {
        let vm = TestVM::default();
        let contract = DataValidator::from(&vm);
        let user = Address::from([0x01; 20]);
        vm.set_sender(user);
        (vm, contract, user)
    }

    #[test]
    fn test_submit_valid_decimal() {
        let (_vm, mut contract, user) = setup();
        let valid_decimal = "12345.6789".to_string();

        contract.submit_data(valid_decimal.clone()).unwrap();
        assert_eq!(contract.get_last_submission(user), valid_decimal);
    }

    #[test]
    fn test_rejects_invalid_format() {
        let (_vm, mut contract, user) = setup();
        let invalid_string = "this-is-not-a-decimal".to_string();

        let result = contract.submit_data(invalid_string);
        assert!(result.is_err());
        assert_eq!(contract.get_last_submission(user), "");
    }
}