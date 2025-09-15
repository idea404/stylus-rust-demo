#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use stylus_sdk::{
    alloy_primitives::{Address, U256},
    prelude::*,
};

/// The cooldown period in seconds between vending requests for a single user.
const VEND_COOLDOWN_SECONDS: u64 = 60;
/// The maximum number of user records the vending machine will store.
const MAX_USERS: usize = 20;

sol_storage! {
    /// Defines a struct to hold the data for a single user.
    pub struct UserRecord {
        address user;
        uint256 balance;
        uint256 last_vend_time;
    }

    #[entrypoint]
    pub struct VendingMachine {
        /// A fixed-size array of user records.
        UserRecord[20] user_records;
        /// The index to write the next new user, creating a circular buffer.
        uint256 next_user_index;
    }
}

#[public]
impl VendingMachine {
    /// Distributes one cupcake to the sender. Updates an existing user's record
    /// or adds a new one by overwriting the oldest record if full.
    pub fn vend(&mut self) -> Result<(), Vec<u8>> {
        let caller = self.vm().msg_sender();
        let current_time = self.vm().block_timestamp();

        // Search for the user's existing record. This is an O(n) operation.
        // We must find their index before we can get a mutable reference.
        let mut found_index: Option<U256> = None;
        for i in 0..self.user_records.len() {
            let i_u256 = U256::from(i);
            if let Some(record) = self.user_records.get(i_u256) {
                if record.user.get() == caller {
                    found_index = Some(i_u256);
                    break;
                }
            }
        }

        if let Some(index) = found_index {
            // User exists, update their record.
            let mut record = self.user_records.setter(index).unwrap();
            let last_time = record.last_vend_time.get();

            if current_time < last_time.to::<u64>() + VEND_COOLDOWN_SECONDS {
                return Err("Cooldown: Please wait before requesting another cupcake.".into());
            }

            record.last_vend_time.set(U256::from(current_time));
            let new_balance = record.balance.get() + U256::from(1);
            record.balance.set(new_balance);
        } else {
            // New user, add them at the next available index.
            let index = self.next_user_index.get();
            let mut record = self.user_records.setter(index).unwrap();

            record.user.set(caller);
            record.balance.set(U256::from(1));
            record.last_vend_time.set(U256::from(current_time));

            // Move the index for the next new user, wrapping around if necessary.
            let next_index = (index + U256::from(1)) % U256::from(MAX_USERS);
            self.next_user_index.set(next_index);
        }

        Ok(())
    }

    /// Returns the cupcake balance for a given address.
    pub fn balance_of(&self, user: Address) -> U256 {
        for i in 0..self.user_records.len() {
            if let Some(record) = self.user_records.get(U256::from(i)) {
                if record.user.get() == user {
                    return record.balance.get();
                }
            }
        }
        U256::ZERO // User not found
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use stylus_sdk::testing::*;

    fn setup() -> (TestVM, VendingMachine, Address) {
        let vm = TestVM::default();
        let contract = VendingMachine::from(&vm);
        let user1 = Address::from([0x01; 20]);
        vm.set_sender(user1);
        (vm, contract, user1)
    }

    #[test]
    fn test_initial_vend_for_new_user() {
        let (_vm, mut contract, user1) = setup();

        contract.vend().unwrap();

        assert_eq!(contract.balance_of(user1), U256::from(1));
        assert_eq!(contract.next_user_index.get(), U256::from(1));
    }

    #[test]
    fn test_subsequent_vend_updates_balance() {
        let (vm, mut contract, user1) = setup();
        contract.vend().unwrap(); // First vend

        // Advance time and vend again
        vm.set_block_timestamp(vm.block_timestamp() + VEND_COOLDOWN_SECONDS + 1);
        contract.vend().unwrap();

        assert_eq!(contract.balance_of(user1), U256::from(2));
        assert_eq!(
            contract.next_user_index.get(),
            U256::from(1),
            "Should not increment index for existing user"
        );
    }

    #[test]
    fn test_cooldown_prevents_vend() {
        let (vm, mut contract, user1) = setup();
        contract.vend().unwrap();

        vm.set_block_timestamp(vm.block_timestamp() + 30);
        let result = contract.vend();

        assert!(result.is_err());
        assert_eq!(contract.balance_of(user1), U256::from(1));
    }

    #[test]
    fn test_circular_buffer_overwrite() {
        let (vm, mut contract, _user) = setup();

        // Fill the array with 20 unique users
        for i in 0..MAX_USERS {
            let user = Address::from([i as u8 + 1; 20]);
            vm.set_sender(user);
            contract.vend().unwrap();
        }

        assert_eq!(
            contract.next_user_index.get(),
            U256::from(0),
            "Index should wrap around to 0"
        );

        let first_user = Address::from([1; 20]);
        let original_balance = contract.balance_of(first_user);
        assert_eq!(original_balance, U256::from(1));

        // The 21st user will overwrite the first user's slot
        let overwriting_user = Address::from([99; 20]);
        vm.set_sender(overwriting_user);
        contract.vend().unwrap();

        assert_eq!(contract.next_user_index.get(), U256::from(1));

        // The first user's record should be gone
        assert_eq!(contract.balance_of(first_user), U256::ZERO);
        // The new user should exist with a balance of 1
        assert_eq!(contract.balance_of(overwriting_user), U256::from(1));
    }
}
