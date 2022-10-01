use crate::{
    *,
    listing::{
        constants::*,
        secondary::lib::{SecondaryListingId}
    },
};

pub(crate) fn hash_secondary_listing_id(listing_id: &SecondaryListingId) -> CryptoHash {
    let hashed_string = format!("{}.{}", listing_id.nft_contract_id, listing_id.nft_id);
    let mut hash = CryptoHash::default();
    hash.copy_from_slice(&env::sha256(hashed_string.as_bytes()));
    hash
}