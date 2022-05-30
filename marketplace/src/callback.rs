use crate::*;
use near_sdk::{AccountId,PromiseResult};

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
        assert_eq!(
            env::promise_results_count(),
            1,
            "This is a callback method"
        );
      
        // handle the result from the cross contract call this method is a callback for
        match env::promise_result(0) {
          PromiseResult::NotReady => unreachable!(),
          PromiseResult::Failed => env::panic_str("nft contract make_collection call failed"),
          PromiseResult::Successful(_result) => "all ok".to_string(),
        //   {
            //   let balance = near_sdk::serde_json::from_slice::<U128>(&result).unwrap();
            //   if balance.0 > 100000 {
            //       "Wow!".to_string()
            //   } else {
            //       "Hmmmm".to_string()
            //   }
        //   },
        }
    }

    fn freeze_collection_completion(&mut self, nft_contract_id: AccountId) {}
    fn delete_collection_completion(&mut self, nft_contract_id: AccountId) {}
}