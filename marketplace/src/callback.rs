/*use crate::*;
use near_sdk::PromiseResult;

#[ext_contract(ext_self_nft)]
trait NFTContractCompletionHandler<S: FnOnce(), F: FnOnce(u64)> {
    fn make_collection_completion(
        &mut self,
        listing_id: PrimaryListingId,
        nft_attached_deposit: Balance,
    ) -> NftCollectionId;
    fn freeze_collection_completion(&mut self, listing_id: PrimaryListingId);
    fn delete_collection_completion(&mut self, listing_id: PrimaryListingId);
}

trait NFTContractCompletionHandler {
    fn make_collection_completion(
        &mut self,
        listing_id: PrimaryListingId,
        nft_attached_deposit: Balance,
    ) -> NftCollectionId;
    fn freeze_collection_completion(&mut self, listing_id: PrimaryListingId);
    fn delete_collection_completion(&mut self, listing_id: PrimaryListingId);
}

#[near_bindgen]
impl NFTContractCompletionHandler for MarketplaceContract {
    #[private]
    fn make_collection_completion(
        &mut self,
        listing_id: PrimaryListingId,
        nft_attached_deposit: Balance,
    ) -> NftCollectionId {
        assert_eq!(env::promise_results_count(), 1, "Too many data receipts");
        match env::promise_result(0) {
            PromiseResult::NotReady | PromiseResult::Failed => {
                let storage_before = env::storage_usage();
                self.internal_remove_primary_listing(&listing_id);
                let freed_storage = storage_before - env::storage_usage();
                // TODO: we may be tempted to decrement next_collection_id here but is this safe?
                // Could another transaction further increment it in the same block (and succeed)?
                let refund = freed_storage as Balance * env::storage_byte_cost() + nft_attached_deposit;
                if refund > 0 {
                    Promise::new(env::signer_account_id()).transfer(refund as Balance);
                }
                panic!("NFT make_collection failed");
            }
            PromiseResult::Successful(val) => {
                let (collection_id, nft_storage_usage) = near_sdk::serde_json::from_slice::<(NftCollectionId,u64)>(&val)
                    .expect("NFT make_collection returned unexpected value.");
                let refund = nft_attached_deposit - nft_storage_usage as Balance * env::storage_byte_cost();
                if refund > 0 {
                    Promise::new(env::signer_account_id()).transfer(refund as Balance);
                }
                collection_id
                // let nft_storage_str = format!("ALL OK, NFT storage {}, NFT attached {}", nft_storage_usage, nft_attached_deposit);
                // env::log_str(&nft_storage_str);
            }
        }
    }

    #[private]
    fn freeze_collection_completion(&mut self, listing_id: PrimaryListingId) {
        format!(
            "NOT IMPLEMENTED: freeze_collection_completion, listing_id {}",
            listing_id
        );
    }

    #[private]
    fn delete_collection_completion(&mut self, listing_id: PrimaryListingId) {
        format!(
            "NOT IMPLEMENTED: delete_collection_completion, listing_id {}",
            listing_id
        );
    }
}
*/