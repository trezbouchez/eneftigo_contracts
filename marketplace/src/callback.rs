use crate::*;
use near_sdk::{PromiseResult};

#[ext_contract(ext_self_nft)]
trait NFTContractCompletionHandler {
    fn make_collection_completion(&mut self, offering_id: OfferingId) -> String;
    fn freeze_collection_completion(&mut self, offering_id: OfferingId);
    fn delete_collection_completion(&mut self, offering_id: OfferingId);
}

trait NFTContractCompletionHandler {
    fn make_collection_completion(&mut self, offering_id: OfferingId) -> String;
    fn freeze_collection_completion(&mut self, offering_id: OfferingId);
    fn delete_collection_completion(&mut self, offering_id: OfferingId);
}

#[near_bindgen]
impl NFTContractCompletionHandler for MarketplaceContract {
    fn make_collection_completion(&mut self, offering_id: OfferingId) -> String {
        assert_eq!(env::promise_results_count(), 1, "Too many data receipts");
        // handle the result from the cross contract call
        // TODO: if failed, free storage and return deposits
        match env::promise_result(0) {
            PromiseResult::NotReady => {
                self.internal_remove_fpo(&offering_id);
                format!("UNREACHABLE: STORAGE USED: {}, ATTACHED_DEPOSIT: {}", env::storage_usage(), env::attached_deposit())
                // unreachable!()
            }
            PromiseResult::Failed => {
                self.internal_remove_fpo(&offering_id);
                format!("FAILED: STORAGE USED: {}, ATTACHED_DEPOSIT: {}", env::storage_usage(), env::attached_deposit())
            }
            PromiseResult::Successful(_result) => {
                format!("SUCCESS: STORAGE USED: {}, ATTACHED_DEPOSIT: {}", env::storage_usage(), env::attached_deposit())
            }
        }
    }

    fn freeze_collection_completion(&mut self, offering_id: OfferingId) {
        format!("NOT IMPLEMENTED: freeze_collection_completion, offering_id {}", offering_id);
    }
    fn delete_collection_completion(&mut self, offering_id: OfferingId) {
        format!("NOT IMPLEMENTED: delete_collection_completion, offering_id {}", offering_id);
    }
}
