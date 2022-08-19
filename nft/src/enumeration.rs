use crate::*;

#[near_bindgen]
impl NftContract {

    //Query for nft tokens on the contract regardless of the owner using pagination
    pub fn nft_tokens(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<JsonNft> {
        //get a vector of the keys in the token_metadata_by_id collection.
        let keys = self.token_metadata_by_id.keys_as_vector();
        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = u128::from(from_index.unwrap_or(U128(0))) as usize;
        let count = limit.unwrap_or(10) as usize;

        //iterate through the keys vector
        keys.iter()
            .skip(start) //skip to the index we specified in the start variable
            .take(count) // return "limit" elements or 0 if missing
            .map(|token_id| self.nft_token(token_id.clone()).unwrap()) // map onto Json Tokens
            .collect()
    }

    //get the total supply of NFTs for a given owner
    pub fn nft_supply_for_owner(&self, account_id: AccountId) -> U128 {
        let tokens_for_owner_set = self.tokens_per_owner.get(&account_id);
        if let Some(tokens_for_owner_set) = tokens_for_owner_set {
            U128(tokens_for_owner_set.len() as u128)
        } else {
            U128(0)
        }
    }

    //Query for all the tokens for an owner
    pub fn nft_tokens_for_owner(
        &self,
        account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonNft> {
        //get the set of tokens for the passed in owner
        let tokens_for_owner_set = self.tokens_per_owner.get(&account_id);
        if let Some(tokens_for_owner_set) = tokens_for_owner_set {
            //we'll convert the UnorderedSet into a vector of strings
            let keys = tokens_for_owner_set.as_vector();

            //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
            let start = u128::from(from_index.unwrap_or(U128(0))) as usize;

            //iterate through the keys vector
            return keys
                .iter()
                .skip(start as usize) //skip to the index we specified in the start variable
                .take(limit.unwrap_or(0) as usize) //take the first "limit" elements in the vector (or 0)
                .map(|token_id| self.nft_token(token_id.clone()).unwrap()) //map into Json Tokens
                .collect(); //turn key iterator back into a vector to return
        } else {
            return vec![];
        }
    }

    //Get tokens belonging to a collection
    pub fn tokens_by_collection_id(
        &self,
        collection_id: NftCollectionId,
        from_index: Option<u64>,
        limit: Option<u64>,
    ) -> Vec<JsonNft> {
        //get the set of tokens for the passed in owner
        let collection = self
            .collections_by_id
            .get(&collection_id)
            .expect("Collection does not exists");
        let token_keys_vec = collection.tokens;
        token_keys_vec
            .iter()
            .skip(from_index.unwrap_or(0) as usize)
            .take(limit.unwrap_or(0) as usize)
            .map(|token_id| self.nft_token(token_id.clone()).unwrap())
            .collect()
    }
}
