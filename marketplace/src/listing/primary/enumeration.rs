use crate::{
    *,
    listing::primary::lib::{PrimaryListingIdJson},
};

use super::super::{
    status::{ListingStatus},
};

use near_sdk::json_types::U128;

// view-only methods

#[near_bindgen]
impl MarketplaceContract {

    pub fn fpos_total_supply(
        &self,
    ) -> U128 {
        U128(self.primary_listings_by_id.len() as u128)
    }

    pub fn fpo_min_proposal_price_yocto(
        &self,
        listing_id: PrimaryListingIdJson
    ) -> Option<U128> {
        let listing_id = PrimaryListingId {
            nft_contract_id: listing_id.nft_contract_id,
            collection_id: listing_id.collection_id.0,
        };
        
        let fpo = self.primary_listings_by_id.get(&listing_id);
        if let Some(fpo) = fpo {
            if fpo.status == ListingStatus::Running {
                if fpo.min_proposal_price_yocto.is_some() {
                    return Some(U128(fpo.acceptable_price_yocto()));
                }
            }
        }
        return None;
    }
}
