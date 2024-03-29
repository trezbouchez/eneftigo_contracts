use crate::*;
use chrono::{DateTime, NaiveDateTime, Utc};
use near_sdk::json_types::U64;
use sha2::*;

#[near_bindgen]
impl NftContract {
    // TODO: is asset per-collection or per-token?! Maybe there's both?
    // Returns token ID and storage used by just-minted token
    #[payable]
    pub fn mint(
        &mut self,
        receiver_id: AccountId,
        collection_id: U64,
        perpetual_royalties: Option<HashMap<AccountId, u32>>,
    ) -> (NftId, U64) {
        assert_eq!(
            &env::predecessor_account_id(),
            &self.owner_id,
            "Only the contract owner (Eneftigo Marketplace) can call this."
        );

        let mut collection = self
            .collections_by_id
            .get(&collection_id.0)
            .expect("Collection does not exists");
        assert!(
            !collection.is_frozen,
            "Collection is frozen. No more NFT can be minted."
        );

        // terminate early to save gas if deposit won't cover worst case storage cost
        let anticipated_storage = mint_storage(
            &collection.nft_metadata.title.clone().unwrap(),
            &collection.nft_metadata.media.clone().unwrap(),
            receiver_id.as_str(),
        );
        let anticipated_storage_cost = anticipated_storage as Balance * env::storage_byte_cost();
        assert!(
            env::attached_deposit() >= anticipated_storage_cost,
            "Attach at least {} yN to cover NFT storage (title {}, media {}, receiver {}",
            anticipated_storage_cost,
            collection.nft_metadata.title.clone().unwrap(),
            collection.nft_metadata.media.clone().unwrap(),
            receiver_id.as_str()
        );

        let new_token_index = collection.tokens.len() as u64;
        assert!(
            new_token_index < collection.max_supply,
            "Max collection supply reached. No more tokens can be minted"
        );
        // measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();

        // create a royalty map to store in the token
        let mut royalty = HashMap::new();

        // if perpetual royalties were passed into the function:
        if let Some(perpetual_royalties) = perpetual_royalties {
            // make sure that the length of the perpetual royalties is below 7 since we won't have enough GAS to pay out that many people
            assert!(
                perpetual_royalties.len() < 7,
                "Cannot add more than 6 perpetual royalty beneficiaries"
            );

            // iterate through the perpetual royalties and insert the account and amount in the royalty map
            for (account, amount) in perpetual_royalties {
                royalty.insert(account, amount);
            }
        }

        // we want to keep the NftId size fixed for predictable storage calculation
        let new_token_id_string = format!("{}:{}", collection_id.0, new_token_index);
        let token_id_hash = Sha256::digest(new_token_id_string.as_bytes());
        let new_token_id = format!("{:01$x}", token_id_hash, 64);

        let new_token = Nft {
            //set the owner ID equal to the receiver ID passed into the function
            minter_id: env::signer_account_id(),
            owner_id: receiver_id.clone(),
            collection_id: collection_id.0,
            approved_account_ids: Default::default(),
            next_approval_id: 0,
            royalty,
        };

        // insert the token ID and token struct and make sure that the token doesn't exist
        let existing_token = self.tokens_by_id.insert(&new_token_id, &new_token);
        assert!(
            existing_token.is_none(),
            "Token with ID {} already exists",
            new_token_id
        );

        let mut token_metadata = collection.nft_metadata.clone();
        token_metadata.copies = Some(new_token_index);
        let block_datetime = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc);
        token_metadata.issued_at = Some(block_datetime.to_rfc3339());
        self.token_metadata_by_id
            .insert(&new_token_id, &token_metadata);

        let token_mutable_metadata = collection.nft_mutable_metadata.clone();
        self.token_mutable_metadata_by_id
            .insert(&new_token_id, &token_mutable_metadata);

        // call the internal method for adding the token to the owner
        self.internal_add_token_to_owner(&new_token.owner_id, &new_token_id);

        // update collection and store
        collection.tokens.push(&new_token_id);
        if new_token_index == collection.tokens.len() {
            collection.is_frozen = true;
        }
        self.collections_by_id.insert(&collection_id.0, &collection);

        //calculate the required storage which was the used - initial
        let storage_usage = env::storage_usage() - initial_storage_usage;

        println!("{}, {}", receiver_id.to_string().len(), storage_usage);

        //refund any excess storage if the user attached too much. Will panic if deposit was insufficient
        refund_excess_deposit(storage_usage);

        // construct the mint log as per the events standard.
        let nft_mint_log: EventLog = EventLog {
            standard: NFT_STANDARD_NAME.to_string(),
            version: NFT_METADATA_SPEC.to_string(),
            event: EventLogVariant::NftMint(vec![NftMintLog {
                owner_id: new_token.owner_id.to_string(),  // token owner
                token_ids: vec![new_token_id.to_string()], // vector of tokens minted
                memo: None,                                // memo (optional)
            }]),
        };
        env::log_str(&nft_mint_log.to_string());

        (new_token_id, U64(storage_usage))
    }

    /*    #[payable]
    pub fn nft_burn(&mut self, token_id: NftId) {
        //measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();
        let owner_id = self.tokens_by_id.get(&token_id).unwrap().owner_id;
        self.internal_remove_token_from_owner(&owner_id, &token_id);
        self.token_metadata_by_id
            .remove(&token_id)
            .expect("Could not find token metadata");
        self.tokens_by_id
            .remove(&token_id)
            .expect("Could not find token by its id");

        //calculate the required storage which was the used - initial
        let freed_storage_in_bytes = initial_storage_usage - env::storage_usage();

        //refund freed storage
        refund(freed_storage_in_bytes);
    }*/
}

fn mint_storage(title: &str, media_url: &str, receiver_id: &str) -> u64 {
    return 1013 + title.len() as u64 + media_url.len() as u64 + 2u64 * receiver_id.len() as u64;
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    const MINT_WORST_CASE_STORAGE_BASE: u64 = 830; // actual, measured

    #[test]
    fn test_nft_mint_storage() {
        let account_id = AccountId::new_unchecked("marketplace.near".to_string());
        let context = VMContextBuilder::new()
            .predecessor_account_id(account_id.clone())
            .signer_account_id(account_id.clone())
            .attached_deposit(20000000000000000000000)
            .build();
        testing_env!(context);

        let mut contract = NftContract::new_default_meta("marketplace.near".parse().unwrap());
        let collection_id = U64(0);
        let title = String::from("collection_title");
        let asset_url = String::from("https://ipfs.io/ipfs/Qmef");
        let mut nft_metadata = TokenMetadata::new(&title, &asset_url);
        let block_datetime = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc);
        nft_metadata.issued_at = Some(block_datetime.to_rfc3339());
        let collection = NftCollection {
            nft_metadata,
            max_supply: 10,
            is_frozen: false,
            tokens: Vector::new(
                StorageKey::CollectionsInner {
                    collection_id: collection_id.0,
                }
                .try_to_vec()
                .unwrap(),
            ),
        };
        contract
            .collections_by_url
            .insert(&asset_url, &collection_id.0);
        contract
            .collections_by_id
            .insert(&collection_id.0, &collection);

        let storage_before = env::storage_usage();
        let receiver_name = String::from("receiver13.near");
        let receiver_id = AccountId::new_unchecked(receiver_name.clone());
        contract.mint(receiver_id, collection_id, None);
        let storage_after = env::storage_usage();
        println!(
            "{}, {}, {}, {}",
            title.len(),
            asset_url.len(),
            receiver_name.len(),
            storage_after - storage_before
        );

        assert!(false);
    }

    #[test]
    fn test_nft_mint() {
        let account_id = AccountId::new_unchecked("marketplace.near".to_string());
        let context = VMContextBuilder::new()
            .predecessor_account_id(account_id.clone())
            .signer_account_id(account_id.clone())
            .attached_deposit(8580000000000000000000)
            .build();
        testing_env!(context);

        let mut contract = NftContract::new_default_meta("marketplace.near".parse().unwrap());
        let collection_id = U64(0);
        let title = String::from("abcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefghabcdefgh");
        assert_eq!(title.len(), 128);
        let asset_url =
            String::from("https://ipfs.io/ipfs/Qme7ss3ARVgxv6rXqVPiikMJ8u2NLgmgszg13pYrDKEoiu");
        assert_eq!(asset_url.len(), 21 + 46);
        let mut nft_metadata = TokenMetadata::new(&title, &asset_url);
        let block_datetime = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(61, 0), Utc);
        nft_metadata.issued_at = Some(block_datetime.to_rfc3339());
        let collection = NftCollection {
            nft_metadata,
            max_supply: 10,
            is_frozen: false,
            tokens: Vector::new(
                StorageKey::CollectionsInner {
                    collection_id: collection_id.0,
                }
                .try_to_vec()
                .unwrap(),
            ),
        };

        let storage_before = env::storage_usage();
        contract
            .collections_by_url
            .insert(&asset_url, &collection_id.0);
        contract
            .collections_by_id
            .insert(&collection_id.0, &collection);
        let storage_after = env::storage_usage();
        assert!(
            storage_after - storage_before == NEW_COLLECTION_WORST_CASE_STORAGE,
            "Collection storage is not what was expected {}",
            storage_after - storage_before
        );

        let storage_before = storage_after;
        let receiver_name = String::from("receiver1.near");
        let receiver_id = AccountId::new_unchecked(receiver_name.clone());
        contract.mint(receiver_id, collection_id, None);
        let storage_after = env::storage_usage();
        let worst_case_storage: u64 = MINT_WORST_CASE_STORAGE_BASE + receiver_name.len() as u64 * 2;
        assert!(
            storage_after - storage_before <= worst_case_storage,
            "Mint storage of {} is not what was expected",
            storage_after - storage_before
        );

        let storage_before = storage_after;
        let receiver_name = String::from("receiver1.near");
        let receiver_id = AccountId::new_unchecked(receiver_name.clone());
        contract.mint(receiver_id, collection_id, None);
        let storage_after = env::storage_usage();
        let worst_case_storage: u64 = MINT_WORST_CASE_STORAGE_BASE + receiver_name.len() as u64 * 2;
        assert!(
            storage_after - storage_before <= worst_case_storage,
            "Mint storage of {} is not what was expected {}",
            storage_after - storage_before,
            worst_case_storage,
        );
    }
}
