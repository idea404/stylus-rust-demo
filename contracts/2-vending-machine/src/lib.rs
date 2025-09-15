#![cfg_attr(not(any(test, feature = "export-abi")), no_main)]
#![cfg_attr(not(any(test, feature = "export-abi")), no_std)]

#[macro_use]
extern crate alloc;

use alloc::vec::Vec;
use stylus_sdk::{alloy_primitives::{Address, U256}, prelude::*};

const VEND_COOLDOWN_SECONDS: u64 = 60;

sol_storage! {
    #[entrypoint]
    pub struct VendingMachine {
        mapping(address => uint256) cupcake_balances;
        mapping(address => uint256) last_vend_time;
    }
}

#[public]
impl VendingMachine {
    pub fn vend(&mut self) -> Result<(), Vec<u8>> {
        let caller = self.vm().msg_sender();
        let last_time = self.last_vend_time.get(caller);
        let current_time = self.vm().block_timestamp();

        // This logic is now correct because our test setup will be more realistic.
        if last_time > U256::ZERO && current_time < last_time.to::<u64>() + VEND_COOLDOWN_SECONDS {
            return Err("Cooldown: Please wait before requesting another cupcake.".into());
        }

        self.last_vend_time.insert(caller, U256::from(current_time));

        let mut balance = self.cupcake_balances.setter(caller);
        let new_balance = balance.get() + U256::from(1);
        balance.set(new_balance);

        Ok(())
    }

    pub fn balance_of(&self, user: Address) -> U256 {
        self.cupcake_balances.get(user)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use stylus_sdk::testing::*;

    fn setup() -> (TestVM, VendingMachine, Address) {
        let vm = TestVM::default();
        vm.set_block_timestamp(1_000_000); 
        let contract = VendingMachine::from(&vm);
        let user = Address::from([0x01; 20]);
        vm.set_sender(user);
        (vm, contract, user)
    }

    #[test]
    fn test_initial_vend_succeeds() {
        let (_vm, mut contract, user) = setup();
        assert_eq!(contract.balance_of(user), U256::ZERO);
        contract.vend().unwrap();
        assert_eq!(contract.balance_of(user), U256::from(1));
    }

    #[test]
    fn test_vend_fails_when_too_soon() {
        let (vm, mut contract, user) = setup();
        contract.vend().unwrap(); // Last vend time is now 1,000,000

        vm.set_block_timestamp(vm.block_timestamp() + 30); // Timestamp is now 1,000,030
        let result = contract.vend();
        assert!(result.is_err()); // This will now pass
        assert_eq!(contract.balance_of(user), U256::from(1));
    }

    #[test]
    fn test_vend_succeeds_after_cooldown() {
        let (vm, mut contract, user) = setup();
        contract.vend().unwrap();

        vm.set_block_timestamp(vm.block_timestamp() + VEND_COOLDOWN_SECONDS + 1);
        contract.vend().unwrap();
        assert_eq!(contract.balance_of(user), U256::from(2));
    }
}