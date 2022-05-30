use crate::*;
use near_sdk::{AccountId,PromiseResult};

#[ext_contract(ext_self_nft)]
trait NFTContractCompletionHandler {
    fn make_collection_completion(&mut self, nft_contract_id: AccountId);
    fn freeze_collection_completion(&mut self, nft_contract_id: AccountId);
    fn delete_collection_completion(&mut self, nft_contract_id: AccountId);
}

trait NFTContractCompletionHandler {
    fn make_collection_completion(&mut self, nft_contract_id: AccountId);
    fn freeze_collection_completion(&mut self, nft_contract_id: AccountId);
    fn delete_collection_completion(&mut self, nft_contract_id: AccountId);
}

#[near_bindgen]
impl NFTContractCompletionHandler for MarketplaceContract {

    fn make_collection_completion(&mut self, nft_contract_id: AccountId) {}
    fn freeze_collection_completion(&mut self, nft_contract_id: AccountId) {}
    fn delete_collection_completion(&mut self, nft_contract_id: AccountId) {}
}