use crate::*;

#[near_bindgen]
impl Contract {
    // returns id of the collection
    #[payable]
    pub fn make_collection(&mut self, collection_id: u64, max_supply: u64) {
        assert_eq!(
            &env::predecessor_account_id(),
            &self.owner_id,
            "Only contract owner (Eneftigo Marketplace) can create collections"
        );

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
            .insert(&new_collection_id, &new_collection);
        assert!(
            previous_collection.is_none(),
            "Collection with id {} already exists",
            collection_id
        );

        // refund excess storage deposit
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;
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