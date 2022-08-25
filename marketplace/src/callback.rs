use crate::*;
use near_sdk::PromiseResult;

#[ext_contract(ext_self_nft)]
trait NFTContractCompletionHandler {
    fn make_collection_completion(&mut self, offering_id: OfferingId, nft_attached_deposit: Balance);
    fn freeze_collection_completion(&mut self, offering_id: OfferingId);
    fn delete_collection_completion(&mut self, offering_id: OfferingId);
}

trait NFTContractCompletionHandler {
    fn make_collection_completion(&mut self, offering_id: OfferingId, nft_attached_deposit: Balance);
    fn freeze_collection_completion(&mut self, offering_id: OfferingId);
    fn delete_collection_completion(&mut self, offering_id: OfferingId);
}

#[near_bindgen]
impl NFTContractCompletionHandler for MarketplaceContract {
    #[private]
    fn make_collection_completion(&mut self, offering_id: OfferingId, nft_attached_deposit: Balance) {
        assert_eq!(env::promise_results_count(), 1, "Too many data receipts");
        // In case of failure we remove the dangling offering and return the full deposit
        // amount to the client
        let res_str = format!("CALLBACK RESULT {:#?}", env::promise_result(0));
        env::log_str(&res_str);
        match env::promise_result(0) {
            PromiseResult::NotReady | PromiseResult::Failed => {
                let storage_before = env::storage_usage();
                self.internal_remove_fpo(&offering_id);
                let freed_storage = storage_before - env::storage_usage();
                let refund = freed_storage as Balance * env::storage_byte_cost() + nft_attached_deposit;
                if refund > 0 {
                    Promise::new(env::signer_account_id()).transfer(refund as Balance);
                }
            }
            PromiseResult::Successful(val) => {
                let nft_storage_usage = near_sdk::serde_json::from_slice::<u64>(&val)
                    .expect("NFT make_collection returned unexpected value.");
                let refund = nft_attached_deposit - nft_storage_usage as Balance * env::storage_byte_cost();
                let nft_storage_str = format!("ALL OK, NFT storage {}, NFT attached {}", nft_storage_usage, nft_attached_deposit);
                env::log_str(&nft_storage_str);
                if refund > 0 {
                    Promise::new(env::signer_account_id()).transfer(refund as Balance);
                }
            }
        }
    }

    fn freeze_collection_completion(&mut self, offering_id: OfferingId) {
        format!(
            "NOT IMPLEMENTED: freeze_collection_completion, offering_id {}",
            offering_id
        );
    }
    fn delete_collection_completion(&mut self, offering_id: OfferingId) {
        format!(
            "NOT IMPLEMENTED: delete_collection_completion, offering_id {}",
            offering_id
        );
    }
}
