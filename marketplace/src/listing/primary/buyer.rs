use crate::{
    constants::*,
    external::nft_contract,
    listing::{
        constants::*,
        primary::{config::*, lib::PrimaryListingIdJson},
        bid::Bid,
        status::ListingStatus,
    },
    *,
};
use near_sdk::{
    json_types::{U128, U64},
    PromiseResult,
};

const NFT_MINT_GAS: Gas = Gas(15_000_000_000_000); // TODO: measure
const NFT_MINT_COMPLETION_GAS: Gas = Gas(5_000_000_000_000); // TODO: measure

// const NFT_MINT_WORST_CASE_STORAGE: u64 = 830;                       // actual, measured

#[cfg(test)]
#[path = "buyer_tests.rs"]
mod buyer_tests;

pub type NftId = String;

#[near_bindgen]
impl MarketplaceContract {
    // purchase at buy now price, provided there's supply
    #[payable]
    pub fn primary_listing_buy(
        &mut self,
        nft_contract_id: AccountId,
        collection_id: U64,
    ) -> Promise {
        let listing_id = PrimaryListingId {
            nft_contract_id,
            collection_id: collection_id.0,
        };

        // update listing status, won't change storage usage
        let mut listing = self
            .primary_listings_by_id
            .get(&listing_id)
            .expect("Could not find NFT listing");
        listing.update_status();
        self.primary_listings_by_id.insert(&listing_id, &listing);

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

        // ensure there's supply left
        assert!(
            listing.supply_left > 0,
            "You are late. All NFTs have been sold."
        );

        // ensure the attached balance is sufficient to pay the price
        let attached_deposit = env::attached_deposit();
        assert!(
            attached_deposit >= price_yocto,
            "Attached deposit of {} is insufficient to pay the price of {}",
            attached_deposit,
            price_yocto
        );

        let storage_byte_cost = env::storage_byte_cost();
        let current_deposit: Balance = self.storage_deposits.get(&buyer_id).unwrap_or(0);
        let nft_worst_case_storage_cost = NFT_MINT_STORAGE_MAX as Balance * storage_byte_cost;
        assert!(
            current_deposit >= nft_worst_case_storage_cost,
            "Your storage deposit is too low. Must be {} yN to process transaction. Please increase your deposit.",
            nft_worst_case_storage_cost
        );

        let listing_id_json = PrimaryListingIdJson {
            nft_contract_id: listing_id.nft_contract_id.clone(),
            collection_id: U64(listing_id.collection_id),
        };

        nft_contract::mint(
            U64(listing_id.collection_id),
            buyer_id,
            None, // perpetual royalties
            listing_id.nft_contract_id.clone(),
            nft_worst_case_storage_cost,
            NFT_MINT_GAS,
        )
        .then(ext_self_nft::primary_listing_buy_now_mint_completion(
            listing.seller_id.clone(),
            attached_deposit,
            price_yocto,
            listing_id_json,
            env::current_account_id(), // we are invoking this function on the current contract
            NO_DEPOSIT,                // don't attach any deposit
            NFT_MINT_COMPLETION_GAS,   // GAS attached to the completion call
        ))
    }

    // place bid
    #[payable]
    pub fn primary_listing_place_bid(
        &mut self,
        nft_contract_id: AccountId,
        collection_id: U64,
        amount_yocto: U128,
    ) -> U64 {
        // TODO: check prepaid gas, terminate early if insufficient

        let collection_id = collection_id.0;
        let amount_yocto = amount_yocto.0;

        let listing_id = PrimaryListingId {
            nft_contract_id,
            collection_id,
        };

        // get listing
        let mut listing = self
            .primary_listings_by_id
            .get(&listing_id)
            .expect("Could not find NFT listing");

        listing.update_status();

        assert!(
            listing.status == ListingStatus::Running,
            "This listing is {}",
            listing.status.as_str()
        );

        let bidder_id = env::predecessor_account_id();
        assert!(
            bidder_id != listing.seller_id,
            "Cannot submit a bid to your own listing"
        );

        // ensure bids are accepted
        let min_bid_yocto = listing
            .min_bid_yocto
            .expect("Bids are not accepted for this listing");

        // ensure there's supply left
        assert!(
            listing.supply_left > 0,
            "You are late. All NFTs have been sold."
        );

        // bid must be lower than buy now, if the latter is set
        if let Some(price_yocto) = listing.price_yocto {
            assert!(
                amount_yocto < price_yocto,
                "Bid must be lower than buy now price of {}",
                price_yocto
            );
        }

        // bid must be multiple of PRICE_STEP_YOCTO
        assert!(
            amount_yocto % BID_STEP_YOCTO == 0,
            "Bid amount must be an integer multple of {} yocto Near",
            BID_STEP_YOCTO
        );

        // get bids vector (was sorted on write) and check if bid is acceptable
        let acceptable_bid_yocto = listing.acceptable_bid_yocto();
        assert!(
            amount_yocto >= acceptable_bid_yocto,
            "Bid is too low. The lowest acceptable amount is {:?}",
            acceptable_bid_yocto
        );

        // ensure the attached balance is sufficient to pay deposit
        let attached_deposit = env::attached_deposit();
        assert!(
            attached_deposit >= amount_yocto,
            "Attached balance must be sufficient to pay the required deposit of {} yocto Near",
            amount_yocto
        );

        // create and add the new bid
        let new_bid = Bid {
            id: listing.next_bid_id,
            bidder_id: bidder_id.clone(),
            amount_yocto: amount_yocto,
        };
        listing.next_bid_id += 1;

        let storage_byte_cost = env::storage_byte_cost();
        let storage_usage_before = env::storage_usage();

        // push to acceptable bids vector, storage is covered by seller reserve
        listing.bids.push(&new_bid);

        // sort acceptable bids
        listing.sort_bids();
        // check if attached deposit is sufficient and compute proposer refund (if any)
        let storage_usage_after = env::storage_usage();
        let storage_usage_added = storage_usage_after - storage_usage_before;
        let storage_cost_added = storage_usage_added as Balance * storage_byte_cost;
        let required_deposit = amount_yocto + storage_cost_added;
        assert!(
            attached_deposit >= required_deposit,
            "Insufficient storage deposit. Please attach at least {}",
            required_deposit
        );
        let refund = attached_deposit - required_deposit;
        if refund > 0 {
            Promise::new(bidder_id).transfer(refund);
        }

        self.primary_listing_remove_supply_exceeding_bids_and_refund_bidders(&mut listing);

        self.primary_listings_by_id.insert(&listing_id, &listing);

        U64(new_bid.id)
    }


    #[payable]
    /*    pub fn primary_listing_modify_proposal(
            &mut self,
            nft_contract_id: AccountId,
            collection_id: U64,
            proposal_id: U64,
            price_yocto: U128,
        ) {
    /       let listing_id = PrimaryListingId { nft_contract_id, collection_id };

            // get listing
            let mut listing = self.primary_listing_by_id.get(&listing_id).expect("Could not find NFT listing");

            listing.update_status();

            assert!(
                listing.status == Running,
                "This listing is {}",
                listing.status.as_str()
            );

            // ensure proposals are accepted
            assert!(
                listing.min_proposal_price_yocto.is_some(),
                "Proposals are not accepted for this listing"
            );

            let price_yocto = price_yocto.0;

            // price must be multiple of PRICE_STEP_YOCTO
            assert!(
                price_yocto % PRICE_STEP_YOCTO == 0,
                "Price must be an integer multple of {} yocto Near",
                PRICE_STEP_YOCTO
            );

            // check if there is a prior proposal from this account
            let predecessor_account_id = env::predecessor_account_id();
            let predecessors_proposals = listing.proposals_by_proposer.get(&predecessor_account_id).expect("No prior proposal from this account");
            assert!(
                predecessors_proposals.contains(&proposal_id),
                "Proposal with ID {} from account {} not found",
                proposal_id, predecessor_account_id
            );

            // ensure the attached balance is sufficient to cover higher required deposit
            let mut proposal = listing.proposals.get(&proposal_id).expect("Could not find proposal");
            let deposit_supplement_yocto = price_yocto - proposal.price_yocto;
            let attached_balance_yocto = env::attached_deposit();
            assert!(
                attached_balance_yocto >= deposit_supplement_yocto,
                "Attached balance must be sufficient to pay the required deposit supplement of {} yocto Near",
                deposit_supplement_yocto
            );

            // if price is >= buy_now_price_yocto then accept right away, terminate early to save gas
            if price_yocto >= listing.buy_now_price_yocto {
                // TODO: this does not work correctly
                // remove from acceptable_proposals (if there), proposals and proposals_by_proposer
                let index_of_this_in_acceptable_proposals = listing.acceptable_proposals
                .iter()
                .position(|acceptable_proposal_id| acceptable_proposal_id == proposal_id);
                if let Some(index_of_this_in_acceptable_proposals) = index_of_this_in_acceptable_proposals {
                    let mut acceptable_proposals_vec = listing.acceptable_proposals.to_vec();
                    acceptable_proposals_vec.remove(index_of_this_in_acceptable_proposals);
                    listing.acceptable_proposals.clear();
                    listing.acceptable_proposals.extend(acceptable_proposals_vec);
                }
                // TODO: modify to correctly handle deposit removal
                // listing.proposals.remove(&proposal_id);
                listing.proposals.remove(&proposal_id).expect("Could not remove proposal");
                let mut proposals_by_this_proposer = listing.proposals_by_proposer.get(&predecessor_account_id).expect("Could not find proposal from this account.");
                proposals_by_this_proposer.remove(&proposal_id);
                if proposals_by_this_proposer.is_empty() {
                    listing.proposals_by_proposer.remove(&predecessor_account_id);
                } else {
                    listing.proposals_by_proposer.insert(&predecessor_account_id, &proposals_by_this_proposer);
                }


                // return surplus deposit
                let surplus_deposit = attached_balance_yocto + proposal.price_yocto - listing.buy_now_price_yocto;
                if surplus_deposit > 0 {
                    if surplus_deposit > 0 {
                        Promise::new(predecessor_account_id).transfer(surplus_deposit);
                    }
                }
                // update supply_left
                listing.supply_left -= 1;

                self.primary_listings_by_id.insert(&listing_id, &listing);

                return;
            }

            // check if proposed price is acceptable
            let acceptable_price_yocto = listing.acceptable_price_yocto();
            assert!(
                price_yocto >= acceptable_price_yocto,
                "The minimum acceptable price is {} yoctoNear",
                acceptable_price_yocto
           );

            // update proposal - set price and mark acceptable, store
            proposal.price_yocto = price_yocto;
            // proposal.is_acceptable = true;

            listing.proposals.insert(&proposal_id, &proposal);

            // if the proposal is among the acceptable ones we'll just re-sort
            // otherwise we need to outbid the lowers-priced proposal
            if !listing.is_proposal_acceptable(proposal_id) {
                // here we assume that it used to be a acceptable one when was first submitted
                // (otherwise it'd have been rejected in the first place) and got outbid at some
                // point - this, in turn, means that the proposal count equals or exceeds the supply
                // so we can just replace the first acceptable proposal (worst price) with this one
                let outbid_proposal_id = listing.acceptable_proposals.replace(0, &proposal_id);
                let mut outbid_proposal = listing.proposals.get(&outbid_proposal_id).expect("Outbid proposal is missing, inconsistent state");

                // TODO:
                // Implement it in another way
                // outbid_proposal.mark_unacceptable_and_refund_deposit();
                listing.proposals.insert(&outbid_proposal_id, &outbid_proposal);
            }

            listing.sort_acceptable_proposals();

            self.primary_listings_by_id.insert(&listing_id, &listing);

            // return surplus deposit
            let surplus_deposit = attached_balance_yocto - deposit_supplement_yocto;
            if surplus_deposit > 0 {
                Promise::new(env::predecessor_account_id()).transfer(surplus_deposit);
            }
        }*/

    pub fn primary_listing_revoke_bid(
        &mut self,
        nft_contract_id: AccountId,
        collection_id: U64,
        bid_id: U64,
    ) {
        let collection_id = collection_id.0;
        let bid_id = bid_id.0;

        let listing_id = PrimaryListingId {
            nft_contract_id,
            collection_id,
        };

        // get listing
        let mut listing = self
            .primary_listings_by_id
            .get(&listing_id)
            .expect("Could not find NFT listing");

        listing.update_status();
        assert!(
            listing.status == ListingStatus::Running,
            "This listing is {}",
            listing.status.as_str()
        );
        // ensure bids are accepted
        assert!(
            listing.min_bid_yocto.is_some(),
            "Bids are not accepted for this listing"
        );

        let storage_before = env::storage_usage();

        let index = listing
            .bids
            .iter()
            .position(|bid| bid.id == bid_id)
            .expect("Could not find bid");
        let removed_bid = listing.bids.swap_remove(index as u64);
        assert!(
            removed_bid.bidder_id == env::predecessor_account_id(),
            "Not authorized to revoke this bid"
        );
        listing.sort_bids();

        let storage_after = env::storage_usage();
        let storage_freed = storage_before - storage_after;
        let storage_refund = storage_freed as Balance * env::storage_byte_cost();

        // store
        self.primary_listings_by_id.insert(&listing_id, &listing);

        // return deposit minus penalty
        let fee = removed_bid.amount_yocto * PROPOSAL_REVOKE_FEE_RATE / 100;
        Promise::new(env::predecessor_account_id())
            .transfer(removed_bid.amount_yocto + storage_refund - fee);

        // transfer penalty to Eneftigo profit account
        Promise::new(self.fees_account_id()).transfer(fee);
    }
}

// If extra fields get added to the NFT metadata this will need to be updated
// fn nft_mint_worst_case_storage(receiver_id: AccountId) -> u64 {
//     let mint_worst_case_storage_base: u64 = 830;        // actual, measured
//     mint_worst_case_storage_base + receiver_id.to_string().len() as u64 * 2
// }

#[ext_contract(ext_self_nft)]
trait PrimaryListingBuyerCallback {
    fn primary_listing_buy_now_mint_completion(
        &mut self,
        seller_id: AccountId,
        attached_deposit: Balance,
        price: Balance,
        listing_id: PrimaryListingIdJson,
    ) -> (NftId, Balance);
}

trait PrimaryListingBuyerCallback {
    fn primary_listing_buy_now_mint_completion(
        &mut self,
        seller_id: AccountId,
        attached_deposit: Balance,
        price: Balance,
        listing_id: PrimaryListingIdJson,
    ) -> (NftId, Balance);
}

#[near_bindgen]
impl PrimaryListingBuyerCallback for MarketplaceContract {
    #[private]
    fn primary_listing_buy_now_mint_completion(
        &mut self,
        seller_id: AccountId,
        attached_deposit: Balance,
        price: Balance,
        listing_id: PrimaryListingIdJson,
    ) -> (NftId, Balance) {
        let listing_id = PrimaryListingId {
            nft_contract_id: listing_id.nft_contract_id,
            collection_id: listing_id.collection_id.0,
        };

        // Here the attached_deposit is the deposit attach buy buyer to the marketplace call (like buy_now)
        // The price is the amount due to be transferred to the seller's account if minting succeeds
        // Pruning the bids will return deposit provided by respective proposers
        assert_eq!(env::promise_results_count(), 1, "Too many data receipts");
        let mint_result = env::promise_result(0);
        match mint_result {
            PromiseResult::NotReady | PromiseResult::Failed => {
                if attached_deposit > 0 {
                    Promise::new(env::signer_account_id()).transfer(attached_deposit);
                }
                panic!("NFT mint failed");
            }
            PromiseResult::Successful(val) => {
                let buyer_id = env::signer_account_id();
                // here the NFT was minted and transferred so we pay the seller before we can panic
                // so that at least this part of the transaction is ok
                Promise::new(seller_id).transfer(price);
                // update listing supply, changing supply_left won't affect the storage so we don't
                // need to update seller's storage deposit
                let mut listing = self
                    .primary_listings_by_id
                    .get(&listing_id)
                    .expect("Could not find NFT listing");
                listing.supply_left -= 1;
                self.primary_listing_remove_supply_exceeding_bids_and_refund_bidders(
                    &mut listing,
                );
                self.primary_listings_by_id.insert(&listing_id, &listing);
                // get the token ID and NFT storage and update buyer storage deposit
                let (token_id, mint_storage_bytes) =
                    near_sdk::serde_json::from_slice::<(NftId, U64)>(&val)
                        .expect("NFT mint returned unexpected value.");
                let mint_storage_cost = mint_storage_bytes.0 as Balance * env::storage_byte_cost();
                let current_deposit = self
                    .storage_deposits
                    .get(&buyer_id)
                    .expect("Could not find buyer's storage deposit record");
                // this should never happen. when it does to be totally correct we should revert the minting
                // and seller payment but it's water under the bridge now. to avoid it we pessimistically
                // compute the storage cost at the beginning of the primary_listing_buy contract call
                let updated_deposit = if current_deposit >= mint_storage_cost {
                    current_deposit - mint_storage_cost
                } else {
                    0 // should never happen, TODO: log a warning to review deposit logic?
                };
                self.storage_deposits.insert(&buyer_id, &updated_deposit);

                (token_id, updated_deposit)
            }
        }
    }
}

#[allow(dead_code)]
fn nft_mint_storage(title: &str, media_url: &str, receiver_id: &str) -> u64 {
    // 1013 + 128 + 2048 + 2*64 =
    return 1013 + title.len() as u64 + media_url.len() as u64 + 2u64 * receiver_id.len() as u64;
}
