use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, log, near_bindgen, AccountId, PublicKey, PanicOnDefault};
use near_sdk::json_types::U128;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub registered_loans: UnorderedMap<PublicKey, Vec<U128>>, // public_key -> loan amounts
    pub prover: AccountId,
}

#[near_bindgen]
impl Contract {

    #[init]
    pub fn initialize(
        prover: AccountId,
    ) -> Self {
        return Self {
            registered_loans: UnorderedMap::new(b"s".to_vec()),
            prover: prover,
        };
    }

    pub fn register(
        &mut self,
        user_pk: PublicKey
    ) -> bool {
        if self.is_user_registered(user_pk.clone()) {
            log!(format!(
                "User is already registered"
            ));
            return false;
        }
        self.registered_loans.insert(&user_pk, &Vec::new());

        true
    }

    pub fn verified_loan(&mut self, user_pk: PublicKey, amount: U128) -> bool{
        if env::predecessor_account_id() != self.prover {
            log!(format!(
                "Forbidden"
            ));
            return false;
        }
        if !self.is_user_registered(user_pk.clone()) {
            log!(format!(
                "User is not registered"
            ));
            return false;
        }

        let mut loans = self.registered_loans.get(&user_pk).unwrap();
        loans.push(amount);
        self.registered_loans.insert(&user_pk, &loans);

        true
    }

    fn is_user_registered(&self, user_pk: PublicKey) -> bool {
        match self.registered_loans.get(&user_pk) {
            Some(_) => true,
            None => false,
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use std::str::FromStr;

    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{testing_env};

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
        let new_user_pk = PublicKey::from_str("ed25519:7E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp").unwrap();
        let mut contract = Contract::initialize(prover);
        assert_eq!(contract.register(new_user_pk.clone()), true);
        assert_eq!(contract.registered_loans.len(), 1);
        assert_eq!(contract.registered_loans.get(&new_user_pk), Some(Vec::new()));
    }

    #[test]
    fn test_register_existing_user() {
        let prover = AccountId::from_str("prover.near").unwrap();
        let new_user_pk = PublicKey::from_str("ed25519:7E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp").unwrap();
        let mut contract = Contract::initialize(prover);
        assert_eq!(contract.register(new_user_pk.clone()), true);
        assert_eq!(contract.registered_loans.len(), 1);
        assert_eq!(contract.registered_loans.get(&new_user_pk), Some(Vec::new()));

        assert_eq!(contract.register(new_user_pk.clone()), false);
        assert_eq!(contract.registered_loans.len(), 1);
        assert_eq!(contract.registered_loans.get(&new_user_pk), Some(Vec::new()));
    }

    #[test]
    fn test_verified_loan() {
        let prover = accounts(1);
        let context = get_context(prover.clone());
        testing_env!(context.build());

        let new_user_pk = PublicKey::from_str("ed25519:7E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp").unwrap();
        let mut contract = Contract::initialize(prover);
        assert_eq!(contract.register(new_user_pk.clone()), true);
        assert_eq!(contract.registered_loans.len(), 1);
        assert_eq!(contract.registered_loans.get(&new_user_pk), Some(Vec::new()));

        assert_eq!(contract.verified_loan(new_user_pk.clone(), U128(123)), true);
        assert_eq!(contract.registered_loans.len(), 1);
        assert_eq!(contract.registered_loans.get(&new_user_pk), Some(vec![U128(123)]));
    }

    #[test]
    fn test_verified_loan_unregistered_user() {
        let prover = accounts(1);
        let context = get_context(prover.clone());
        testing_env!(context.build());

        let new_user_pk = PublicKey::from_str("ed25519:7E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp").unwrap();
        let mut contract = Contract::initialize(prover);

        assert_eq!(contract.verified_loan(new_user_pk.clone(), U128(123)), false);
        assert_eq!(contract.registered_loans.len(), 0);
    }

    #[test]
    fn test_verified_loan_wrong_caller() {
        let prover = accounts(1);
        let context = get_context(accounts(0));
        testing_env!(context.build());

        let new_user_pk = PublicKey::from_str("ed25519:7E8sCci9badyRkXb3JoRpBj5p8C6Tw41ELDZoiihKEtp").unwrap();
        let mut contract = Contract::initialize(prover);

        assert_eq!(contract.register(new_user_pk.clone()), true);

        assert_eq!(contract.verified_loan(new_user_pk.clone(), U128(123)), false);
        assert_eq!(contract.registered_loans.len(), 1);
    }
}