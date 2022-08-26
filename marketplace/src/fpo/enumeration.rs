use crate::*;
use crate::FixedPriceOfferingStatus::*;

use near_sdk::json_types::U128;
// view-only methods

#[near_bindgen]
impl MarketplaceContract {

    pub fn fpos_total_supply(
        &self,
    ) -> U128 {
        U128(self.fpos_by_id.len() as u128)
    }

    pub fn fpo_min_proposal_price_yocto(
        &self,
        offering_id: OfferingId
    ) -> Option<U128> {
        let fpo = self.fpos_by_id.get(&offering_id);
        if let Some(fpo) = fpo {
            if fpo.status == Running {
                if fpo.min_proposal_price_yocto.is_some() {
                    return Some(U128(fpo.acceptable_price_yocto()));
                }
            }
        }
        return None;
    }
}
