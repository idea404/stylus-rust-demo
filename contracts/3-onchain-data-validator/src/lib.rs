#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::str::FromStr;
use rust_decimal::Decimal;
use stylus_sdk::prelude::*;

sol_storage! {
    #[entrypoint]
    pub struct DataValidator {
        /// Stores the last valid decimal submitted by a user. The value is a StorageString.
       string last_valid_submission;
    }
}

#[public]
impl DataValidator {
    /// Submits a string, validates if it's a proper decimal using rust_decimal, and stores it.
    /// Reverts the transaction if the format is invalid.
    pub fn submit_data(&mut self, value: String) -> Result<(), Vec<u8>> {
        match Decimal::from_str(&value) {
            Ok(decimal) => {
                // Additional validation: ensure the decimal is within reasonable bounds
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

        // Store the validated decimal string
        self.last_valid_submission.set_str(&value);

        Ok(())
    }

    /// Retrieves the last valid submission
    pub fn get_last_submission(&self) -> String {
        self.last_valid_submission.get_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use stylus_sdk::testing::*;

    /// Helper function to set up the test environment
    fn setup() -> (TestVM, DataValidator) {
        let vm = TestVM::default();
        let contract = DataValidator::from(&vm);
        (vm, contract)
    }

    #[test]
    fn test_submit_valid_decimal() {
        let (_vm, mut contract) = setup();
        let valid_decimal = "12345.6789".to_string();

        // submit valid data
        let result = contract.submit_data(valid_decimal.clone());
        assert!(result.is_ok(), "Submitting a valid decimal should succeed");

        // state was updated correctly
        assert_eq!(contract.get_last_submission(), valid_decimal);
    }

    #[test]
    fn test_rejects_invalid_format() {
        let (_vm, mut contract) = setup();
        let invalid_string = "this-is-not-a-decimal".to_string();

        // submit invalid data
        let result = contract.submit_data(invalid_string);
        assert!(result.is_err(), "Submitting an invalid format should fail");

        // the error message is correct and state is unchanged
        let err_msg = String::from_utf8(result.unwrap_err()).unwrap();
        assert_eq!(err_msg, "Invalid decimal format");
        assert_eq!(
            contract.get_last_submission(),
            "",
            "State should not change on failure"
        );
    }

    #[test]
    fn test_rejects_value_too_large() {
        let (_vm, mut contract) = setup();
        let large_value = "1000000001".to_string(); // One more than the max

        // submit a value that is too large
        let result = contract.submit_data(large_value);
        assert!(result.is_err(), "Submitting a large value should fail");

        // the error message is correct and state is unchanged
        let err_msg = String::from_utf8(result.unwrap_err()).unwrap();
        assert_eq!(err_msg, "Decimal value too large");
        assert_eq!(
            contract.get_last_submission(),
            "",
            "State should not change on failure"
        );
    }

    #[test]
    fn test_rejects_value_too_small() {
        let (_vm, mut contract) = setup();
        let small_value = "-1000000001".to_string(); // One less than the min

        // submit a value that is too small
        let result = contract.submit_data(small_value);
        assert!(result.is_err(), "Submitting a small value should fail");

        // the error message is correct and state is unchanged
        let err_msg = String::from_utf8(result.unwrap_err()).unwrap();
        assert_eq!(err_msg, "Decimal value too small");
        assert_eq!(
            contract.get_last_submission(),
            "",
            "State should not change on failure"
        );
    }
}
