use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn make_collection(&mut self, asset_url: String, collection_id: u64, max_supply: u64) {
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

        let new_collection = Collection {
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
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;
        let storage_str = format!("NFT make_collection storage: {}", required_storage_in_bytes);
        env::log_str(&storage_str);

        // println!("required storage: {}", required_storage_in_bytes);

        // let required_cost = env::storage_byte_cost() * Balance::from(required_storage_in_bytes);
        // let storage_string = format!("STORAGE {}, PREDICTED {}", required_storage_in_bytes, predicted_cost);
        // env::log_str(&storage_string);

        // let log_string = format!("NFT deposit attached {} required {}, will refund to {}", required_cost, env::attached_deposit(), env::predecessor_account_id());
        // env::log_str(&log_string);

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
            .expect("Could not remove collection from collections_by_id");

        self.collections_by_url
            .remove(&collection.asset_url)
            .expect("Could not remove collection from collections_by_url");
        let storage_freed = initial_storage_usage - env::storage_usage();
        refund(storage_freed);
    }
}

impl Contract {
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

        let mut contract = Contract::new_default_meta("test.near".parse().unwrap());
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

        let mut contract = Contract::new_default_meta("test.near".parse().unwrap());
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
