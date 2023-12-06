use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, Promise};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub registered_loans: UnorderedMap<AccountId, Vec<U128>>, // public_key -> loan amounts
    pub prover: AccountId,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn initialize(prover: AccountId) -> Self {
        return Self {
            registered_loans: UnorderedMap::new(b"s".to_vec()),
            prover,
        };
    }

    pub fn register(&mut self, user: AccountId) -> bool {
        if self.is_user_registered(&user) {
            log!(format!("User is already registered"));
            return false;
        }
        self.registered_loans.insert(&user, &Vec::new());

        true
    }

    pub fn verified_loan(&mut self, user: AccountId, amount: U128) -> bool {
        if env::predecessor_account_id() != self.prover {
            log!(format!("Forbidden"));
            return false;
        }
        if !self.is_user_registered(&user) {
            log!(format!("User is not registered"));
            return false;
        }

        let mut loans = self.registered_loans.get(&user).unwrap();
        loans.push(amount);
        self.registered_loans.insert(&user, &loans);

        let token_amt = near_sdk::ONE_NEAR * amount.0;
        Promise::new(user).transfer(token_amt);

        true
    }

    fn is_user_registered(&self, user: &AccountId) -> bool {
        match self.registered_loans.get(&user) {
            Some(_) => true,
            None => false,
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::str::FromStr;

    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    use super::*;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_initialize() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let prover = AccountId::from_str("prover.near").unwrap();
        let contract = Contract::initialize(prover.clone());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.registered_loans.is_empty(), true);
        assert_eq!(contract.prover, prover)
    }

    #[test]
    fn test_register_new_user() {
        let prover = AccountId::from_str("prover.near").unwrap();
        let new_user = AccountId::from_str("user.near").unwrap();
        let mut contract = Contract::initialize(prover);
        assert_eq!(contract.register(new_user.clone()), true);
        assert_eq!(contract.registered_loans.len(), 1);
        assert_eq!(contract.registered_loans.get(&new_user), Some(Vec::new()));
    }

    #[test]
    fn test_register_existing_user() {
        let prover = AccountId::from_str("prover.near").unwrap();
        let new_user = AccountId::from_str("user.near").unwrap();

        let mut contract = Contract::initialize(prover);
        assert_eq!(contract.register(new_user.clone()), true);
        assert_eq!(contract.registered_loans.len(), 1);
        assert_eq!(contract.registered_loans.get(&new_user), Some(Vec::new()));

        assert_eq!(contract.register(new_user.clone()), false);
        assert_eq!(contract.registered_loans.len(), 1);
        assert_eq!(contract.registered_loans.get(&new_user), Some(Vec::new()));
    }

    #[test]
    fn test_verified_loan() {
        let prover = accounts(1);
        let context = get_context(prover.clone());
        testing_env!(context.build());

        let user = AccountId::from_str("user.near").unwrap();
        let mut contract = Contract::initialize(prover);
        assert_eq!(contract.register(user.clone()), true);
        assert_eq!(contract.registered_loans.len(), 1);
        assert_eq!(contract.registered_loans.get(&user), Some(Vec::new()));

        assert_eq!(contract.verified_loan(user.clone(), U128(123)), true);
        assert_eq!(contract.registered_loans.len(), 1);
        assert_eq!(contract.registered_loans.get(&user), Some(vec![U128(123)]));
    }

    #[test]
    fn test_verified_loan_unregistered_user() {
        let prover = accounts(1);
        let context = get_context(prover.clone());
        testing_env!(context.build());

        let user = AccountId::from_str("user.near").unwrap();
        let mut contract = Contract::initialize(prover);

        assert_eq!(contract.verified_loan(user.clone(), U128(123)), false);
        assert_eq!(contract.registered_loans.len(), 0);
    }

    #[test]
    fn test_verified_loan_wrong_caller() {
        let prover = accounts(1);
        let context = get_context(accounts(0));
        testing_env!(context.build());

        let user = AccountId::from_str("user.near").unwrap();
        let mut contract = Contract::initialize(prover);

        assert_eq!(contract.register(user.clone()), true);

        assert_eq!(contract.verified_loan(user.clone(), U128(123)), false);
        assert_eq!(contract.registered_loans.len(), 1);
    }
}
