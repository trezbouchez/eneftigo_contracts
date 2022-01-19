use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct JsonFixedPriceOffering {
    pub nft_contract_id: AccountId,
    pub offeror_id: AccountId,
    pub nft_max_supply: U128,
    pub price_yocto: U128,
    pub nft_metadata: NFTMetadata,
    pub end_timestamp: Option<i64>,         // nanoseconds since 1970-01-01
    pub nft_supply_sold: U128,
}

#[near_bindgen]
impl MarketplaceContract {

    // Query for FPOs from all offerrors, results are paginated
    pub fn fpos(&self, from_index: Option<U128>, limit: Option<u64>) -> Vec<JsonFixedPriceOffering> {
        // get a vector of the FPOs
        let fpos = self.fpos_by_contract_id.values_as_vector();

        //where to start pagination - if we have a from_index, we'll use that - otherwise start from 0 index
        let start = u128::from(from_index.unwrap_or(U128(0)));

        //iterate through the fpos
        fpos.iter()
            .skip(start as usize)   //skip to the index we specified in the start variable
            .take(limit.unwrap_or(0) as usize)     // return "limit" elements or 0 if missing
            .map(|fpo| JsonFixedPriceOffering {
                nft_contract_id: fpo.nft_contract_id,
                offeror_id: fpo.offeror_id,
                nft_max_supply: U128(fpo.nft_max_supply),
                price_yocto: U128(fpo.price_yocto),
                nft_metadata: fpo.nft_metadata,
                end_timestamp: fpo.end_timestamp,
                nft_supply_sold: U128(fpo.nft_supply_sold),
            })
            .collect()
    }
}