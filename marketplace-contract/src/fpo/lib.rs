use crate::*;
use crate::internal::*;

use std::cmp::Ordering;
use chrono::{DateTime, Duration, Utc, TimeZone};

use near_sdk::collections::Vector;
use near_sdk::json_types::{U128};

// use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
// use near_sdk::serde::{Deserialize, Serialize};

// TODO: tweak these if needed
//const STORAGE_PER_FIXED_PRICE_OFFERING: u128 = 1000 * STORAGE_PRICE_PER_BYTE;   // TODO: measure and tweak
const NFT_TOTAL_SUPPLY_MAX: u64 = 100;
const FPO_MIN_PRICE_YOCTO: u128 = 1000;
const FPO_PRICE_STEP_YOCTO: u128 = 10;

// TODO: it is important to set the minimum effective yocto deposit
// (determined by FPO_MIN_PRICE_YOCTO and FPO_PROPOSAL_DEPOSIT_RATE)
// to be greater than or equal the storage cost per proposal; this
// prevents the mallicious attempt to drain the contract out of its
// Near balance known as "million cheap data additions" attack
// https://docs.near.org/docs/concepts/storage-staking
const FPO_PROPOSAL_DEPOSIT_RATE: u128 = 10;     // percent

pub type ProposalId = u128;

// contains fixed-price offering parameters

#[derive(BorshStorageKey, BorshSerialize)]
pub enum FixedPriceOfferingStorageKey {
    Proposals { nft_contract_id_hash: CryptoHash },
    ProposalsByProposer { nft_contract_id_hash: CryptoHash },
    ProposalsByProposerInner { nft_contract_id_hash: CryptoHash, proposer_id_hash: CryptoHash },
    WinningProposals { nft_contract_id_hash: CryptoHash },
}

#[derive(BorshDeserialize, BorshSerialize/*, Serialize, Deserialize*/)]
#[derive(Eq)]
// #[serde(crate = "near_sdk::serde")]
pub struct FixedPriceOfferingProposal {
    pub id: ProposalId,
    pub proposer_id: AccountId,
    pub price_yocto: u128,
    pub is_winning: bool,
}

impl Ord for FixedPriceOfferingProposal {       // lower looses, greater wins
    fn cmp(&self, other: &Self) -> Ordering {
        if self.price_yocto < other.price_yocto {     // lower proposed price
            Ordering::Less
        } else if self.price_yocto == other.price_yocto {
            other.id.cmp(&self.id)
        } else {
            Ordering::Greater
        }
    }
}

impl PartialOrd for FixedPriceOfferingProposal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for FixedPriceOfferingProposal {
    fn eq(&self, other: &Self) -> bool {
        self.price_yocto == other.price_yocto &&
        self.id == other.id
    }
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FixedPriceOffering {
    pub nft_contract_id: AccountId,
    pub offeror_id: AccountId,
    pub supply_total: u64,
    pub buy_now_price_yocto: u128,             
    pub min_proposal_price_yocto: u128,     
    pub end_timestamp: Option<i64>,         // nanoseconds since 1970-01-01
    pub nft_metadata: TokenMetadata,
    pub supply_left: u64,    
    pub proposals: LookupMap<ProposalId, FixedPriceOfferingProposal>,              
    pub proposals_by_proposer: LookupMap<AccountId, UnorderedSet<ProposalId>>,
    pub winning_proposals: Vector<FixedPriceOfferingProposal>,      // ordered by ascending price
    pub next_proposal_id: u128,
}

#[ext_contract(ext_self)]
trait FixedPriceOfferingResolver {
    fn fpo_resolve_purchase(
        &mut self,
        nft_contract_id: AccountId,
        buyer_id: AccountId,
    ) -> bool;
}

trait FixedPriceOfferingResolver {
    fn fpo_resolve_purchase(
        &mut self,
        nft_contract_id: AccountId,
        buyer_id: AccountId,
    ) -> bool;
}

#[near_bindgen]
impl MarketplaceContract {

    pub fn fpo_total_supply(
        &self,
    ) -> U128 {
        U128(self.fpos_by_contract_id.len() as u128)
    }

    #[payable]
    pub fn fpo_list(
        &mut self,
        nft_contract_id: AccountId,
        offeror_id: AccountId,
        supply_total: u64,
        buy_now_price_yocto: U128,
        min_proposal_price_yocto: U128,
        nft_metadata: TokenMetadata,
        duration_days: Option<u64>,         // if duration is set, end_date must be None 
        end_date: Option<String>,
    ) {
        // make sure it's called by marketplace 
        assert_eq!(
            &env::predecessor_account_id(),
            &self.owner_id,
            "Only Eneftigo Marketplace owner can add listing."
        );

        // ensure max supply does not exceed limit
        assert!(
            supply_total > 0 && supply_total <= NFT_TOTAL_SUPPLY_MAX,
            "Max NFT supply must be between 1 and {}.", 
            NFT_TOTAL_SUPPLY_MAX
        );

        // make sure it's not yet listed
        assert!(
            self.fpos_by_contract_id.get(&nft_contract_id).is_none(),
            "Already listed"
        );

        // price must be at least FPO_MIN_PRICE_YOCTO
        assert!(
            buy_now_price_yocto.0 >= FPO_MIN_PRICE_YOCTO,
            "Price cannot be lower than {} yocto Near", FPO_MIN_PRICE_YOCTO
        );
        
        // prices must be multiple of FPO_PRICE_STEP_YOCTO
        assert!(
            buy_now_price_yocto.0 % FPO_PRICE_STEP_YOCTO == 0 || min_proposal_price_yocto.0 % FPO_PRICE_STEP_YOCTO == 0,
            "Prices must be integer multiple of {} yocto Near",
            FPO_PRICE_STEP_YOCTO
        );

        // get initial storage
        let initial_storage_usage = env::storage_usage();

        // end timestamp
        let end_timestamp: Option<i64>;
        if let Some(duration_days) = duration_days {
            assert!(end_date.is_none(), "Either duration or end date can be provided, not both.");
            let current_block_timestamp_nanos = env::block_timestamp() as i64;
            let current_block_datetime = Utc.ymd(1970, 1, 1).and_hms(0, 0, 0) + Duration::nanoseconds(current_block_timestamp_nanos);
            let end_datetime = current_block_datetime + Duration::days(duration_days as i64);
            let end_timestamp_nanos = end_datetime.timestamp_nanos();
            end_timestamp = Some(end_timestamp_nanos);
            let end_datetime_str = (Utc.ymd(1970, 1, 1).and_hms(0, 0, 0) + Duration::nanoseconds(end_timestamp_nanos)).to_rfc3339();
            env::log_str(&end_datetime_str);
        } else if let Some(end_date_str) = end_date {
            assert!(duration_days.is_none(), "Either duration or end date can be provided, not both.");
            let end_datetime = DateTime::parse_from_rfc3339(&end_date_str).expect("Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)");
            let end_timestamp_nanos = end_datetime.timestamp_nanos();
            let current_block_timestamp_nanos = env::block_timestamp() as i64;
            assert!(end_timestamp_nanos > current_block_timestamp_nanos, "End date is into the past");
            end_timestamp = Some(end_timestamp_nanos);
            // let end_datetime_str = (Utc.ymd(1970, 1, 1).and_hms(0, 0, 0) + Duration::nanoseconds(end_timestamp_nanos)).to_rfc3339();
            // env::log_str(&end_datetime_str);
        } else {
            end_timestamp = None;
        }

        let nft_contract_id_hash = hash_account_id(&nft_contract_id);
        let fpo = FixedPriceOffering {
            nft_contract_id,
            offeror_id,
            supply_total: supply_total,
            buy_now_price_yocto: buy_now_price_yocto.0,
            min_proposal_price_yocto: min_proposal_price_yocto.0,
            nft_metadata,
            end_timestamp,
            supply_left: supply_total,
            proposals: LookupMap::new(
                FixedPriceOfferingStorageKey::Proposals {
                    nft_contract_id_hash: nft_contract_id_hash,
                }
                .try_to_vec()
                .unwrap()
            ),
            proposals_by_proposer: LookupMap::new(
                FixedPriceOfferingStorageKey::ProposalsByProposer {
                    nft_contract_id_hash: nft_contract_id_hash,
                }
                .try_to_vec()
                .unwrap()
            ),
            winning_proposals: Vector::new(
                FixedPriceOfferingStorageKey::WinningProposals {
                    nft_contract_id_hash: nft_contract_id_hash,
                }
                .try_to_vec()
                .unwrap()
            ),
            next_proposal_id: 0,
        };

        self.fpos_by_contract_id.insert(&fpo.nft_contract_id, &fpo);

        self.internal_add_fpo_to_offeror(&fpo.offeror_id, &fpo.nft_contract_id);

        // calculate the extra storage used by FPO entries
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        // refund any excess storage if the user attached too much. Panic if they didn't attach enough to cover what's required.
        refund_deposit(required_storage_in_bytes);
    }

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
        let mut fpo = self.fpos_by_contract_id.get(&nft_contract_id).expect("Could not find NFT listing");

        // ensure there's supply left
        assert!(
            fpo.supply_left > 0,
            "You are late. All NFTs have been sold."
        );

        // ensure the attached balance is sufficient
        let attached_balance = env::attached_deposit();
        assert!(
            attached_balance >= fpo.buy_now_price_yocto, 
            "Attached Near must be sufficient to pay the price of {:?} yocto Near", 
            fpo.buy_now_price_yocto
        );

        // process the purchase
        let buyer_id = env::predecessor_account_id();

        fpo.process_purchase(buyer_id);

        //get the refund amount from the attached deposit - required cost
        // let refund = attached_deposit - required_cost;
    
        //if the refund is greater than 1 yocto NEAR, we refund the predecessor that amount
        // if refund > 1 {
        //     Promise::new(env::predecessor_account_id()).transfer(refund);
        // }
    }

    // place price proposal
    #[payable]
    pub fn fpo_place_proposal(
        &mut self,
        nft_contract_id: AccountId,
        price_yocto: u128,
    ) {
        // get FPO
        let mut fpo = self.fpos_by_contract_id.get(&nft_contract_id).expect("Could not find NFT listing");

        // ensure there's supply left
        assert!(
            fpo.supply_left > 0,
            "You are late. All NFTs have been sold."
        );

        // price must be multiple of FPO_PRICE_STEP_YOCTO
        assert!(
            price_yocto % FPO_PRICE_STEP_YOCTO == 0,
            "Price must be an integer multple of {} yocto Near", 
            FPO_PRICE_STEP_YOCTO
        );
        
        // ensure the attached balance is sufficient to pay deposit
        // TODO: should we adopt approvals instead?
        let attached_balance_yocto = env::attached_deposit();
        let deposit_yocto = price_yocto * FPO_PROPOSAL_DEPOSIT_RATE / 100;
        assert!(
            attached_balance_yocto >= deposit_yocto, 
            "Attached balance must be sufficient to pay the required {:?}% deposit ({:?} yocto Near)", 
            FPO_PROPOSAL_DEPOSIT_RATE, 
            deposit_yocto 
        );

        // get proposals vector (was sorted on write) and check if proposed price is acceptable
        let winning_proposals = &mut fpo.winning_proposals;
        let unmatched_supply_exists = winning_proposals.len() < fpo.supply_left;
        let min_accepted_price_yocto = if unmatched_supply_exists {
            fpo.min_proposal_price_yocto
        } else {
            winning_proposals.get(0).unwrap().price_yocto + FPO_PRICE_STEP_YOCTO
        };
        assert!(
            price_yocto >= min_accepted_price_yocto,
            "Proposed price is too low. The lowest acceptable price is {:?}",
            min_accepted_price_yocto
        );

        // register proposal
        let proposer_id = env::predecessor_account_id();
        let new_proposal = FixedPriceOfferingProposal {
            id: fpo.next_proposal_id,
            proposer_id: proposer_id,
            price_yocto: price_yocto,
            is_winning: true,
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

        if unmatched_supply_exists {
            winning_proposals.push(&new_proposal);
        } else {
            let outbid_proposal_id = winning_proposals.replace(0, &new_proposal).id;
            let outbid_proposal = &mut fpo.proposals.get(&outbid_proposal_id).expect("Outbid proposal is missing, inconsistent state");
            outbid_proposal.is_winning = false;
        }

        fpo.sort_winning_proposals();

        // let mut winning_proposals_vec_sorted = winning_proposals.to_vec();
        // winning_proposals_vec_sorted.sort();
        // let nft_contract_id_hash = hash_account_id(&nft_contract_id);
        // let mut winning_proposals_sorted = Vector::new(FixedPriceOfferingStorageKey::Proposals {
        //     nft_contract_id_hash: nft_contract_id_hash,
        // });
        // for winning_proposal in &winning_proposals_vec_sorted {
        //     winning_proposals_sorted.push(winning_proposal);
        // }
        // fpo.winning_proposals = winning_proposals_sorted;

        self.fpos_by_contract_id.insert(&fpo.nft_contract_id, &fpo);
    }
}

#[near_bindgen]
impl FixedPriceOfferingResolver for FixedPriceOffering {

    fn fpo_resolve_purchase(
        &mut self,
        nft_contract_id: AccountId,
        buyer_id: AccountId,
    ) -> bool {
        return false;
        // TODO: update proposals by popping least attractive ones until their number matches the supply
    }
}

// place a fixed-price offering. The sale will go through as long as your deposit is greater than or equal to the list price
//#[payable]
//pub fn offer(&mut self, nft_contract_id: AccountId, token_id: String) {
/*    //get the attached deposit and make sure it's greater than 0
    let deposit = env::attached_deposit();
    assert!(deposit > 0, "Attached deposit must be greater than 0");

    //convert the nft_contract_id from a AccountId to an AccountId
    let contract_id: AccountId = nft_contract_id.into();
    //get the unique sale ID (contract + DELIMITER + token ID)
    let contract_and_token_id = format!("{}{}{}", contract_id, DELIMETER, token_id);
    
    //get the sale object from the unique sale ID. If the sale doesn't exist, panic.
    let sale = self.sales.get(&contract_and_token_id).expect("No sale");
    
    //get the buyer ID which is the person who called the function and make sure they're not the owner of the sale
    let buyer_id = env::predecessor_account_id();
    assert_ne!(sale.owner_id, buyer_id, "Cannot bid on your own sale.");
    
    //get the u128 price of the token (dot 0 converts from U128 to u128)
    let price = sale.sale_conditions.0;

    //make sure the deposit is greater than the price
    assert!(deposit >= price, "Attached deposit must be greater than or equal to the current price: {:?}", price);

    //process the purchase (which will remove the sale, transfer and get the payout from the nft contract, and then distribute royalties) 
    self.process_purchase(
        contract_id,
        token_id,
        U128(deposit),
        buyer_id,
    );*/
// }