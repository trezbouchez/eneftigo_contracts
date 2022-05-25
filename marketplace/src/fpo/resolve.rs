use crate::*;
use near_sdk::{AccountId,PromiseResult};

#[ext_contract(ext_self)]
trait FixedPriceOfferingResolver {
    fn fpo_resolve_purchase(&mut self, nft_account_id: AccountId, price_yocto: Balance);
    fn fpo_resolve_nft_deploy(&mut self, nft_account_id: AccountId) -> String;
}

trait FixedPriceOfferingResolver {
    fn fpo_resolve_purchase(&mut self, nft_account_id: AccountId, price_yocto: Balance);
    fn fpo_resolve_nft_deploy(&mut self, nft_account_id: AccountId) -> String;
}

#[near_bindgen]
impl FixedPriceOfferingResolver for MarketplaceContract {

    fn fpo_resolve_purchase(&mut self, nft_account_id: AccountId, price_yocto: Balance) {
        let fpo = &mut self
            .fpos_by_contract_id
            .get(&nft_account_id)
            .expect("Could not find NFT listing");

        fpo.supply_left -= 1;

        // transfer funds to seller
        Promise::new(fpo.offeror_id.clone()).transfer(price_yocto);

        // end offer if no supply left
        if fpo.supply_left == 0 {
            self.fpo_conclude(nft_account_id);
        } else {
            fpo.prune_supply_exceeding_acceptable_proposals();
        }
    }

    // if the NFT contract deployment fails, we need to remove the FPO from the marketplace
    // we don't decrement self.nft_account_id_prefix because there might have been multiple
    // new listings transaction in a single block and the resulting (decremented) value could 
    // correspond to a valid NFT contract
    // TODO: how about returning storage deposit to the signer?
    fn fpo_resolve_nft_deploy(&mut self, nft_account_id: AccountId) -> String {
//        assert_eq!(env::promise_results_count(), 1, "Too many promise results");
        match env::promise_result(0) {
            PromiseResult::NotReady => {
                self.internal_remove_fpo(&nft_account_id);
                env::panic_str("NFT contract unreachable")
                // unreachable!()
            },
            PromiseResult::Successful(_val) => {
                env::panic_str("All OK!");
                // return "SUKCES".to_string();
                // if let Ok(is_whitelisted) = near_sdk::serde_json::from_slice::<bool>(&val) {
                //     is_whitelisted
                // } else {
                //     env::panic(b"ERR_WRONG_VAL_RECEIVED")
                // }
            },
            PromiseResult::Failed => {
                self.internal_remove_fpo(&nft_account_id);
                env::panic_str("NFT deployment failed")
            }
        }
    }
}
