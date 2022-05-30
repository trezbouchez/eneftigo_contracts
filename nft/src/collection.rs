use crate::*;

#[near_bindgen]
impl Contract {

    #[payable]
    pub fn make_collection(&mut self, collection_id: u64, max_supply: u64) {
        // assert_eq!(
        //     &env::predecessor_account_id(),
        //     &self.owner_id,
        //     "Only contract owner (Eneftigo Marketplace) can create collections"
        // );

        // measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();

        let new_collection = Collection {
            max_supply: max_supply,
            is_frozen: false,
            tokens: Vector::new(
                StorageKey::CollectionsInner {
                    collection_id: collection_id,
                }
                .try_to_vec()
                .unwrap(),
            ),
        };

        let previous_collection = self
            .collections_by_id
            .insert(&collection_id, &new_collection);
        assert!(
            previous_collection.is_none(),
            "Collection with id {} already exists",
            collection_id
        );

        // refund excess storage deposit
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;
        println!("required storage: {}", required_storage_in_bytes);
        refund_excess_deposit(required_storage_in_bytes);
    }

    pub fn freeze_collection(&mut self, collection_id: u64) {
        assert_eq!(
            &env::predecessor_account_id(),
            &self.owner_id,
            "Only contract owner (Eneftigo Marketplace) can freeze collections"
        );

        let mut collection = self
            .collections_by_id
            .get(&collection_id)
            .expect("Collection does not exist");
        assert!(
            !collection.tokens.is_empty(),
            "Cannot freeze an empty collection"
        );
        collection.is_frozen = true;
        self.collections_by_id.insert(&collection_id, &collection);
    }

    pub fn delete_collection(&mut self, collection_id: u64) {
        assert_eq!(
            &env::predecessor_account_id(),
            &self.owner_id,
            "Only contract owner (Eneftigo Marketplace) can delete collections"
        );

        // measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();

        let collection = self
            .collections_by_id
            .get(&collection_id)
            .expect("Collection does not exist");
        assert!(
            collection.tokens.is_empty(),
            "Can only delete a collection if not tokens have been minted"
        );
        self.collections_by_id
            .remove(&collection_id)
            .expect("Could not remove collection");

        let storage_freed = initial_storage_usage - env::storage_usage();
        refund(storage_freed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    #[test]
    fn test_storage() {
        let account_id = AccountId::new_unchecked("marketplace.near".to_string());
        let context = VMContextBuilder::new()
            .predecessor_account_id(account_id.clone())
            .signer_account_id(account_id.clone())
            .attached_deposit(790000000000000000000)
            .build();
        testing_env!(context);

        let mut contract = Contract::new_default_meta("test.near".parse().unwrap());
        contract.make_collection(9007199254740991, 9007199254740991);
        contract.make_collection(0, 10);
    }
}