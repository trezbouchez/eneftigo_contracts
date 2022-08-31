use crate::*;

#[near_bindgen]
impl NftContract {
    // Makes new NFT collection. Asset URL and collecton_id must be unique.
    // Returns collection ID and the storage used by the collection (in bytes)
    #[payable]
    pub fn make_collection(
        &mut self,
        asset_url: String,
        collection_id: u64,
        max_supply: u64,
    ) -> (NftCollectionId, u64) {
        // assert_eq!(
        //     &env::predecessor_account_id(),
        //     &self.owner_id,
        //     "Only contract owner (Eneftigo Marketplace) can create collections"
        // );

        // measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();

        let previous_collection = self.collections_by_url.insert(&asset_url, &collection_id);
        assert!(
            previous_collection.is_none(),
            "Collection exists for {}",
            asset_url
        );

        let new_collection = NftCollection {
            asset_url,
            max_supply,
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
        let storage_usage = env::storage_usage() - initial_storage_usage;
        let storage_cost = storage_usage as Balance * env::storage_byte_cost();
        let attached_deposit = env::attached_deposit();
        assert!(
            attached_deposit >= storage_cost,
            "The attached deposit of {} yN is insufficient to cover the storage costs of {} yN.",
            attached_deposit,
            storage_cost
        );

        let refund_amount = attached_deposit - storage_cost;
        if refund_amount > 0 {
            Promise::new(env::predecessor_account_id()).transfer(refund_amount);
        }

        (collection_id, storage_usage)
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
            .expect("Could not remove collection from collections_by_id");

        self.collections_by_url
            .remove(&collection.asset_url)
            .expect("Could not remove collection from collections_by_url");
        let storage_freed = initial_storage_usage - env::storage_usage();
        refund(storage_freed);
    }
}

impl NftContract {
    #[allow(dead_code)]
    pub(crate) fn make_collection_storage(asset_url: &str) -> u64 {
        let asset_url_len: u64 = asset_url.len().try_into().unwrap();
        136 + 2 * asset_url_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    #[test]
    #[should_panic(expected = r#"Collection exists for http://eneftigo/asset.png"#)]
    fn test_nft_make_collection_duplcated_asset_url() {
        let account_id = AccountId::new_unchecked("marketplace.near".to_string());
        let context = VMContextBuilder::new()
            .predecessor_account_id(account_id.clone())
            .signer_account_id(account_id.clone())
            .attached_deposit(1880000000000000000000)
            .build();
        testing_env!(context);

        let mut contract = NftContract::new_default_meta("test.near".parse().unwrap());
        let storage_before = env::storage_usage();
        contract.make_collection(
            String::from("http://eneftigo/asset.png"),
            9007199254740991,
            9007199254740991,
        );
        let storage_between = env::storage_usage();
        contract.make_collection(String::from("http://eneftigo/asset.png"), 0, 10);
    }

    #[test]
    fn test_nft_make_collection() {
        let account_id = AccountId::new_unchecked("marketplace.near".to_string());
        let context = VMContextBuilder::new()
            .predecessor_account_id(account_id.clone())
            .signer_account_id(account_id.clone())
            .attached_deposit(1880000000000000000000)
            .build();
        testing_env!(context);

        let mut contract = NftContract::new_default_meta("test.near".parse().unwrap());
        let storage_before = env::storage_usage();
        contract.make_collection(
            String::from("http://eneftigo/asset1.png"),
            9007199254740991,
            9007199254740991,
        );
        let storage_between = env::storage_usage();
        contract.make_collection(String::from("http://eneftigo/asset2.png"), 0, 10);
        let storage_after = env::storage_usage();

        println!(
            "{}, {}",
            storage_between - storage_before,
            storage_after - storage_between
        );
    }
}
