use crate::{
    *,
    external::{NftMetadata},
    listing::{
        proposal::{ProposalId},
        secondary::lib::{SecondaryListingIdJson},
    }
};

use super::super::{
    status::{ListingStatus},
};

use near_sdk::json_types::{U64,U128};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonSecondaryListing {
    pub nft_contract_id: AccountId,
    pub token_id: String,
    pub seller_id: AccountId,
    pub buy_now_price_yocto: U128,
    pub nft_metadata: NftMetadata,
    pub end_timestamp: Option<i64>, // nanoseconds since 1970-01-01
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonSecondaryListingProposal {
    pub id: U64,
    pub proposer_id: AccountId,
    pub price_yocto: U128,
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
        listings.iter()
            .skip(start) //skip to the index we specified in the start variable
            .take(count) // return "limit" elements or 0 if missing
            .map(|listing| JsonSecondaryListing {
                nft_contract_id: listing.id.nft_contract_id,
                token_id: listing.id.token_id,
                seller_id: listing.seller_id,
                buy_now_price_yocto: U128(listing.buy_now_price_yocto),
                nft_metadata: listing.nft_metadata,
                end_timestamp: listing.end_timestamp,
            })
            .collect()
    }

    // get PrimaryListing by nft_contract_id
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
            seller_id: listing.seller_id,
            buy_now_price_yocto: U128(listing.buy_now_price_yocto),
            nft_metadata: listing.nft_metadata,
            end_timestamp: listing.end_timestamp,
        }
    }

    // get proposal by nft_contract_id and ProposalId
    // there's no way to enumerate all proposals for given primary listing
    pub fn secondary_listing_proposal(
        &self,
        nft_contract_id: AccountId,
        token_id: String,
        proposal_id: ProposalId,
    ) -> Option<JsonSecondaryListingProposal> {
        let listing_id = SecondaryListingId {
            nft_contract_id,
            token_id
        };
        let listing = self
            .secondary_listings_by_id
            .get(&listing_id)
            .expect("Could not find secondary listing");
        listing.proposal(&proposal_id)
    }

    // get proposals by nft_contract_id and proposer_id, results are paginated
    pub fn secondary_listing_proposals_by_proposer(
        &self,
        nft_contract_id: AccountId,
        token_id: String,
        proposer_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonSecondaryListingProposal> {
        let listing_id = SecondaryListingId {
            nft_contract_id,
            token_id,
        };
        let listing = self
            .secondary_listings_by_id
            .get(&listing_id)
            .expect("Could not find secondary listing");
        let start = u128::from(from_index.unwrap_or(U128(0))) as usize;
        let count = limit.unwrap_or(10) as usize;
        listing.proposals
            .iter()
            .filter(|proposal| proposal.proposer_id == proposer_id)
            .skip(start)
            .take(count)
            .map(|proposal| JsonSecondaryListingProposal {
                id: U64(proposal.id),
                proposer_id: proposal.proposer_id,
                price_yocto: U128(proposal.price_yocto),
            })
            .collect()
    }

    // get acceptable proposals by nft_contract_id, results are paginated
    pub fn secondary_listing_proposals(
        &self,
        nft_contract_id: AccountId,
        token_id: String,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonSecondaryListingProposal> {
        let listing_id = SecondaryListingId {
            nft_contract_id,
            token_id,
        };
        let listing = self
            .secondary_listings_by_id
            .get(&listing_id)
            .expect("Could not find secondary listing");

        // where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = u128::from(from_index.unwrap_or(U128(0))) as usize;
        let count = limit.unwrap_or(10) as usize;

        listing.proposals
            .iter()
            .skip(start) //skip to the index we specified in the start variable
            .take(count) // return "limit" elements or 0 if missing
            .map(|proposal| JsonSecondaryListingProposal {
                id: U64(proposal.id),
                proposer_id: proposal.proposer_id,
                price_yocto: U128(proposal.price_yocto),
            })
            .collect()
    }
}

impl SecondaryListing {
    pub(crate) fn proposal(
        &self,
        proposal_id: &u64,
    ) -> Option<JsonSecondaryListingProposal> {
        if let Some(proposal) = self
            .proposals
            .iter()
            .find(|proposal| proposal.id == *proposal_id)
        {
            Some(JsonSecondaryListingProposal {
                id: U64(proposal.id),
                proposer_id: proposal.proposer_id,
                price_yocto: U128(proposal.price_yocto),
            })
        } else {
            None
        }
    }
}
