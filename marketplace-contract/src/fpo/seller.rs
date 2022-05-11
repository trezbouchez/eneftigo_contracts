use crate::fpo::config::*;
use crate::FixedPriceOfferingStatus::*;
use crate::internal::*;
use crate::*;

use chrono::DateTime;

use near_sdk::collections::{LookupMap, Vector};
use near_sdk::json_types::{U128};
use near_sdk::AccountId;

#[cfg(test)]
#[path = "seller_tests.rs"]
mod seller_tests;

#[near_bindgen]
impl MarketplaceContract {

    #[payable]
    pub fn fpo_add_buy_now_only(
        &mut self,
        nft_contract_id: AccountId,
        offeror_id: AccountId,
        supply_total: u64,
        buy_now_price_yocto: U128,
        nft_metadata: TokenMetadata,
        start_date: Option<String>, // if missing, it's start accepting bids when this transaction is mined
        end_date: Option<String>,
    ) {
        // ensure max supply does not exceed limit
        assert!(
            supply_total > 0 && supply_total <= TOTAL_SUPPLY_MAX,
            "Max NFT supply must be between 1 and {}.",
            TOTAL_SUPPLY_MAX
        );

        // make sure it's not yet listed
        assert!(
            self.fpos_by_contract_id.get(&nft_contract_id).is_none(),
            "Already listed"
        );

        // price must be at least MIN_PRICE_YOCTO
        assert!(
            buy_now_price_yocto.0 >= MIN_BUY_NOW_PRICE_YOCTO,
            "Price cannot be lower than {} yoctoNear",
            MIN_BUY_NOW_PRICE_YOCTO
        );

        // price must be multiple of PRICE_STEP_YOCTO
        assert!(
            buy_now_price_yocto.0 % PRICE_STEP_YOCTO == 0,
            "Price must be integer multiple of {} yoctoNear",
            PRICE_STEP_YOCTO
        );

        // get initial storage
        let initial_storage_usage = env::storage_usage();

        // start timestamp
        let start_timestamp: Option<i64> = if let Some(start_date_str) = start_date {
            let start_datetime = DateTime::parse_from_rfc3339(&start_date_str).expect(
                "Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)",
            );
            let start_timestamp = start_datetime.timestamp_nanos();
            let current_block_timestamp = env::block_timestamp() as i64;
            assert!(
                start_timestamp >= current_block_timestamp,
                "Start date is into the past"
            );
            Some(start_timestamp)
        } else {
            None
        };

        // end timestamp
        let end_timestamp: Option<i64> = if let Some(end_date_str) = end_date {
            let end_datetime = DateTime::parse_from_rfc3339(&end_date_str).expect(
                "Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)",
            );
            let end_timestamp = end_datetime.timestamp_nanos();
            let current_block_timestamp = env::block_timestamp() as i64;
            assert!(
                end_timestamp >= current_block_timestamp,
                "End date is into the past"
            );
            Some(end_timestamp)
            // let end_datetime_str = (Utc.ymd(1970, 1, 1).and_hms(0, 0, 0) + Duration::nanoseconds(end_timestamp_nanos)).to_rfc3339();
            // env::log_str(&end_datetime_str);
        } else {
            None
        };

        if let Some(start_timestamp) = start_timestamp {
            if let Some(end_timestamp) = end_timestamp {
                let duration = end_timestamp - start_timestamp;
                assert!(
                    duration >= MIN_DURATION_NANO,
                    "Offering duration too short"
                );
            }
        }

        let nft_contract_id_hash = hash_account_id(&nft_contract_id);
        let fpo = FixedPriceOffering {
            nft_contract_id,
            offeror_id,
            supply_total,
            buy_now_price_yocto: buy_now_price_yocto.0,
            min_proposal_price_yocto: None,
            nft_metadata,
            start_timestamp,
            end_timestamp,
            status: Unstarted,
            supply_left: supply_total,
            proposals: LookupMap::new(
                FixedPriceOfferingStorageKey::Proposals {
                    nft_contract_id_hash: nft_contract_id_hash,
                }
                .try_to_vec()
                .unwrap(),
            ),
            proposals_by_proposer: LookupMap::new(
                FixedPriceOfferingStorageKey::ProposalsByProposer {
                    nft_contract_id_hash: nft_contract_id_hash,
                }
                .try_to_vec()
                .unwrap(),
            ),
            acceptable_proposals: Vector::new(
                FixedPriceOfferingStorageKey::AcceptableProposals {
                    nft_contract_id_hash: nft_contract_id_hash,
                }
                .try_to_vec()
                .unwrap(),
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

    #[payable]
    pub fn fpo_add_accepting_proposals(
        &mut self,
        nft_contract_id: AccountId,
        offeror_id: AccountId,
        supply_total: u64,
        buy_now_price_yocto: U128,
        min_proposal_price_yocto: U128,
        nft_metadata: TokenMetadata,
        start_date: Option<String>, // if None, will start when block is mined
        end_date: String,
    ) {
        // ensure max supply does not exceed limit
        assert!(
            supply_total > 0 && supply_total <= TOTAL_SUPPLY_MAX,
            "Max NFT supply must be between 1 and {}.",
            TOTAL_SUPPLY_MAX
        );

        // make sure it's not yet listed
        assert!(
            self.fpos_by_contract_id.get(&nft_contract_id).is_none(),
            "Already listed"
        );

        // price must be at least MIN_PRICE_YOCTO
        assert!(
            buy_now_price_yocto.0 >= MIN_BUY_NOW_PRICE_YOCTO,
            "Price cannot be lower than {} yoctoNear",
            MIN_BUY_NOW_PRICE_YOCTO
        );

        // prices must be multiple of PRICE_STEP_YOCTO
        assert!(
            buy_now_price_yocto.0 % PRICE_STEP_YOCTO == 0
                && min_proposal_price_yocto.0 % PRICE_STEP_YOCTO == 0,
            "Prices must be integer multiple of {} yoctoNear",
            PRICE_STEP_YOCTO
        );

        // buy_now_price_yocto must be greater than min_proposal_price_yocto
        assert!(
            buy_now_price_yocto.0 > min_proposal_price_yocto.0,
            "Min proposal price must be lower than buy now price"
        );

        // get initial storage
        let initial_storage_usage = env::storage_usage();

        // start timestamp
        let start_timestamp: Option<i64> = if let Some(start_date_str) = start_date {
            let start_datetime = DateTime::parse_from_rfc3339(&start_date_str).expect(
                "Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)",
            );
            let start_timestamp = start_datetime.timestamp_nanos();
            let current_block_timestamp = env::block_timestamp() as i64;
            assert!(
                start_timestamp >= current_block_timestamp,
                "Start date is into the past"
            );
            Some(start_timestamp)
        } else {
            None
        };

        // end timestamp
        let end_datetime = DateTime::parse_from_rfc3339(&end_date)
            .expect("Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)");
        let end_timestamp = end_datetime.timestamp_nanos();

        if let Some(start_timestamp) = start_timestamp {
            let duration = end_timestamp - start_timestamp;
            assert!(
                duration >= MIN_DURATION_NANO,
                "Offering duration too short"
            );
            assert!(
                duration <= MAX_DURATION_NANO,
                "Offering duration too long"
            );
        } else {
            let current_block_timestamp = env::block_timestamp() as i64;
            assert!(
                end_timestamp >= current_block_timestamp,
                "End date is into the past"
            );
        }

        let nft_contract_id_hash = hash_account_id(&nft_contract_id);
        let fpo = FixedPriceOffering {
            nft_contract_id,
            offeror_id,
            supply_total: supply_total,
            buy_now_price_yocto: buy_now_price_yocto.0,
            min_proposal_price_yocto: Some(min_proposal_price_yocto.0),
            nft_metadata,
            start_timestamp,
            end_timestamp: Some(end_timestamp),
            status: Unstarted,
            supply_left: supply_total,
            proposals: LookupMap::new(
                FixedPriceOfferingStorageKey::Proposals {
                    nft_contract_id_hash: nft_contract_id_hash,
                }
                .try_to_vec()
                .unwrap(),
            ),
            proposals_by_proposer: LookupMap::new(
                FixedPriceOfferingStorageKey::ProposalsByProposer {
                    nft_contract_id_hash: nft_contract_id_hash,
                }
                .try_to_vec()
                .unwrap(),
            ),
            acceptable_proposals: Vector::new(
                FixedPriceOfferingStorageKey::AcceptableProposals {
                    nft_contract_id_hash: nft_contract_id_hash,
                }
                .try_to_vec()
                .unwrap(),
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

    pub fn fpo_accept_proposals(
        &mut self,
        nft_contract_id: AccountId,
        accepted_proposals_count: u64,
        ) {
        // get the FPO
        let mut fpo = self.fpos_by_contract_id.get(&nft_contract_id).expect("Could not find NFT listing");

        // make sure it's the offeror who's calling this
        assert!(
            env::predecessor_account_id() == fpo.offeror_id,
            "Only the offeror can accept proposals"
        );

        // make sure there's enough proposals
        let num_acceptable_proposals = fpo.acceptable_proposals.len();
        assert!(
            num_acceptable_proposals >= accepted_proposals_count,
            "There's not enough proposals ({})",
            num_acceptable_proposals
        );

        // accept best proposals
        let mut acceptable_proposals_vec = fpo.acceptable_proposals.to_vec();
        let first_accepted_proposal_index = (num_acceptable_proposals - accepted_proposals_count) as usize;

        let best_proposals_iter = acceptable_proposals_vec.drain(first_accepted_proposal_index..(num_acceptable_proposals as usize));
        let mut minted_nft_id = fpo.supply_total - fpo.supply_left;
        
        for proposal_being_accepted_id in best_proposals_iter {
            let proposal_being_accepted = fpo.proposals
                .get(&proposal_being_accepted_id)
                .expect("Proposal being accepted is missing, inconsistent state");
            let proposer_id = proposal_being_accepted.proposer_id;
            self.fpo_process_purchase(
                nft_contract_id.clone(),
                minted_nft_id.to_string(),
                proposer_id.clone(),
                proposal_being_accepted.price_yocto.clone()
            );
            minted_nft_id += 1;

            // TODO: move these to fpo_process_purchase resolve
            let _removed_proposal = fpo.proposals.remove(&proposal_being_accepted_id).expect("Could not find proposal");

            let mut proposals_by_this_proposer = fpo.proposals_by_proposer.get(&proposer_id)
            .expect("Could not get proposals for proposer whose proposal is being accepted");
            let removed = proposals_by_this_proposer.remove(&proposal_being_accepted_id);
            assert!(removed, "Could not find id for proposer's proposals");
            if proposals_by_this_proposer.is_empty() {
                fpo.proposals_by_proposer.remove(&proposer_id).expect("Could not remove empty array for proposer whose proposals have all been accepted");
            } else {
                fpo.proposals_by_proposer.insert(&proposer_id, &proposals_by_this_proposer);
            }
        }

        fpo.acceptable_proposals.clear();
        fpo.acceptable_proposals.extend(acceptable_proposals_vec);

        fpo.supply_left -= accepted_proposals_count;        // TODO: move to resolve, one by one
        
        self.fpos_by_contract_id.insert(&nft_contract_id, &fpo);
    }

    // here the caller will need to cover the refund transfers gas if there's supply left
    // this is because there may be multiple acceptable proposals pending which have active deposits
    // they need to be returned
    // must be called by the offeror!
    pub(crate) fn fpo_conclude(&mut self, nft_contract_id: AccountId) {
        // get the FPO
        let fpo = &mut self.fpos_by_contract_id.get(&nft_contract_id).expect("Could not find NFT listing");

        // make sure it's the offeror who's calling this
        assert!(
            env::predecessor_account_id() == fpo.offeror_id,
            "Only the offeror can accept proposals"
        );

        // remove FPO
        let removed_fpo = self
            .fpos_by_contract_id
            .remove(&nft_contract_id)
            .expect("Could not find this NFT listing");

        // refund all acceptable but not accepted proposals
        for unaccepted_proposal in removed_fpo.acceptable_proposals.iter()
        .map(|proposal_id| removed_fpo.proposals.get(&proposal_id).expect("Could not find proposal")) {
            unaccepted_proposal.refund_deposit();
        }

        let offeror_id = removed_fpo.offeror_id;

        // check if the caller can end the listing
        // 1. the marketplace account can end at any time
        // 2. the offeror can end if either of these is met:
        // - the end time has passed or is not set
        // - the offering is already marked closed
        let predecessor_account_id = env::predecessor_account_id();
        let contract_account_id = env::current_account_id();

        if predecessor_account_id != contract_account_id {
            assert!(
                predecessor_account_id == offeror_id,
                "Only offeror can conclude the offering."
            );
            if let Some(end_timestamp) = removed_fpo.end_timestamp {
                let current_block_timestamp_nanos = env::block_timestamp() as i64;
                assert!(
                    current_block_timestamp_nanos >= end_timestamp,
                    "Can only conclude after end date has passed"
                );
            }
        }

        let fpos_by_this_offeror = &mut self
            .fpos_by_offeror_id
            .get(&offeror_id)
            .expect("Could not find offers for this offeror");
        let did_remove = fpos_by_this_offeror.remove(&nft_contract_id);
        assert!(did_remove, "Offer not on offeror's list");
        if fpos_by_this_offeror.is_empty() {
            self.fpos_by_offeror_id
                .remove(&offeror_id)
                .expect("Could not remove the now-empty offer list");
        }
    }
}
