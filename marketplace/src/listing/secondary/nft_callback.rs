use crate::{
    external::{NftMetadata},
    *,
    listing::secondary::seller::*,
};

use near_sdk::{
    json_types::{U128},
    serde::{Deserialize},
};

#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde")]
struct SecondaryListingNftApprovalMsg {
    pub action: String,
    pub token_metadata: NftMetadata,
    pub buy_now_price_yocto: U128,
    pub min_proposal_price_yocto: Option<U128>, // if None then no proposals will be accepted
    pub start_date: Option<String>,        // nanoseconds since 1970-01-01
    pub end_date: Option<String>,          // nanoseconds since 1970-01-01
}

trait NonFungibleTokenApprovalsReceiver {
    fn nft_on_approve(
        &mut self,
        token_id: NftId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    );
}


#[near_bindgen]
impl NonFungibleTokenApprovalsReceiver for MarketplaceContract {

    fn nft_on_approve(
        &mut self,
        token_id: NftId,
        owner_id: AccountId,
        approval_id: u64,
        msg: String,
    ) {
        let nft_contract_id = env::predecessor_account_id();
        let signer_id = env::signer_account_id();
        //make sure that the signer isn't the predecessor - make sure it's a cross-contract call
        assert_ne!(
            nft_contract_id, signer_id,
            "nft_on_approve should only be called via cross-contract call"
        );

        // make sure the owner ID is the signer.
        assert_eq!(owner_id, signer_id, "owner_id should be signer_id");

        let msg: SecondaryListingNftApprovalMsg =
            near_sdk::serde_json::from_str(&msg).expect("Could not decode approval message");

        if msg.action == "add_listing" {
            self.secondary_listing_add(
                owner_id,
                nft_contract_id,
                approval_id,
                token_id,
                msg.token_metadata,
                msg.buy_now_price_yocto,
                msg.min_proposal_price_yocto,
                msg.start_date,
                msg.end_date
            );
        }
    }
}
