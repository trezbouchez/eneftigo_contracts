use crate::{external::{NftMetadata, NftMutableMetadata}, *, listing::status::ListingStatus};

use near_sdk::json_types::{U128, U64};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonSecondaryListing {
    pub nft_contract_id: AccountId,
    pub token_id: String,
    pub approval_id: U64,
    pub seller_id: AccountId,
    pub price_yocto: Option<U128>,
    pub min_bid_yocto: Option<U128>,
    pub nft_metadata: NftMetadata,
    pub nft_mutable_metadata: NftMutableMetadata,
    pub start_timestamp: i64,
    pub end_timestamp: Option<i64>, // nanoseconds since 1970-01-01
    pub status: ListingStatus,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonSecondaryListingBid {
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

    pub fn secondary_listings(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonSecondaryListing> {
        // get a vector of the listings
        let listings = self.secondary_listings_by_id.values_as_vector();

        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = u128::from(from_index.unwrap_or(U128(0))) as usize;
        let count = limit.unwrap_or(10) as usize;

        //iterate through the listings
        listings
            .iter()
            .skip(start) //skip to the index we specified in the start variable
            .take(count) // return "limit" elements or 0 if missing
            .map(|listing| JsonSecondaryListing {
                nft_contract_id: listing.id.nft_contract_id,
                token_id: listing.id.token_id,
                approval_id: U64(listing.approval_id),
                seller_id: listing.seller_id,
                price_yocto: listing.price_yocto.map(|p| U128(p)),
                min_bid_yocto: listing.min_bid_yocto.map(|b| U128(b)),
                nft_metadata: listing.nft_metadata,
                nft_mutable_metadata: listing.nft_mutable_metadata,
                start_timestamp: listing.start_timestamp,
                end_timestamp: listing.end_timestamp,
                status: listing.status,
            })
            .collect()
    }

    pub fn secondary_listings_by_seller(
        &self,
        seller_account_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonSecondaryListing> {
        // get a vector of the listings
        if let Some(listing_ids) = self.secondary_listings_by_seller_id.get(&seller_account_id) {
            //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
            let start = u128::from(from_index.unwrap_or(U128(0))) as usize;
            let count = limit.unwrap_or(10) as usize;

            //iterate through the listings
            listing_ids
                .iter()
                .skip(start) //skip to the index we specified in the start variable
                .take(count) // return "limit" elements or 0 if missing
                .map(|listing_id| {
                    let listing = self
                        .secondary_listings_by_id
                        .get(&listing_id)
                        .expect("Could not find listing");
                    JsonSecondaryListing {
                        nft_contract_id: listing.id.nft_contract_id,
                        token_id: listing.id.token_id,
                        approval_id: U64(listing.approval_id),
                        seller_id: listing.seller_id,
                        price_yocto: listing.price_yocto.map(|p| U128(p)),
                        min_bid_yocto: listing.min_bid_yocto.map(|b| U128(b)),
                        nft_metadata: listing.nft_metadata,
                        nft_mutable_metadata: listing.nft_mutable_metadata,
                        start_timestamp: listing.start_timestamp,
                        end_timestamp: listing.end_timestamp,
                        status: listing.status,
                    }
                })
                .collect()
        } else {
            vec![]
        }
    }

    // get PrimaryListing by nft_contract_id
    pub fn is_listed(&self, nft_contract_id: AccountId, token_id: String) -> bool {
        let listing_id = SecondaryListingId {
            nft_contract_id: nft_contract_id.clone(),
            token_id: token_id.clone(),
        };
        self.secondary_listings_by_id.get(&listing_id).is_some()
    }

    pub fn secondary_listing(
        &self,
        nft_contract_id: AccountId,
        token_id: String,
    ) -> JsonSecondaryListing {
        let listing_id = SecondaryListingId {
            nft_contract_id: nft_contract_id.clone(),
            token_id: token_id.clone(),
        };
        let listing = self
            .secondary_listings_by_id
            .get(&listing_id)
            .expect("Could not find secondary listing");

        JsonSecondaryListing {
            nft_contract_id: nft_contract_id,
            token_id,
            approval_id: U64(listing.approval_id),
            seller_id: listing.seller_id,
            price_yocto: listing.price_yocto.map(|p| U128(p)),
            min_bid_yocto: listing.min_bid_yocto.map(|b| U128(b)),
            nft_metadata: listing.nft_metadata,
            nft_mutable_metadata: listing.nft_mutable_metadata,
            start_timestamp: listing.start_timestamp,
            end_timestamp: listing.end_timestamp,
            status: listing.status,
        }
    }

    // get bid by nft_contract_id and BidlId
    // there's no way to enumerate all bids for given primary listing
    pub fn secondary_listing_bid(
        &self,
        nft_contract_id: AccountId,
        token_id: String,
        bid_id: U64,
    ) -> JsonSecondaryListingBid {
        let bid_id = bid_id.0;

        let listing_id = SecondaryListingId {
            nft_contract_id,
            token_id,
        };
        let listing = self
            .secondary_listings_by_id
            .get(&listing_id)
            .expect("Could not find secondary listing");

        let bid = listing.bids.get(bid_id).expect("Bid not found");

        JsonSecondaryListingBid {
            id: U64(bid_id),
            bidder_id: bid.bidder_id,
            amount_yocto: U128(bid.amount_yocto),
        }
    }

    // get bids by nft_contract_id and bidder_id, results are paginated
    pub fn secondary_listing_bids_by_bidder(
        &self,
        nft_contract_id: AccountId,
        token_id: String,
        bidder_id: AccountId,
        from_index: Option<U128>,
        limit: Option<U64>,
    ) -> Vec<JsonSecondaryListingBid> {
        let from_index = from_index.map(|i| i.0);
        let limit = limit.map(|l| l.0);

        let listing_id = SecondaryListingId {
            nft_contract_id,
            token_id,
        };
        let listing = self
            .secondary_listings_by_id
            .get(&listing_id)
            .expect("Could not find secondary listing");

        let start = from_index.unwrap_or(0) as usize;
        let count = limit.unwrap_or(10) as usize;
        listing
            .bids
            .iter()
            .filter(|bid| bid.bidder_id == bidder_id)
            .skip(start)
            .take(count)
            .map(|bid| JsonSecondaryListingBid {
                id: U64(bid.id),
                bidder_id: bid.bidder_id,
                amount_yocto: U128(bid.amount_yocto),
            })
            .collect()
    }

    // get acceptable proposals by nft_contract_id, results are paginated
    pub fn secondary_listing_bids(
        &self,
        nft_contract_id: AccountId,
        token_id: String,
        from_index: Option<U128>,
        limit: Option<U64>,
    ) -> Vec<JsonSecondaryListingBid> {
        let from_index = from_index.map(|i| i.0);
        let limit = limit.map(|l| l.0);

        let listing_id = SecondaryListingId {
            nft_contract_id,
            token_id,
        };
        let listing = self
            .secondary_listings_by_id
            .get(&listing_id)
            .expect("Could not find secondary listing");

        // where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = from_index.unwrap_or(0) as usize;
        let count = limit.unwrap_or(10) as usize;

        listing
            .bids
            .iter()
            .skip(start) //skip to the index we specified in the start variable
            .take(count) // return "limit" elements or 0 if missing
            .map(|bid| JsonSecondaryListingBid {
                id: U64(bid.id),
                bidder_id: bid.bidder_id,
                amount_yocto: U128(bid.amount_yocto),
            })
            .collect()
    }
}

impl SecondaryListing {
    pub(crate) fn bid(&self, bid_id: &u64) -> Option<JsonSecondaryListingBid> {
        if let Some(bid) = self
            .bids
            .iter()
            .find(|bid| bid.id == *bid_id)
        {
            Some(JsonSecondaryListingBid {
                id: U64(bid.id),
                bidder_id: bid.bidder_id,
                amount_yocto: U128(bid.amount_yocto),
            })
        } else {
            None
        }
    }
}
