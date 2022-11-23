use crate::{
    constants::*,
    external::nft_contract,
    listing::{
        // constants::*, 
        // primary::lib::PrimaryListingIdJson, 
        // bid::Bid, 
        status::ListingStatus,
    },
    *,
};
use near_sdk::{
    // env::attached_deposit,
    json_types::{U128},
    PromiseResult,
};

const NFT_TRANSFER_GAS: Gas = Gas(15_000_000_000_000); // TODO: measure
const NFT_TRANSFER_COMPLETION_GAS: Gas = Gas(5_000_000_000_000); // TODO: measure

#[cfg(test)]
#[path = "buyer_tests.rs"]
mod buyer_tests;

pub type NftId = String;

#[near_bindgen]
impl MarketplaceContract {
    #[payable]
    pub fn secondary_listing_buy(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
    ) -> Promise {
        let listing_id = SecondaryListingId {
            nft_contract_id: nft_contract_id.clone(),
            token_id: token_id.clone(),
        };

        let mut listing = self
            .secondary_listings_by_id
            .get(&listing_id)
            .expect("Could not find NFT listing");
        listing.update_status();
        self.secondary_listings_by_id.insert(&listing_id, &listing);

        // make sure buy now is possible
        let price_yocto = listing
            .price_yocto
            .expect("Buy Now is not possible for this listing");

        assert!(
            listing.status == ListingStatus::Running,
            "This listing is {}",
            listing.status.as_str()
        );

        let buyer_id = env::predecessor_account_id();
        assert!(buyer_id != listing.seller_id, "Cannot buy from yourself");

        // ensure the attached balance is sufficient to pay the price and nft_transfer 1yN required deposit
        let required_deposit = price_yocto + 1;
        let attached_deposit = env::attached_deposit();
        assert!(
            attached_deposit >= required_deposit,
            "Attached deposit of {} is insufficient to pay the price of {} and 1yN deposit for NFT transfer",
            attached_deposit,
            required_deposit,
        );

        nft_contract::nft_transfer(
            buyer_id,
            token_id.clone(),
            Some(listing.approval_id),
            None,
            nft_contract_id.clone(),
            1,
            NFT_TRANSFER_GAS,
        )
        .then(ext_self_nft::nft_transfer_completion(
            nft_contract_id,
            token_id,
            listing.seller_id.clone(),
            U128(attached_deposit),
            U128(price_yocto),
            env::current_account_id(), // we are invoking this function on the current contract
            NO_DEPOSIT,                // don't attach any deposit
            NFT_TRANSFER_COMPLETION_GAS, // GAS attached to the completion call
        ))
    }
}

#[ext_contract(ext_self_nft)]
trait SecondaryListingBuyerCallback {
    fn nft_transfer_completion(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        seller_id: AccountId,
        attached_deposit: U128,
        price_yocto: U128,
    );
}

trait SecondaryListingBuyerCallback {
    fn nft_transfer_completion(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        seller_id: AccountId,
        attached_deposit: U128,
        price_yocto: U128,
    );
}

#[near_bindgen]
impl SecondaryListingBuyerCallback for MarketplaceContract {
    #[private]
    fn nft_transfer_completion(
        &mut self,
        nft_contract_id: AccountId,
        token_id: String,
        seller_id: AccountId,
        attached_deposit: U128,
        price_yocto: U128,
    ) {
        let attached_deposit = attached_deposit.0;
        let price_yocto = price_yocto.0;

        let listing_id = SecondaryListingId {
            nft_contract_id,
            token_id,
        };

        // Here the attached_deposit is the deposit attach buy buyer to the marketplace call (like buy_now)
        // The price is the amount due to be transferred to the seller's account if transfer succeeds
        // Pruning the proposals will return deposit provided by respective proposers
        assert_eq!(env::promise_results_count(), 1, "Too many data receipts");
        let nft_transfer_result = env::promise_result(0);
        match nft_transfer_result {
            PromiseResult::NotReady | PromiseResult::Failed => {
                if attached_deposit > 0 {
                    Promise::new(env::signer_account_id()).transfer(attached_deposit);
                }
                panic!("nft_transfer failed");
            }
            PromiseResult::Successful(val) => {
                let buyer_id = env::signer_account_id();

                // transfer price to the seller
                Promise::new(seller_id.clone()).transfer(price_yocto);

                // remove listing and return storage deposit to seller
                let storage_before = env::storage_usage();
                let removed_listing = self
                    .secondary_listings_by_id
                    .remove(&listing_id)
                    .expect("Could not remove listing: Could not find listing");
                let storage_after = env::storage_usage();
                let storage_byte_cost: Balance = env::storage_byte_cost();
                let deposit_refund =
                    (storage_before - storage_after) as Balance * storage_byte_cost;
                if deposit_refund > 0 {
                    let current_deposit = self
                        .storage_deposits
                        .get(&seller_id)
                        .expect("Could not find seller's storage deposit record");
                    let updated_deposit = current_deposit + deposit_refund;
                    self.storage_deposits.insert(&seller_id, &updated_deposit);
                }

                // TODO: refund proposals (if any) and remove listing
                // self.secondary_listing_remove_supply_exceeding_proposals_and_refund_proposers(&mut removed_listing);

                // return excess attached deposit
                let required_deposit = price_yocto + 1; // 1yN required by nft_transfer
                if attached_deposit > required_deposit {
                    Promise::new(buyer_id).transfer(attached_deposit - required_deposit);
                }
            }
        }
    }
}

#[allow(dead_code)]
fn nft_mint_storage(title: &str, media_url: &str, receiver_id: &str) -> u64 {
    // 1013 + 128 + 2048 + 2*64 =
    return 1013 + title.len() as u64 + media_url.len() as u64 + 2u64 * receiver_id.len() as u64;
}
