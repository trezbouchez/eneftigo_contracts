
use crate::config::*;
use crate::fpo::config::*;
use crate::FixedPriceOfferingStatus::*;
use crate::internal::*;
use crate::*;
use near_sdk::{
    PromiseResult,
    json_types::{U128},
};

const NFT_MINT_GAS: Gas = Gas(15_000_000_000_000);                  // TODO: measure
const NFT_MINT_COMPLETION_GAS: Gas = Gas(5_000_000_000_000);        // TODO: measure

const NFT_MINT_WORST_CASE_STORAGE: u64 = 830;                       // actual, measured


#[cfg(test)]
#[path = "buyer_tests.rs"]
mod buyer_tests;

pub type NftId = String;

#[near_bindgen]
impl MarketplaceContract {

    // purchase at buy now price, provided there's supply
    #[payable]
    pub fn fpo_buy(
        &mut self,
        nft_contract_id: AccountId,
        collection_id: NftCollectionId,
    ) -> Promise {
        let offering_id = OfferingId{ nft_contract_id, collection_id };
        // update FPO status, won't change storage usage
        let mut fpo = self.fpos_by_id.get(&offering_id).expect("Could not find NFT listing");
        fpo.update_status();
        self.fpos_by_id.insert(&offering_id, &fpo);

        assert!(
            fpo.status == Running,
            "This offering is {}",
            fpo.status.as_str()
        );

        let buyer_id = env::predecessor_account_id();
        assert!(buyer_id != fpo.offeror_id, "Cannot buy from yourself");

        // ensure there's supply left
        assert!(
            fpo.supply_left > 0,
            "You are late. All NFTs have been sold."
        );

        // ensure the attached balance is sufficient
        let attached_deposit = env::attached_deposit();
        let nft_storage_cost = NFT_MINT_WORST_CASE_STORAGE as Balance * env::storage_byte_cost();
        let price = fpo.buy_now_price_yocto;
        let total_cost = price + nft_storage_cost;

        assert!(
            attached_deposit >= total_cost, 
            "Attached Near must be at least {}, enough to pay the price and the NFT minting storage", 
            total_cost,
        );

        nft_contract::mint(
            offering_id.collection_id,
            buyer_id,
            None,               // perpetual royalties
            offering_id.nft_contract_id.clone(),
            nft_storage_cost,
            NFT_MINT_GAS,
        )
        .then(ext_self_nft::fpo_buy_now_mint_completion(
            fpo.offeror_id.clone(),
            attached_deposit,
            price,
            offering_id.clone(),
            env::current_account_id(), // we are invoking this function on the current contract
            NO_DEPOSIT,                // don't attach any deposit
            NFT_MINT_COMPLETION_GAS, // GAS attached to the completion call
        ))
    }

    // place price proposal
    #[payable]
    pub fn fpo_place_proposal(
        &mut self,
        nft_contract_id: AccountId,
        collection_id: NftCollectionId,
        price_yocto: U128,
    ) -> ProposalId {
        let offering_id = OfferingId{ nft_contract_id, collection_id };

        // get FPO
        let mut fpo = self.fpos_by_id.get(&offering_id).expect("Could not find NFT listing");

        fpo.update_status();

        assert!(
            fpo.status == Running,
            "This offering is {}",
            fpo.status.as_str()
        );

        let proposer_id = env::predecessor_account_id();
        assert!(proposer_id != fpo.offeror_id, "Cannot submit a proposal to your own offering");

        // ensure proposals are accepted
        assert!(
            fpo.min_proposal_price_yocto.is_some(),
            "Proposals are not accepted for this offering"
        );

        // ensure there's supply left
        assert!(
            fpo.supply_left > 0,
            "You are late. All NFTs have been sold."
        );

        let price_yocto = price_yocto.0;

        // price must be lower than buy now
        assert!(
            price_yocto < fpo.buy_now_price_yocto,
            "Proposed price must be lower than buy now price of {}",
            fpo.buy_now_price_yocto
        );

        // price must be multiple of PRICE_STEP_YOCTO
        assert!(
            price_yocto % PRICE_STEP_YOCTO == 0,
            "Price must be an integer multple of {} yocto Near", 
            PRICE_STEP_YOCTO
        );
        
        // get proposals vector (was sorted on write) and check if proposed price is acceptable
        let acceptable_price = fpo.acceptable_price_yocto();
        assert!(
            price_yocto >= acceptable_price,
            "Proposed price is too low. The lowest acceptable price is {:?}",
            acceptable_price
        );    
        
        // ensure the attached balance is sufficient to pay deposit
        // TODO: should we adopt approvals instead?
        let attached_balance_yocto = env::attached_deposit();
        assert!(
            attached_balance_yocto >= price_yocto, 
            "Attached balance must be sufficient to pay the required deposit of {} yocto Near", 
            price_yocto 
        );

        // register proposal
        let new_proposal = FixedPriceOfferingProposal {
            id: fpo.next_proposal_id,
            proposer_id: proposer_id,
            price_yocto: price_yocto,
            is_acceptable: true,
        };
        fpo.next_proposal_id += 1;

        fpo.proposals.insert(&new_proposal.id, &new_proposal);

        let mut proposals_by_proposer_set = fpo.proposals_by_proposer.get(&new_proposal.proposer_id).unwrap_or_else(|| {
            let offering_id_hash = hash_offering_id(&offering_id);
            let proposer_id_hash = hash_account_id(&new_proposal.proposer_id);
                UnorderedSet::new(
                    FixedPriceOfferingStorageKey::ProposalsByProposerInner { offering_id_hash, proposer_id_hash }.try_to_vec().unwrap()
                )
        });
        proposals_by_proposer_set.insert(&new_proposal.id);
        fpo.proposals_by_proposer.insert(&new_proposal.proposer_id, &proposals_by_proposer_set);

        let unmatched_supply_exists = fpo.acceptable_proposals.len() < fpo.supply_left;
        if unmatched_supply_exists {
            fpo.acceptable_proposals.push(&new_proposal.id);
        } else {
            let outbid_proposal_id = fpo.acceptable_proposals.replace(0, &new_proposal.id);
            let mut outbid_proposal = fpo.proposals.get(&outbid_proposal_id).expect("Outbid proposal is missing, inconsistent state");
            outbid_proposal.mark_unacceptable_and_refund_deposit();
            fpo.proposals.insert(&outbid_proposal_id, &outbid_proposal);
        }

        fpo.sort_acceptable_proposals();

        self.fpos_by_id.insert(&offering_id, &fpo);

        // return surplus deposit
        let surplus_deposit = attached_balance_yocto - price_yocto;
        if surplus_deposit > 0 {
            Promise::new(env::predecessor_account_id()).transfer(surplus_deposit);
        }
        
        new_proposal.id
    }    

    #[payable]
    pub fn fpo_modify_proposal(
        &mut self,
        nft_contract_id: AccountId,
        collection_id: NftCollectionId,
        proposal_id: ProposalId,
        price_yocto: U128,
    ) {
        let offering_id = OfferingId { nft_contract_id, collection_id };
        
        // get FPO
        let mut fpo = self.fpos_by_id.get(&offering_id).expect("Could not find NFT listing");

        fpo.update_status();
        
        assert!(
            fpo.status == Running,
            "This offering is {}",
            fpo.status.as_str()
        );
        
        // ensure proposals are accepted
        assert!(
            fpo.min_proposal_price_yocto.is_some(),
            "Proposals are not accepted for this offering"
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
        let predecessors_proposals = fpo.proposals_by_proposer.get(&predecessor_account_id).expect("No prior proposal from this account");
        assert!(
            predecessors_proposals.contains(&proposal_id),
            "Proposal with ID {} from account {} not found",
            proposal_id, predecessor_account_id
        );

        // ensure the attached balance is sufficient to cover higher required deposit
        let mut proposal = fpo.proposals.get(&proposal_id).expect("Could not find proposal");
        let deposit_supplement_yocto = price_yocto - proposal.price_yocto;
        let attached_balance_yocto = env::attached_deposit();
        assert!(
            attached_balance_yocto >= deposit_supplement_yocto, 
            "Attached balance must be sufficient to pay the required deposit supplement of {} yocto Near", 
            deposit_supplement_yocto
        );

        // if price is >= buy_now_price_yocto then accept right away, terminate early to save gas
        if price_yocto >= fpo.buy_now_price_yocto {
            // remove from acceptable_proposals (if there), proposals and proposals_by_proposer
            let index_of_this_in_acceptable_proposals = fpo.acceptable_proposals
            .iter()
            .position(|acceptable_proposal_id| acceptable_proposal_id == proposal_id);
            if let Some(index_of_this_in_acceptable_proposals) = index_of_this_in_acceptable_proposals {
                let mut acceptable_proposals_vec = fpo.acceptable_proposals.to_vec();
                acceptable_proposals_vec.remove(index_of_this_in_acceptable_proposals);
                fpo.acceptable_proposals.clear();
                fpo.acceptable_proposals.extend(acceptable_proposals_vec);
            }
            fpo.proposals.remove(&proposal_id).expect("Could not remove proposal");
            let mut proposals_by_this_proposer = fpo.proposals_by_proposer.get(&predecessor_account_id).expect("Could not find proposal from this account.");
            proposals_by_this_proposer.remove(&proposal_id);
            if proposals_by_this_proposer.is_empty() {
                fpo.proposals_by_proposer.remove(&predecessor_account_id);
            } else {
                fpo.proposals_by_proposer.insert(&predecessor_account_id, &proposals_by_this_proposer);
            }

            // TODO:
/*            nft_contract::mint(
                offering_id.collection_id,
                predecessor_account_id,
                None,               // perpetual royalties
                offering_id.nft_contract_id.clone(),
                deposit_left,       // should be >= 7_060_000_000_000_000_000_000 yN
                NFT_MINT_GAS,
            )
            .then(ext_self_nft::fpo__mint_completion(
                fpo.offeror_id,
                attached_deposit,
                price,
                offering_id,
                env::current_account_id(), // we are invoking this function on the current contract
                NO_DEPOSIT,                // don't attach any deposit
                NFT_MINT_COMPLETION_GAS, // GAS attached to the completion call
            ));*/
            // process purchase (mint and transfer)
            // self.fpo_process_purchase(
            //     offering_id.clone(),
            //     predecessor_account_id.clone(),
            //     fpo.buy_now_price_yocto
            // );

            // return surplus deposit
            let surplus_deposit = attached_balance_yocto + proposal.price_yocto - fpo.buy_now_price_yocto;
            if surplus_deposit > 0 {
                if surplus_deposit > 0 {
                    Promise::new(predecessor_account_id).transfer(surplus_deposit);
                }
            }
            // update supply_left
            fpo.supply_left -= 1;

            self.fpos_by_id.insert(&offering_id, &fpo);

            return;
        }

        // check if proposed price is acceptable
        let acceptable_price_yocto = fpo.acceptable_price_yocto();
        assert!(
            price_yocto >= acceptable_price_yocto,
            "The minimum acceptable price is {} yoctoNear",
            acceptable_price_yocto
       );
                
        // update proposal - set price and mark acceptable, store
        proposal.price_yocto = price_yocto;
        proposal.is_acceptable = true;

        fpo.proposals.insert(&proposal_id, &proposal);

        // if the proposal is among the acceptable ones we'll just re-sort
        // otherwise we need to outbid the lowers-priced proposal
        if !fpo.is_proposal_acceptable(proposal_id) {
            // here we assume that it used to be a acceptable one when was first submitted
            // (otherwise it'd have been rejected in the first place) and got outbid at some
            // point - this, in turn, means that the proposal count equals or exceeds the supply
            // so we can just replace the first acceptable proposal (worst price) with this one
            let outbid_proposal_id = fpo.acceptable_proposals.replace(0, &proposal_id);
            let mut outbid_proposal = fpo.proposals.get(&outbid_proposal_id).expect("Outbid proposal is missing, inconsistent state");
            outbid_proposal.mark_unacceptable_and_refund_deposit();
            fpo.proposals.insert(&outbid_proposal_id, &outbid_proposal);
        }
        
        fpo.sort_acceptable_proposals();

        self.fpos_by_id.insert(&offering_id, &fpo);

        // return surplus deposit
        let surplus_deposit = attached_balance_yocto - deposit_supplement_yocto;
        if surplus_deposit > 0 {
            Promise::new(env::predecessor_account_id()).transfer(surplus_deposit);
        }
    }

    pub fn fpo_revoke_proposal(
        &mut self,
        nft_contract_id: AccountId,
        collection_id: NftCollectionId,
        proposal_id: ProposalId
    ) {
        let offering_id = OfferingId { nft_contract_id, collection_id };

        // get FPO
        let mut fpo = self.fpos_by_id.get(&offering_id).expect("Could not find NFT listing");

        fpo.update_status();
        
        assert!(
            fpo.status == Running,
            "This offering is {}",
            fpo.status.as_str()
        );
        
        // ensure proposals are accepted
        assert!(
            fpo.min_proposal_price_yocto.is_some(),
            "Proposals are not accepted for this offering"
        );

        // check if there exists a proposal from this proposer
        let predecessor_account_id = env::predecessor_account_id();
        let predecessors_proposals = fpo.proposals_by_proposer.get(&predecessor_account_id).expect("No prior proposal from this account");
        assert!(
            predecessors_proposals.contains(&proposal_id),
            "Proposal with ID {} from account {} not found",
            proposal_id, predecessor_account_id
        );

        // check if the proposal is still acceptable
        let acceptable_proposal_index = fpo.acceptable_proposals.iter().position(|acceptable_proposal_id| acceptable_proposal_id == proposal_id).expect("This proposal has been outbid. The deposit has been returned");

        // remove from acceptable_proposals, no sorting of acceptable_proposals is required
        let mut acceptable_proposals_vec = fpo.acceptable_proposals.to_vec();
        let _removed_acceptable_proposal_id = acceptable_proposals_vec.remove(acceptable_proposal_index);
        fpo.acceptable_proposals.clear();
        fpo.acceptable_proposals.extend(acceptable_proposals_vec);

        // remove from proposals
        let removed_proposal = fpo.proposals.remove(&proposal_id).expect("Could not find proposal");

        // remove from proposals_by_proposer
        let mut proposals_by_predecessor = fpo.proposals_by_proposer.get(&predecessor_account_id).expect("This account has not submitted any proposals");
        let was_removed_from_proposer_proposals = proposals_by_predecessor.remove(&proposal_id);
        assert!(
            was_removed_from_proposer_proposals,
            "Could not find it among proposals submitted by this account"
        );
        if proposals_by_predecessor.is_empty() {
            fpo.proposals_by_proposer.remove(&predecessor_account_id);
        } else {
            fpo.proposals_by_proposer.insert(&predecessor_account_id, &proposals_by_predecessor);
        }

        // store
        self.fpos_by_id.insert(&offering_id, &fpo);

        // return deposit minus penalty
        let penalty = removed_proposal.price_yocto * PROPOSAL_REVOKE_PENALTY_RATE / 100;
        Promise::new(env::predecessor_account_id()).transfer(removed_proposal.price_yocto - penalty);

        // transfer penalty to Eneftigo profit account
        Promise::new(ENEFTIGO_PROFIT_ACCOUNT_ID.parse().unwrap()).transfer(penalty);
    }
}

// If extra fields get added to the NFT metadata this will need to be updated
fn nft_mint_worst_case_storage(receiver_id: AccountId) -> u64 {
    let mint_worst_case_storage_base: u64 = 830;        // actual, measured
    mint_worst_case_storage_base + receiver_id.to_string().len() as u64 * 2
}

#[ext_contract(ext_self_nft)]
trait FPOBuyerCallback {
    fn fpo_buy_now_mint_completion(
        &mut self,
        seller_id: AccountId,
        attached_deposit: Balance,
        price: Balance,
        offering_id: OfferingId,
    ) -> NftId;
}

trait FPOBuyerCallback {
    fn fpo_buy_now_mint_completion(
        &mut self,
        seller_id: AccountId,
        attached_deposit: Balance,
        price: Balance,
        offering_id: OfferingId,
    ) -> NftId;
}

#[near_bindgen]
impl FPOBuyerCallback for MarketplaceContract {
    #[private]
    fn fpo_buy_now_mint_completion(
        &mut self,
        seller_id: AccountId,
        attached_deposit: Balance,
        price: Balance,
        offering_id: OfferingId,
    ) -> NftId {
        // Here the attached_deposit is the deposit attach buy buyer to the marketplace call (like buy_now)
        // The price is the amount due to be transferred to the seller's account if minting succeeds
        // Pruning the proposals will return deposit provided by respective proposers
        assert_eq!(env::promise_results_count(), 1, "Too many data receipts");
        match env::promise_result(0) {
            PromiseResult::NotReady | PromiseResult::Failed => {
                let refund = attached_deposit;
                if refund > 0 {
                    Promise::new(env::signer_account_id()).transfer(refund);
                }
                let mut fpo = self.fpos_by_id.get(&offering_id)
                    .expect("Could not find the offering");
                fpo.supply_left += 1; // supply was decremented before attempting to mint
                self.fpos_by_id.insert(&offering_id, &fpo);
                panic!("NFT mint failed");
            }
            PromiseResult::Successful(val) => {
                // here the NFT was minted and transferred so we pay the seller before we can panic
                // so that at least this part of the transaction is ok
                Promise::new(seller_id).transfer(price);
                // update offering supply
                let mut fpo = self.fpos_by_id.get(&offering_id).expect("Could not find NFT listing");
                fpo.supply_left -= 1;
                fpo.prune_supply_exceeding_acceptable_proposals();
                self.fpos_by_id.insert(&offering_id, &fpo);
                // get the token ID and NFT storage and compute refund
                let (token_id, mint_storage_bytes) =
                    near_sdk::serde_json::from_slice::<(NftId, u64)>(&val)
                        .expect("NFT mint returned unexpected value.");
                let mint_storage_cost = mint_storage_bytes as Balance * env::storage_byte_cost();
                let total_storage_cost  = price + mint_storage_cost;
                // this should never happen. when it does to be totally correct we should revert the minting
                // and seller payment but it's water under the bridge now. to avoid it we pessimistically 
                // compute the storage cost at the beginning of the fpo_buy contract call 
                assert!(attached_deposit >= total_storage_cost, "Attached deposit won't cover the price and NFT storage");
                let refund = attached_deposit - total_storage_cost;
                if refund > 0 {
                    Promise::new(env::signer_account_id()).transfer(refund);
                }
                // let token_id_str = format!("ALL OK, NFT minted with TokenId {}", token_id);
                // env::log_str(&token_id_str);

                token_id
            }
        }
    }
}
