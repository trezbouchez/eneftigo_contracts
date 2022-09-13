use crate::*;

use url::Url;

const MAX_TITLE_LEN: usize = 128;
// for MAX_TITLE_LEN and maximum URL length of 2048 chars
// formula is 152 + title.len() + 2*media_url.len()
// this gives 152 + 128 + 2*2048 = 4376
pub const NEW_COLLECTION_WORST_CASE_STORAGE: u64 = 4376;

#[near_bindgen]
impl NftContract {
    // Makes new NFT collection. Asset URL and collecton_id must be unique.
    // Returns collection ID and the storage used by the collection (in bytes)
    #[payable]
    pub fn make_collection(
        &mut self,
        nft_metadata: TokenMetadata,
        max_supply: u64,
    ) -> (NftCollectionId, u64) {
        // assert_eq!(
        //     &env::predecessor_account_id(),
        //     &self.owner_id,
        //     "Only contract owner (Eneftigo Marketplace) can create collections"
        // );

        let title = nft_metadata
            .title
            .clone()
            .expect("NFT metadata must include title");
        // this is because the storage usage is (pesimistacally) computed for this max title length
        assert!(
            title.len() <= MAX_TITLE_LEN,
            "Title length cannot exceed {} characters",
            MAX_TITLE_LEN
        );

        let media_url = nft_metadata
            .media
            .clone()
            .expect("NFT metadata must include media (URL)");
        assert!(Url::parse(&media_url).is_ok(), "NFT asset URL is invalid");

        let storage_byte_cost = env::storage_byte_cost();
        let anticipated_storage_usage = make_collection_storage(&title, &media_url);
        let anticipated_storage_cost = anticipated_storage_usage as Balance * storage_byte_cost;
        let attached_deposit = env::attached_deposit();
        assert!(
            attached_deposit >= anticipated_storage_cost,
            "Attach at least {} yN to cover storage cost",
            anticipated_storage_cost
        );

        let initial_storage_usage = env::storage_usage();

        let collection_id = self.next_collection_id;
        self.next_collection_id += 1;

        let previous_collection = self.collections_by_url.insert(&media_url, &collection_id);
        assert!(
            previous_collection.is_none(),
            "Collection exists for media at {}",
            media_url,
        );

        let new_collection = NftCollection {
            nft_metadata,
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
        let actual_storage_usage = env::storage_usage() - initial_storage_usage;
        assert_eq!(
            actual_storage_usage, anticipated_storage_usage,
            "Anticipated storage usage of {} differs from actual {}",
            anticipated_storage_usage, actual_storage_usage
        );
        let actual_storage_cost = anticipated_storage_cost;

        // assert!(
        //     attached_deposit >= storage_cost,
        //     "The attached deposit of {} yN is insufficient to cover the storage costs of {} yN.",
        //     attached_deposit,
        //     storage_cost
        // );

        let refund_amount = attached_deposit - actual_storage_cost;
        if refund_amount > 0 {
            Promise::new(env::predecessor_account_id()).transfer(refund_amount);
        }

        (collection_id, actual_storage_usage)
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
            .remove(&collection.nft_metadata.media.unwrap())
            .expect("Could not remove collection from collections_by_url");
        let storage_freed = initial_storage_usage - env::storage_usage();
        refund(storage_freed);
    }
}

fn make_collection_storage(title: &str, media: &str) -> u64 {
    return 152 + title.len() as u64 + 2u64 * media.len() as u64;
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    #[test]
    #[should_panic(expected = r#"Collection exists for media at http://eneftigo/asset.png"#)]
    fn test_nft_make_collection_duplcated_asset_url() {
        let storage_cost_byte = env::storage_byte_cost();
        let account_id = AccountId::new_unchecked("marketplace.near".to_string());
        let context = VMContextBuilder::new()
            .predecessor_account_id(account_id.clone())
            .signer_account_id(account_id.clone())
            .attached_deposit(NEW_COLLECTION_WORST_CASE_STORAGE as Balance * storage_cost_byte)
            .build();
        testing_env!(context);

        let mut contract = NftContract::new_default_meta("test.near".parse().unwrap());
        let nft_metadata = TokenMetadata::new("collection", "http://eneftigo/asset.png");
        contract.make_collection(nft_metadata.clone(), 5);
        contract.make_collection(nft_metadata.clone(), 10);
    }

    #[test]
    fn test_nft_make_collection() {
        let storage_cost_byte = env::storage_byte_cost();
        let account_id = AccountId::new_unchecked("marketplace.near".to_string());
        let context = VMContextBuilder::new()
            .predecessor_account_id(account_id.clone())
            .signer_account_id(account_id.clone())
            .attached_deposit(NEW_COLLECTION_WORST_CASE_STORAGE as Balance * storage_cost_byte)
            .build();
        testing_env!(context);

        let mut contract = NftContract::new_default_meta("test.near".parse().unwrap());
        let nft_metadata = TokenMetadata::new("collection", "http://eneftigo/asset1.png");
        contract.make_collection(nft_metadata, 5);
        let nft_metadata = TokenMetadata::new("collection2", "http://eneftigo/asset0.png");
        contract.make_collection(nft_metadata, 5);
        let nft_metadata = TokenMetadata::new("collection", "http://eneftigo/asset22.png");
        contract.make_collection(nft_metadata, 10);
    }
}
