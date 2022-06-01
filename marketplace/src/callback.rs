use crate::*;
use near_sdk::{AccountId, PromiseResult};

#[ext_contract(ext_self_nft)]
trait NFTContractCompletionHandler {
    fn make_collection_completion(&mut self, offering_id: OfferingId) -> String;
    fn freeze_collection_completion(&mut self, nft_contract_id: AccountId);
    fn delete_collection_completion(&mut self, nft_contract_id: AccountId);
}

trait NFTContractCompletionHandler {
    fn make_collection_completion(&mut self, offering_id: OfferingId) -> String;
    fn freeze_collection_completion(&mut self, nft_contract_id: AccountId);
    fn delete_collection_completion(&mut self, nft_contract_id: AccountId);
}

#[near_bindgen]
impl NFTContractCompletionHandler for MarketplaceContract {
    fn make_collection_completion(&mut self, offering_id: OfferingId) -> String {
        assert_eq!(env::promise_results_count(), 1, "Too many data receipts");
        // handle the result from the cross contract call this method is a callback for
        match env::promise_result(0) {
            PromiseResult::NotReady => {
                // self.internal_remove_fpo(&offering_id);
                format!("UNREACHABLE: STORAGE USED: {}, ATTACHED_DEPOSIT: {}", env::storage_usage(), env::attached_deposit())
                // unreachable!()
            }
            PromiseResult::Failed => {
                // self.internal_remove_fpo(&offering_id);
                format!("FAILED: STORAGE USED: {}, ATTACHED_DEPOSIT: {}", env::storage_usage(), env::attached_deposit())

                // env::panic_str("NFT Contract make_collection failed")
            }
            PromiseResult::Successful(_result) => {
                format!("SUCCESS: STORAGE USED: {}, ATTACHED_DEPOSIT: {}", env::storage_usage(), env::attached_deposit())
            }
        }
    }

    fn freeze_collection_completion(&mut self, nft_contract_id: AccountId) {}
    fn delete_collection_completion(&mut self, nft_contract_id: AccountId) {}
}
