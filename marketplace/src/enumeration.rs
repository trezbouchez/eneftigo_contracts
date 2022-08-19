use crate::*;
// use crate::internal::*;

use near_sdk::json_types::{U128, U64};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonFixedPriceOffering {
    pub nft_contract_id: AccountId,
    pub collection_id: NftCollectionId,
    pub offeror_id: AccountId,
    pub supply_total: U64,
    pub buy_now_price_yocto: U128,
    // pub nft_metadata: TokenMetadata,
    pub end_timestamp: Option<i64>, // nanoseconds since 1970-01-01
    pub supply_left: U64,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonFixedPriceOfferingProposal {
    pub id: u64,
    pub proposer_id: AccountId,
    pub price_yocto: U128,
    pub is_acceptable: bool,
}

#[near_bindgen]
impl MarketplaceContract {
    // Query for FPOs from all offerrors, results are paginated
    pub fn fpos(
        &self,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonFixedPriceOffering> {
        // get a vector of the FPOs
        let fpos = self.fpos_by_id.values_as_vector();

        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = u128::from(from_index.unwrap_or(U128(0))) as usize;
        let count = limit.unwrap_or(10) as usize;

        //iterate through the fpos
        fpos.iter()
            .skip(start) //skip to the index we specified in the start variable
            .take(count) // return "limit" elements or 0 if missing
            .map(|fpo| JsonFixedPriceOffering {
                nft_contract_id: fpo.offering_id.nft_contract_id,
                collection_id: fpo.offering_id.collection_id,
                offeror_id: fpo.offeror_id,
                supply_total: U64(fpo.supply_total),
                buy_now_price_yocto: U128(fpo.buy_now_price_yocto),
                // nft_metadata: fpo.nft_metadata,
                end_timestamp: fpo.end_timestamp,
                supply_left: U64(fpo.supply_left),
            })
            .collect()
    }

    // get FixedPriceOffering by nft_contract_id
    pub fn fpo(
        &self,
        nft_contract_id: AccountId,
        collection_id: NftCollectionId,
    ) -> JsonFixedPriceOffering {
        let offering_id = OfferingId {
            nft_contract_id: nft_contract_id.clone(),
            collection_id,
        };
        let fpo = self
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not find Fixed Price Offering");

        JsonFixedPriceOffering {
            nft_contract_id: nft_contract_id,
            collection_id: collection_id,
            offeror_id: fpo.offeror_id,
            supply_total: U64(fpo.supply_total),
            buy_now_price_yocto: U128(fpo.buy_now_price_yocto),
            // nft_metadata: fpo.nft_metadata,
            end_timestamp: fpo.end_timestamp,
            supply_left: U64(fpo.supply_left),
        }
    }

    // get proposal by nft_contract_id and ProposalId
    // there's no way to enumerate all proposals for given FPO
    pub fn fpo_proposal(
        &self,
        nft_contract_id: AccountId,
        collection_id: NftCollectionId,
        proposal_id: ProposalId,
    ) -> Option<JsonFixedPriceOfferingProposal> {
        let offering_id = OfferingId {
            nft_contract_id,
            collection_id,
        };
        let fpo = self
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not find Fixed Price Offering");
        fpo.proposal(&proposal_id)
    }

    // get proposals by nft_contract_id and proposer_id, results are paginated
    pub fn fpo_proposals_by_proposer(
        &self,
        nft_contract_id: AccountId,
        collection_id: NftCollectionId,
        proposer_id: AccountId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonFixedPriceOfferingProposal> {
        let offering_id = OfferingId {
            nft_contract_id,
            collection_id,
        };
        let fpo = self
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not find Fixed Price Offering");
        let proposals_by_proposer_set = fpo.proposals_by_proposer.get(&proposer_id);
        if let Some(proposals_by_proposer_set) = proposals_by_proposer_set {
            let keys = proposals_by_proposer_set.as_vector();
            let start = u128::from(from_index.unwrap_or(U128(0))) as usize;
            let count = limit.unwrap_or(10) as usize;
            return keys
                .iter()
                .skip(start)
                .take(count)
                .map(|proposal_id| fpo.proposal(&proposal_id).unwrap())
                .collect();
        } else {
            return vec![];
        }
    }

    // get acceptable proposals by nft_contract_id, results are paginated
    pub fn fpo_acceptable_proposals(
        &self,
        nft_contract_id: AccountId,
        collection_id: NftCollectionId,
        from_index: Option<U128>,
        limit: Option<u64>,
    ) -> Vec<JsonFixedPriceOfferingProposal> {
        let offering_id = OfferingId {
            nft_contract_id,
            collection_id,
        };
        let fpo = self
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not find Fixed Price Offering");

        // where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = u128::from(from_index.unwrap_or(U128(0))) as usize;
        let count = limit.unwrap_or(10) as usize;

        fpo.acceptable_proposals
            .iter()
            .skip(start) //skip to the index we specified in the start variable
            .take(count) // return "limit" elements or 0 if missing
            .map(|proposal_id| {
                let proposal = fpo
                    .proposals
                    .get(&proposal_id)
                    .expect("Could not find proposal");
                JsonFixedPriceOfferingProposal {
                    id: proposal.id,
                    proposer_id: proposal.proposer_id,
                    price_yocto: U128(proposal.price_yocto),
                    is_acceptable: proposal.is_acceptable, // should always be true!
                }
            })
            .collect()
    }
}

impl FixedPriceOffering {
    pub(crate) fn proposal(
        &self,
        proposal_id: &ProposalId,
    ) -> Option<JsonFixedPriceOfferingProposal> {
        if let Some(proposal) = self.proposals.get(&proposal_id) {
            Some(JsonFixedPriceOfferingProposal {
                id: proposal.id,
                proposer_id: proposal.proposer_id,
                price_yocto: U128(proposal.price_yocto),
                is_acceptable: proposal.is_acceptable,
            })
        } else {
            None
        }
    }
}
