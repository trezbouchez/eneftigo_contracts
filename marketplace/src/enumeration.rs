use crate::{
    *,
    external::{NftMetadata},
    listing::{
        proposal::{ProposalId},
    },
};

use near_sdk::json_types::{U128, U64};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonPrimaryListing {
    pub nft_contract_id: AccountId,
    pub collection_id: NftCollectionId,
    pub seller_id: AccountId,
    pub supply_total: U64,
    pub buy_now_price_yocto: U128,
    pub nft_metadata: NftMetadata,
    pub end_timestamp: Option<i64>, // nanoseconds since 1970-01-01
    pub supply_left: U64,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonPrimaryListingProposal {
    pub id: u64,
    pub proposer_id: AccountId,
    pub price_yocto: U128,
}

#[near_bindgen]
impl MarketplaceContract {
    // Query for primary listings from all seller, results are paginated
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
        listings.iter()
            .skip(start) //skip to the index we specified in the start variable
            .take(count) // return "limit" elements or 0 if missing
            .map(|listing| JsonPrimaryListing {
                nft_contract_id: listing.id.nft_contract_id,
                collection_id: listing.id.collection_id,
                seller_id: listing.seller_id,
                supply_total: U64(listing.supply_total),
                buy_now_price_yocto: U128(listing.buy_now_price_yocto),
                nft_metadata: listing.nft_metadata,
                end_timestamp: listing.end_timestamp,
                supply_left: U64(listing.supply_left),
            })
            .collect()
    }

    // get PrimaryListing by nft_contract_id
    pub fn primary_listing(
        &self,
        nft_contract_id: AccountId,
        collection_id: NftCollectionId,
    ) -> JsonPrimaryListing {
        let listing_id = PrimaryListingId {
            nft_contract_id: nft_contract_id.clone(),
            collection_id,
        };
        let listing = self
            .primary_listings_by_id
            .get(&listing_id)
            .expect("Could not find primary listing");

        JsonPrimaryListing {
            nft_contract_id: nft_contract_id,
            collection_id: collection_id,
            seller_id: listing.seller_id,
            supply_total: U64(listing.supply_total),
            buy_now_price_yocto: U128(listing.buy_now_price_yocto),
            nft_metadata: listing.nft_metadata,
            end_timestamp: listing.end_timestamp,
            supply_left: U64(listing.supply_left),
        }
    }

    // get proposal by nft_contract_id and ProposalId
    // there's no way to enumerate all proposals for given primary listing
    pub fn primary_listing_proposal(
        &self,
        nft_contract_id: AccountId,
        collection_id: NftCollectionId,
        proposal_id: ProposalId,
    ) -> Option<JsonPrimaryListingProposal> {
        let listing_id = PrimaryListingId {
            nft_contract_id,
            collection_id,
        };
        let listing = self
            .primary_listings_by_id
            .get(&listing_id)
            .expect("Could not find primary listing");
        listing.proposal(&proposal_id)
    }

    // get proposals by nft_contract_id and proposer_id, results are paginated
    pub fn primary_listing_proposals_by_proposer(
        &self,
        nft_contract_id: AccountId,
        collection_id: NftCollectionId,
        proposer_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonPrimaryListingProposal> {
        let listing_id = PrimaryListingId {
            nft_contract_id,
            collection_id,
        };
        let listing = self
            .primary_listings_by_id
            .get(&listing_id)
            .expect("Could not find primary listing");
        let start = u128::from(from_index.unwrap_or(U128(0))) as usize;
        let count = limit.unwrap_or(10) as usize;
        listing.proposals
            .iter()
            .filter(|proposal| proposal.proposer_id == proposer_id)
            .skip(start)
            .take(count)
            .map(|proposal| JsonPrimaryListingProposal {
                id: proposal.id,
                proposer_id: proposal.proposer_id,
                price_yocto: U128(proposal.price_yocto),
            })
            .collect()
    }

    // get acceptable proposals by nft_contract_id, results are paginated
    pub fn primary_listing_proposals(
        &self,
        nft_contract_id: AccountId,
        collection_id: NftCollectionId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonPrimaryListingProposal> {
        let listing_id = PrimaryListingId {
            nft_contract_id,
            collection_id,
        };
        let listing = self
            .primary_listings_by_id
            .get(&listing_id)
            .expect("Could not find primary listing");

        // where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = u128::from(from_index.unwrap_or(U128(0))) as usize;
        let count = limit.unwrap_or(10) as usize;

        listing.proposals
            .iter()
            .skip(start) //skip to the index we specified in the start variable
            .take(count) // return "limit" elements or 0 if missing
            .map(|proposal| JsonPrimaryListingProposal {
                id: proposal.id,
                proposer_id: proposal.proposer_id,
                price_yocto: U128(proposal.price_yocto),
            })
            .collect()
    }
}

impl PrimaryListing {
    pub(crate) fn proposal(
        &self,
        proposal_id: &ProposalId,
    ) -> Option<JsonPrimaryListingProposal> {
        if let Some(proposal) = self
            .proposals
            .iter()
            .find(|proposal| proposal.id == *proposal_id)
        {
            Some(JsonPrimaryListingProposal {
                id: proposal.id,
                proposer_id: proposal.proposer_id,
                price_yocto: U128(proposal.price_yocto),
            })
        } else {
            None
        }
    }
}
