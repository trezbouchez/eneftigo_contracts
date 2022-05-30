
use crate::config::*;
use crate::fpo::config::*;
use crate::FixedPriceOfferingStatus::*;
use crate::internal::*;
use crate::*;
use near_sdk::json_types::{U128};

#[cfg(test)]
#[path = "buyer_tests.rs"]
mod buyer_tests;

#[near_bindgen]
impl MarketplaceContract {

    // purchase at buy now price, provided there's supply
    #[payable]
    pub fn fpo_buy(
        &mut self,
        nft_contract_id: AccountId,
        collection_id: CollectionId,
    ) {
        let offering_id = OfferingId{ nft_contract_id, collection_id };

        // get FPO
        let mut fpo = self.fpos_by_id.get(&offering_id).expect("Could not find NFT listing");

        fpo.update_status();

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
        let attached_balance_yocto = env::attached_deposit();
        assert!(
            attached_balance_yocto >= fpo.buy_now_price_yocto, 
            "Attached Near must be sufficient to pay the price of {:?} yocto Near", 
            fpo.buy_now_price_yocto
        );

        // self.fpo_process_purchase(
        //     fpo.nft_contract_id.clone(),
        //     mint_token_id.to_string(),
        //     buyer_id,
        //     fpo.buy_now_price_yocto
        // );

        nft_contract::mint(
            offering_id.collection_id,
            buyer_id,
            None,
            offering_id.nft_contract_id.clone(),
            1,
            GAS_FOR_NFT_MINT,
        );

        // };

        // TODO: move to resolve
        fpo.supply_left -= 1;
        fpo.prune_supply_exceeding_acceptable_proposals();

        self.fpos_by_id.insert(&offering_id, &fpo);

        // return surplus deposit
        let surplus_deposit = attached_balance_yocto - fpo.buy_now_price_yocto;
        if surplus_deposit > 0 {
            Promise::new(env::predecessor_account_id()).transfer(surplus_deposit);
        }
    }

    // place price proposal
    #[payable]
    pub fn fpo_place_proposal(
        &mut self,
        nft_contract_id: AccountId,
        collection_id: CollectionId,
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
        // self.fpos_by_contract_id.insert(&fpo.nft_contract_id, &fpo);
    }    

    #[payable]
    pub fn fpo_modify_proposal(
        &mut self,
        nft_contract_id: AccountId,
        collection_id: CollectionId,
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

            // process purchase (mint and transfer)
            self.fpo_process_purchase(
                offering_id.clone(),
                predecessor_account_id.clone(),
                fpo.buy_now_price_yocto
            );
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
        collection_id: CollectionId,
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

