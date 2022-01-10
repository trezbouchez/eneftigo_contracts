use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        metadata: TokenMetadata,
        receiver_id: AccountId,
        perpetual_royalties: Option<HashMap<AccountId, u32>>,        
    ) {
        // measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();

        // create a royalty map to store in the token
        let mut royalty = HashMap::new();
        
        // if perpetual royalties were passed into the function: 
        if let Some(perpetual_royalties) = perpetual_royalties {
            // make sure that the length of the perpetual royalties is below 7 since we won't have enough GAS to pay out that many people
            assert!(perpetual_royalties.len() < 7, "Cannot add more than 6 perpetual royalty beneficiaries");

            // iterate through the perpetual royalties and insert the account and amount in the royalty map
            for (account, amount) in perpetual_royalties {
                royalty.insert(account, amount);
            }
        }

        //specify the token struct that contains the owner ID 
        let token = Token {
            //set the owner ID equal to the receiver ID passed into the function
            owner_id: receiver_id,
            approved_account_ids: Default::default(),
            next_approval_id: 0,
            royalty,
        };

        // insert the token ID and token struct and make sure that the token doesn't exist
        assert!(
            self.tokens_by_id.insert(&token_id, &token).is_none(),
            "Token already exists"
        );

        // insert the token ID and metadata
        self.token_metadata_by_id.insert(&token_id, &metadata);

        // call the internal method for adding the token to the owner
        self.internal_add_token_to_owner(&token.owner_id, &token_id);

        // construct the mint log as per the events standard.
        let nft_mint_log: EventLog = EventLog {
            standard: NFT_STANDARD_NAME.to_string(),
            version: NFT_METADATA_SPEC.to_string(),
            event: EventLogVariant::NftMint(vec![NftMintLog {
                owner_id: token.owner_id.to_string(),       // token owner
                token_ids: vec![token_id.to_string()],      // vector of tokens minted
                memo: None,                                 // memo (optional)
            }]),
        };

        // log the serialized json.
        env::log_str(&nft_mint_log.to_string());

        //calculate the required storage which was the used - initial
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        //refund any excess storage if the user attached too much. Panic if they didn't attach enough to cover what's required.
        refund_deposit(required_storage_in_bytes);
    }

    #[payable]
    pub fn nft_burn(
        &mut self,
        token_id: TokenId,
    ) {
         //measure the initial storage being used on the contract
        let initial_storage_usage = env::storage_usage();
        
        let owner_id = self.tokens_by_id.get(&token_id).unwrap().owner_id;
        self.internal_remove_token_from_owner(&owner_id, &token_id);
        self.token_metadata_by_id.remove(&token_id).expect("Could not find token metadata");
        self.tokens_by_id.remove(&token_id).expect("Could not find token by its id");

        //calculate the required storage which was the used - initial
        let freed_storage_in_bytes = initial_storage_usage - env::storage_usage();

        //refund freed storage
        refund(freed_storage_in_bytes);
    }
}