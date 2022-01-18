use crate::*;

#[near_bindgen]
impl MarketplaceContract {

    // Query for FPOs from all offerrors, results are paginated
    pub fn fpos(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<FixedPriceOffering> {
        //get a vector of the keys in the token_metadata_by_id collection.  
        return self.fpos_by_contract_id.values_as_vector().to_vec();
    }
}