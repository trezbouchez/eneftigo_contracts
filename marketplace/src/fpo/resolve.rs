use crate::*;
use near_sdk::{AccountId,PromiseResult};

#[ext_contract(ext_self)]
trait FixedPriceOfferingResolver {
    fn fpo_resolve_purchase(&mut self, offering_id: OfferingId, price_yocto: Balance);
}

trait FixedPriceOfferingResolver {
    fn fpo_resolve_purchase(&mut self, offering_id: OfferingId, price_yocto: Balance);
}

#[near_bindgen]
impl FixedPriceOfferingResolver for MarketplaceContract {

    fn fpo_resolve_purchase(&mut self, offering_id: OfferingId, price_yocto: Balance) {
        let fpo = &mut self
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not find NFT listing");

        fpo.supply_left -= 1;

        // transfer funds to seller
        Promise::new(fpo.offeror_id.clone()).transfer(price_yocto);

        // end offer if no supply left
        if fpo.supply_left == 0 {
            self.fpo_conclude(offering_id.nft_contract_id, offering_id.collection_id);
        } else {
            fpo.prune_supply_exceeding_acceptable_proposals();
        }
    }
}
