use crate::*;
use near_sdk::AccountId;

#[ext_contract(ext_self)]
trait FixedPriceOfferingResolver {
    fn fpo_resolve_purchase(&mut self, nft_contract_id: AccountId, price_yocto: Balance);
}

trait FixedPriceOfferingResolver {
    fn fpo_resolve_purchase(&mut self, nft_contract_id: AccountId, price_yocto: Balance);
}

#[near_bindgen]
impl FixedPriceOfferingResolver for MarketplaceContract {

    fn fpo_resolve_purchase(
        &mut self, 
        nft_contract_id: AccountId,
        price_yocto: Balance
    ) {
        let fpo = &mut self
            .fpos_by_contract_id
            .get(&nft_contract_id)
            .expect("Could not find NFT listing");

        fpo.supply_left -= 1;

        // transfer funds to seller
        Promise::new(fpo.offeror_id.clone()).transfer(price_yocto);

        // end offer if no supply left
        if fpo.supply_left == 0 {
            self.fpo_conclude(nft_contract_id);
        } else {
            fpo.prune_supply_exceeding_acceptable_proposals();
        }
    }
}
