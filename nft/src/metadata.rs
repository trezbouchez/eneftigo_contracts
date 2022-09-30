use crate::*;
pub type NftId = String;
pub type NftCollectionId = u64;

//defines the payout type we'll be returning as a part of the royalty standards.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
    pub payout: HashMap<AccountId, U128>,
} 

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct NFTContractMetadata {
    pub spec: String,              // required, essentially a version like "nft-1.0.0"
    pub name: String,              // required, ex. "Trez"
    pub symbol: String,            // required, ex. "TREZ"
    pub icon: Option<String>,      // Data URL
    pub base_uri: Option<String>,  // Centralized gateway known to have reliable access to decentralized storage assets referenced by `reference` or `media` URLs
    pub reference: Option<String>, // URL to a JSON file with more info
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[derive(Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub title: Option<String>, // ex. "Arch Nemesis: Mail Carrier" or "Parcel #5055"
    pub description: Option<String>, // free-form description
    pub media: Option<String>, // URL to associated media, preferably to decentralized, content-addressed storage
    pub media_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of content referenced by the `media` field. Required if `media` is included.
    pub copies: Option<u64>, // number of copies of this set of metadata in existence when token was minted.
    pub issued_at: Option<u64>, // When token was issued or minted, Unix epoch in milliseconds
    pub expires_at: Option<u64>, // When token expires, Unix epoch in milliseconds
    pub starts_at: Option<u64>, // When token starts being valid, Unix epoch in milliseconds
    pub updated_at: Option<u64>, // When token was last updated, Unix epoch in milliseconds
    pub extra: Option<String>, // anything extra the NFT wants to store on-chain. Can be stringified JSON.
    pub reference: Option<String>, // URL to an off-chain JSON file with more info.
    pub reference_hash: Option<Base64VecU8>, // Base64-encoded sha256 hash of JSON from reference field. Required if `reference` is included.
}

impl TokenMetadata {

    #[allow(dead_code)]
    pub(crate) fn new(title: &str, media: &str) -> TokenMetadata {
        TokenMetadata { 
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

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Nft {
    // owner of the token
    pub owner_id: AccountId,
    // collection id
    pub collection_id: NftCollectionId,
    // list of approved accounts and their approval IDs
    pub approved_account_ids: HashMap<AccountId, u64>,
    // the next approval ID to give out. 
    pub next_approval_id: u64,
    // this is for the standard (perpetual) royalties (as opposed to Music Rights royalties)
    pub royalty: HashMap<AccountId, u32>,
}

//The Json token is what will be returned from view calls. 
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonNft {
    //token ID
    pub token_id: NftId,
    //owner of the token
    pub owner_id: AccountId,
    //collection_id
    pub collection_id: NftCollectionId,
    //token metadata
    pub metadata: TokenMetadata,
    //list of approved accounts and their approval IDs
    pub approved_account_ids: HashMap<AccountId, u64>,
    // this is for the standard (perpetual) royalties (as opposed to Music Rights royalties)
    pub royalty: HashMap<AccountId, u32>,
}

pub trait NonFungibleTokenMetadata {
    //view call for returning the contract metadata
    fn nft_metadata(&self) -> NFTContractMetadata;
}

#[near_bindgen]
impl NonFungibleTokenMetadata for NftContract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}