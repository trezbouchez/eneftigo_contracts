use crate::{external::{NftMetadata, NftMutableMetadata}, *, listing::status::ListingStatus};

use near_sdk::json_types::{U128, U64};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonPrimaryListing {
    pub nft_contract_id: AccountId,
    pub collection_id: U64,
    pub seller_id: AccountId,
    pub supply_total: U64,
    pub price_yocto: Option<U128>,
    pub min_bid_yocto: Option<U128>,
    pub acceptable_bid_yocto: Option<U128>,
    pub nft_metadata: NftMetadata,
    pub nft_mutable_metadata: NftMutableMetadata,
    pub end_timestamp: Option<i64>, // nanoseconds since 1970-01-01
    pub supply_left: U64,
    pub status: ListingStatus,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonPrimaryListingBid {
    pub id: U64,
    pub bidder_id: AccountId,
    pub amount_yocto: U128,
}

// view-only methods

#[near_bindgen]
impl MarketplaceContract {
    // pub fn fpos_total_supply(
    //     &self,
    // ) -> U128 {
    //     U128(self.primary_listings_by_id.len() as u128)
    // }

    // pub fn fpo_min_proposal_price_yocto(
    //     &self,
    //     listing_id: PrimaryListingIdJson
    // ) -> Option<U128> {
    //     let listing_id = PrimaryListingId {
    //         nft_contract_id: listing_id.nft_contract_id,
    //         collection_id: listing_id.collection_id.0,
    //     };

    //     let fpo = self.primary_listings_by_id.get(&listing_id);
    //     if let Some(fpo) = fpo {
    //         if fpo.status == ListingStatus::Running {
    //             if fpo.min_proposal_price_yocto.is_some() {
    //                 return Some(U128(fpo.acceptable_price_yocto()));
    //             }
    //         }
    //     }
    //     return None;
    // }

    pub fn primary_listings(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonPrimaryListing> {
        // get a vector of the listings
        let listings = self.primary_listings_by_id.values_as_vector();

        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = u128::from(from_index.unwrap_or(U128(0))) as usize;
        let count = limit.unwrap_or(10) as usize;

        //iterate through the listings
        listings
            .iter()
            .skip(start) //skip to the index we specified in the start variable
            .take(count) // return "limit" elements or 0 if missing
            .map(|listing| {
                let acceptable_bid_yocto: Option<u128> = if listing.min_bid_yocto.is_some() {
                    Some(listing.acceptable_bid_yocto())
                } else {
                    None
                };
                JsonPrimaryListing {
                    nft_contract_id: listing.id.nft_contract_id,
                    collection_id: U64(listing.id.collection_id),
                    seller_id: listing.seller_id,
                    supply_total: U64(listing.supply_total),
                    price_yocto: listing.price_yocto.map(|p| U128(p)),
                    min_bid_yocto: listing.min_bid_yocto.map(|b| U128(b)),
                    acceptable_bid_yocto: acceptable_bid_yocto.map(|b| U128(b)),
                    nft_metadata: listing.nft_metadata,
                    nft_mutable_metadata: listing.nft_mutable_metadata,
                    end_timestamp: listing.end_timestamp,
                    supply_left: U64(listing.supply_left),
                    status: listing.status,
                }
            })
            .collect()
    }

    pub fn primary_listings_by_seller(
        &self,
        seller_account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonPrimaryListing> {
        // get a vector of the listings
        let listing_ids = self.primary_listings_by_seller_id.get(&seller_account_id);
        if listing_ids.is_none() {
            return vec![];
        }
        let listing_ids = listing_ids.unwrap();

        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = u128::from(from_index.unwrap_or(U128(0))) as usize;
        let count = limit.unwrap_or(10) as usize;

        listing_ids
            .iter()
            .skip(start) //skip to the index we specified in the start variable
            .take(count) // return "limit" elements or 0 if missing
            .map(|listing_id| {
                let listing = self
                    .primary_listings_by_id
                    .get(&listing_id)
                    .expect("Listing record does not exist");
                let acceptable_bid_yocto: Option<u128> = if listing.min_bid_yocto.is_some() {
                    Some(listing.acceptable_bid_yocto())
                } else {
                    None
                };
                JsonPrimaryListing {
                    nft_contract_id: listing.id.nft_contract_id,
                    collection_id: U64(listing.id.collection_id),
                    seller_id: listing.seller_id,
                    supply_total: U64(listing.supply_total),
                    price_yocto: listing.price_yocto.map(|p| U128(p)),
                    min_bid_yocto: listing.min_bid_yocto.map(|b| U128(b)),
                    acceptable_bid_yocto: acceptable_bid_yocto.map(|b| U128(b)),
                    nft_metadata: listing.nft_metadata,
                    nft_mutable_metadata: listing.nft_mutable_metadata,
                    end_timestamp: listing.end_timestamp,
                    supply_left: U64(listing.supply_left),
                    status: listing.status,
                }
            })
            .collect()
    }

    // get PrimaryListing by nft_contract_id
    pub fn primary_listing(
        &self,
        nft_contract_id: AccountId,
        collection_id: U64,
    ) -> JsonPrimaryListing {
        let listing_id = PrimaryListingId {
            nft_contract_id: nft_contract_id.clone(),
            collection_id: collection_id.0,
        };
        let listing = self
            .primary_listings_by_id
            .get(&listing_id)
            .expect("Could not find primary listing");
        let acceptable_bid_yocto: Option<u128> = if listing.min_bid_yocto.is_some() {
            Some(listing.acceptable_bid_yocto())
        } else {
            None
        };
        JsonPrimaryListing {
            nft_contract_id: nft_contract_id,
            collection_id: collection_id,
            seller_id: listing.seller_id,
            supply_total: U64(listing.supply_total),
            price_yocto: listing.price_yocto.map(|p| U128(p)),
            min_bid_yocto: listing.min_bid_yocto.map(|p| U128(p)),
            acceptable_bid_yocto: acceptable_bid_yocto.map(|b| U128(b)),
            nft_metadata: listing.nft_metadata,
            nft_mutable_metadata: listing.nft_mutable_metadata,
            end_timestamp: listing.end_timestamp,
            supply_left: U64(listing.supply_left),
            status: listing.status,
        }
    }

    // get bid by nft_contract_id and BidId
    pub fn primary_listing_bid(
        &self,
        nft_contract_id: AccountId,
        collection_id: U64,
        bid_id: U64,
    ) -> JsonPrimaryListingBid {
        let collection_id = collection_id.0;
        let bid_id = bid_id.0;

        let listing_id = PrimaryListingId {
            nft_contract_id,
            collection_id,
        };
        let listing = self
            .primary_listings_by_id
            .get(&listing_id)
            .expect("Could not find primary listing");

        let bid = listing.bids.get(bid_id).expect("Bid not found");

        JsonPrimaryListingBid {
            id: U64(bid_id),
            bidder_id: bid.bidder_id,
            amount_yocto: U128(bid.amount_yocto),
        }
    }

    // get bids by nft_contract_id and bidder_id, results are paginated
    pub fn primary_listing_bids_by_bidder(
        &self,
        nft_contract_id: AccountId,
        collection_id: U64,
        bidder_id: AccountId,
        from_index: Option<U128>,
        limit: Option<U64>,
    ) -> Vec<JsonPrimaryListingBid> {
        let collection_id = collection_id.0;
        let from_index = from_index.map(|i| i.0);
        let limit = limit.map(|l| l.0);

        let listing_id = PrimaryListingId {
            nft_contract_id,
            collection_id,
        };
        let listing = self
            .primary_listings_by_id
            .get(&listing_id)
            .expect("Could not find primary listing");
        let start = from_index.unwrap_or(0) as usize;
        let count = limit.unwrap_or(10) as usize;
        listing
            .bids
            .iter()
            .filter(|bid| bid.bidder_id == bidder_id)
            .skip(start)
            .take(count)
            .map(|bid| JsonPrimaryListingBid {
                id: U64(bid.id),
                bidder_id: bid.bidder_id,
                amount_yocto: U128(bid.amount_yocto),
            })
            .collect()
    }

    // get acceptable bids by nft_contract_id, results are paginated
    pub fn primary_listing_bids(
        &self,
        nft_contract_id: AccountId,
        collection_id: U64,
        from_index: Option<U128>,
        limit: Option<U64>,
    ) -> Vec<JsonPrimaryListingBid> {
        let collection_id = collection_id.0;
        let from_index = from_index.map(|i| i.0);
        let limit = limit.map(|l| l.0);

        let listing_id = PrimaryListingId {
            nft_contract_id,
            collection_id,
        };
        let listing = self
            .primary_listings_by_id
            .get(&listing_id)
            .expect("Could not find primary listing");

        // where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = from_index.unwrap_or(0) as usize;
        let count = limit.unwrap_or(10) as usize;

        listing
            .bids
            .iter()
            .skip(start) //skip to the index we specified in the start variable
            .take(count) // return "limit" elements or 0 if missing
            .map(|bid| JsonPrimaryListingBid {
                id: U64(bid.id),
                bidder_id: bid.bidder_id,
                amount_yocto: U128(bid.amount_yocto),
            })
            .collect()
    }
}

impl PrimaryListing {
    pub(crate) fn bid(&self, bid_id: &u64) -> Option<JsonPrimaryListingBid> {
        if let Some(bid) = self.bids.iter().find(|bid| bid.id == *bid_id) {
            Some(JsonPrimaryListingBid {
                id: U64(bid.id),
                bidder_id: bid.bidder_id,
                amount_yocto: U128(bid.amount_yocto),
            })
        } else {
            None
        }
    }
}
