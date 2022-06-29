use crate::*;

#[ext_contract(nft_contract)]
trait NFTContract {
    fn make_collection(&mut self, collection_id: u64, max_supply: u64) -> u64;
    fn freeze_collection(&mut self, collection_id: u64);
    fn delete_colleciton(&mut self, collection_id: u64);

    fn mint(
        &mut self,
        collection_id: CollectionId,
        receiver_id: AccountId,
        perpetual_royalties: Option<HashMap<AccountId, u32>>,
    );
}
