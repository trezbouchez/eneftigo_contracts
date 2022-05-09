
use crate::fpo::config::*;
use crate::FixedPriceOfferingStatus::*;
use crate::internal::*;
use crate::*;

#[near_bindgen]
impl MarketplaceContract {

    // purchase at buy now price, provided there's supply
    #[payable]
    pub fn fpo_buy(
        &mut self,
        nft_contract_id: AccountId,
    ) {
        // make sure it's called by marketplace 
        // TODO: should this hold? shouldn't we allow anyone?
        assert_eq!(
            &env::predecessor_account_id(),
            &self.owner_id,
            "Only Eneftigo Marketplace owner can place order."
        );

        // get FPO
        let fpo = &mut self.fpos_by_contract_id.get(&nft_contract_id).expect("Could not find NFT listing");

        fpo.update_status();

        assert!(
            fpo.status == Running,
            "This offering is {}",
            fpo.status.as_str()
        );

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

        // process the purchase
        let buyer_id = env::predecessor_account_id();

        self.fpo_process_purchase(
            fpo.nft_contract_id.clone(),
            fpo.supply_left.to_string(),
            buyer_id,
        );

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
        price_yocto: u128,
    ) {
        // get FPO
        let fpo = &mut self.fpos_by_contract_id.get(&nft_contract_id).expect("Could not find NFT listing");

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

        // ensure there's supply left
        assert!(
            fpo.supply_left > 0,
            "You are late. All NFTs have been sold."
        );

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
        let deposit_yocto = price_yocto * PROPOSAL_DEPOSIT_RATE / 100;
        assert!(
            attached_balance_yocto >= deposit_yocto, 
            "Attached balance must be sufficient to pay the required {:?}% deposit ({:?} yocto Near)", 
            PROPOSAL_DEPOSIT_RATE, 
            deposit_yocto 
        );

        // register proposal
        let proposer_id = env::predecessor_account_id();
        let new_proposal = FixedPriceOfferingProposal {
            id: fpo.next_proposal_id,
            proposer_id: proposer_id,
            price_yocto: price_yocto,
            is_acceptable: true,
        };
        fpo.next_proposal_id += 1;

        fpo.proposals.insert(&new_proposal.id, &new_proposal);

        let mut proposals_by_proposer_set = fpo.proposals_by_proposer.get(&new_proposal.proposer_id).unwrap_or_else(|| {
            let nft_contract_id_hash = hash_account_id(&nft_contract_id);
            let proposer_id_hash = hash_account_id(&new_proposal.proposer_id);
                UnorderedSet::new(
                    FixedPriceOfferingStorageKey::ProposalsByProposerInner {
                        nft_contract_id_hash: nft_contract_id_hash,
                        proposer_id_hash: proposer_id_hash,
                    }.try_to_vec().unwrap()
                )
        });
        proposals_by_proposer_set.insert(&new_proposal.id);
        fpo.proposals_by_proposer.insert(&new_proposal.proposer_id, &proposals_by_proposer_set);

        let unmatched_supply_exists = fpo.acceptable_proposals.len() < fpo.supply_left;
        if unmatched_supply_exists {
            fpo.acceptable_proposals.push(&new_proposal.id);
        } else {
            let outbid_proposal_id = fpo.acceptable_proposals.replace(0, &new_proposal.id);
            let outbid_proposal = &mut fpo.proposals.get(&outbid_proposal_id).expect("Outbid proposal is missing, inconsistent state");
            outbid_proposal.mark_unacceptable_and_refund_deposit();
        }

        fpo.sort_acceptable_proposals();

        // return surplus deposit
        let surplus_deposit = attached_balance_yocto - deposit_yocto;
        if surplus_deposit > 0 {
            Promise::new(env::predecessor_account_id()).transfer(surplus_deposit);
        }
        
        // self.fpos_by_contract_id.insert(&fpo.nft_contract_id, &fpo);
    }    

    #[payable]
    pub fn fpo_modify_proposal(
        &mut self,
        nft_contract_id: AccountId,
        proposal_id: ProposalId,
        price_yocto: u128,
    ) {
        // get FPO
        let fpo = &mut self.fpos_by_contract_id.get(&nft_contract_id).expect("Could not find NFT listing");

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
    
        // check if there is a prior proposal from this signer
        let signer_account_id = env::signer_account_id();
        let signers_proposals = fpo.proposals_by_proposer.get(&signer_account_id).expect("No prior proposal from this account");
        assert!(
            signers_proposals.contains(&proposal_id),
            "Proposal with ID {} from account {} not found",
            proposal_id, signer_account_id
        );

        // check if proposed price is acceptable
        let acceptable_price_yocto = fpo.acceptable_price_yocto();
                assert!(
                    price_yocto >= acceptable_price_yocto,
                    "The minimum acceptable price is {} yoctoNear",
                    acceptable_price_yocto
                );
        
        // ensure the attached balance is sufficient to cover higher required deposit
        let proposal = &mut fpo.proposals.get(&proposal_id).expect("Could not find proposal");
        let deposit_supplement_yocto = (price_yocto - proposal.price_yocto) * PROPOSAL_DEPOSIT_RATE / 100;
        let attached_balance_yocto = env::attached_deposit();
        assert!(
            attached_balance_yocto >= deposit_supplement_yocto, 
            "Attached balance must be sufficient to pay the required deposit supplement of ({:?} yocto Near)", 
            deposit_supplement_yocto
        );
        
        // update price and mark acceptable
        proposal.price_yocto = price_yocto;
        proposal.is_acceptable = true;

        // if the proposal is among the acceptable ones we'll just re-sort
        // otherwise we need to outbid the lowers-priced proposal
        if !fpo.is_proposal_acceptable(proposal_id) {
            // here we assume that it used to be a acceptable one when was first submitted
            // (otherwise it'd have been rejected in the first place) and got outbid at some
            // point - this, in turn, means that the proposal count equals or exceeds the supply
            // so we can just replace the first acceptable proposal (worst price) with this one
            let outbid_proposal_id = fpo.acceptable_proposals.replace(0, &proposal_id);
            let outbid_proposal = &mut fpo.proposals.get(&outbid_proposal_id).expect("Outbid proposal is missing, inconsistent state");
            outbid_proposal.mark_unacceptable_and_refund_deposit();
        }
        
        fpo.sort_acceptable_proposals();

        // return surplus deposit
        let surplus_deposit = attached_balance_yocto - deposit_supplement_yocto;
        if surplus_deposit > 0 {
            Promise::new(env::predecessor_account_id()).transfer(surplus_deposit);
        }
    }
}

