use crate::{
    constants::*,
    external::{nft_contract, NftMetadata},
    listing::{
        constants::*,
        secondary::{config::*, internal::hash_secondary_listing_id, lib::SecondaryListingStorageKey},
        status::ListingStatus,
    },
    *,
};

use near_sdk::json_types::{U64};

const NFT_MAKE_COLLECTION_GAS: Gas = Gas(5_000_000_000_000); // highest measured 3_920_035_683_889
const NFT_MAKE_COLLECTION_COMPLETION_GAS: Gas = Gas(6_000_000_000_000); // highest measured 5_089_357_803_858
