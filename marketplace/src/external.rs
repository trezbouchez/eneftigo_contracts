use crate::*;

use near_sdk::json_types::{U64};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[derive(Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NftMetadata {
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub media: Option<String>, // URL to associated media, preferably to decentralized, content-addressed storage
    pub media_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
    pub copies: Option<u64>, // number of copies of this set of metadata in existence when token was minted.
    pub issued_at: Option<String>, // When token was issued or minted, Unix epoch in milliseconds
    pub expires_at: Option<String>, // When token expires, Unix epoch in milliseconds
    pub starts_at: Option<String>, // When token starts being valid, Unix epoch in milliseconds
    pub updated_at: Option<String>, // When token was last updated, Unix epoch in milliseconds
    pub extra: Option<String>, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
    pub reference: Option<String>, // URL to an off-chain JSON file with more info.
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

impl NftMetadata {
    pub(crate) fn new(title: &str, media: &str) -> NftMetadata {
        NftMetadata { 
            title: Some(String::from(title)), 
            description: None, 
            media: Some(String::from(media)), 
            media_hash: None, 
            copies: None, 
            issued_at: None, 
            expires_at: None, 
            starts_at: None, 
            updated_at: None, 
            extra: None, 
            reference: None, 
            reference_hash: None
        }
    }
}

#[ext_contract(nft_contract)]
trait NFTContract {
    fn make_collection(&mut self, nft_metadata: NftMetadata, max_supply: U64) -> (U64,U64);
    fn freeze_collection(&mut self, collection_id: U64);
    fn delete_colleciton(&mut self, collection_id: U64);

    fn mint(
        &mut self,
        collection_id: U64,
        receiver_id: AccountId,
        perpetual_royalties: Option<HashMap<AccountId, u32>>,
    );
}
