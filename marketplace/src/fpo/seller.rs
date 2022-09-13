use crate::config::*;
use crate::fpo::config::*;
use crate::internal::*;
use crate::FixedPriceOfferingStatus::*;
use crate::*;

use chrono::DateTime;
use url::Url;

use near_sdk::{
    collections::{LookupMap, TreeMap, Vector},
    json_types::U128,
    AccountId, PromiseResult,
};

const NFT_MAKE_COLLECTION_GAS: Gas = Gas(5_000_000_000_000); // highest measured 3_920_035_683_889
const NFT_MAKE_COLLECTION_COMPLETION_GAS: Gas = Gas(6_000_000_000_000); // highest measured 5_089_357_803_858

pub const MAX_TITLE_LEN: usize = 128;

#[cfg(test)]
#[path = "seller_tests.rs"]
mod seller_tests;

#[near_bindgen]
impl MarketplaceContract {
    #[payable]
    pub fn fpo_add_buy_now_only(
        &mut self,
        title: String,
        media_url: String,
        supply_total: u64,
        buy_now_price_yocto: U128,
        start_date: Option<String>, // if missing, it's start accepting bids when this transaction is mined
        end_date: Option<String>,
    ) -> Promise {
        let attached_deposit = env::attached_deposit();
        let anticipated_marketplace_storage =
            fpo_add_buy_now_only_storage(&title, &media_url, &start_date, &end_date);
        let anticipated_nft_storage = nft_make_collection_storage(&title, &media_url);
        let anticipated_storage = anticipated_marketplace_storage + anticipated_nft_storage;
        let anticipated_storage_cost = anticipated_storage as Balance * env::storage_byte_cost();
        assert!(
            attached_deposit >= anticipated_storage_cost,
            "Attach at least {} yN for storage (marketplace storage of {} and NFT collection storage of {})",
            anticipated_storage_cost,
            anticipated_marketplace_storage,
            anticipated_nft_storage
        );
        assert!(
            title.len() <= MAX_TITLE_LEN,
            "Title length cannot exceed {} characters",
            MAX_TITLE_LEN
        );
        assert!(Url::parse(&media_url).is_ok(), "NFT media URL is invalid");

        // ensure max supply does not exceed limit
        assert!(
            supply_total > 0 && supply_total <= TOTAL_SUPPLY_MAX,
            "Max NFT supply must be between 1 and {}.",
            TOTAL_SUPPLY_MAX
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

        if let Some(end_timestamp) = end_timestamp {
            if let Some(start_timestamp) = start_timestamp {
                let duration = end_timestamp - start_timestamp;
                assert!(duration >= MIN_DURATION_NANO, "Offering duration too short");
            } else {
                let current_block_timestamp = env::block_timestamp() as i64;
                let duration = end_timestamp - current_block_timestamp;
                assert!(duration >= MIN_DURATION_NANO, "Offering duration too short");
            }
        }

        let attached_deposit = env::attached_deposit();
        let nft_contract_id = self.internal_nft_shared_contract_id();
        let nft_metadata = NftMetadata::new(&title, &media_url);

        nft_contract::make_collection(
            nft_metadata.clone(),
            supply_total,
            nft_contract_id.clone(),
            attached_deposit,
            NFT_MAKE_COLLECTION_GAS,
        )
        .then(ext_self_nft::fpo_add_make_collection_completion(
            nft_contract_id,
            U128(attached_deposit),
            nft_metadata,
            supply_total,
            buy_now_price_yocto,
            None, // min_proposal_price_yocto
            start_timestamp,
            end_timestamp,
            env::current_account_id(),
            NO_DEPOSIT,
            NFT_MAKE_COLLECTION_COMPLETION_GAS,
        ))
    }

    #[payable]
    pub fn fpo_add_accepting_proposals(
        &mut self,
        title: String,
        media_url: String,
        supply_total: u64,
        buy_now_price_yocto: U128,
        min_proposal_price_yocto: U128,
        start_date: Option<String>, // if None, will start when block is mined
        end_date: String,
    ) -> Promise {
        let attached_deposit = env::attached_deposit();
        let storage_byte_cost = env::storage_byte_cost();

        let anticipated_marketplace_storage =
            fpo_add_accepting_proposals_storage(&title, &media_url, &start_date);
        let anticipated_marketplace_storage_cost =
            anticipated_marketplace_storage as Balance * storage_byte_cost;
        let anticipated_nft_storage = nft_make_collection_storage(&title, &media_url);
        let anticipated_nft_storage_cost = anticipated_nft_storage as Balance * storage_byte_cost;
        let anticipated_storage_cost = anticipated_marketplace_storage_cost + anticipated_nft_storage_cost;
        assert!(
            attached_deposit >= anticipated_storage_cost,
            "Attach at least {} yN",
            anticipated_storage_cost
        );

        assert!(
            title.len() <= MAX_TITLE_LEN,
            "Title length cannot exceed {} characters",
            MAX_TITLE_LEN
        );
        assert!(Url::parse(&media_url).is_ok(), "NFT media URL is invalid");

        // ensure max supply does not exceed limit
        assert!(
            supply_total > 0 && supply_total <= TOTAL_SUPPLY_MAX,
            "Max NFT supply must be between 1 and {}.",
            TOTAL_SUPPLY_MAX
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

        // start timestamp
        let current_block_timestamp = env::block_timestamp() as i64;
        let start_timestamp: Option<i64> = if let Some(start_date_str) = start_date {
            let start_datetime = DateTime::parse_from_rfc3339(&start_date_str).expect(
                "Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)",
            );
            let start_timestamp = start_datetime.timestamp_nanos();
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
            assert!(duration >= MIN_DURATION_NANO, "Offering duration too short");
            assert!(duration <= MAX_DURATION_NANO, "Offering duration too long");
        } else {
            let current_block_timestamp = env::block_timestamp() as i64;
            let duration = end_timestamp - current_block_timestamp;
            assert!(duration >= MIN_DURATION_NANO, "Offering duration too short");
            assert!(duration <= MAX_DURATION_NANO, "Offering duration too long");
        }

        let attached_deposit = env::attached_deposit();
        let nft_contract_id = self.internal_nft_shared_contract_id();
        let nft_metadata = NftMetadata::new(&title, &media_url);

        nft_contract::make_collection(
            nft_metadata.clone(),
            supply_total,
            nft_contract_id.clone(),
            anticipated_nft_storage_cost,
            NFT_MAKE_COLLECTION_GAS,
        )
        .then(ext_self_nft::fpo_add_make_collection_completion(
            nft_contract_id,
            U128(attached_deposit),
            nft_metadata,
            supply_total,
            buy_now_price_yocto,
            Some(min_proposal_price_yocto),
            start_timestamp,
            Some(end_timestamp),
            env::current_account_id(), // we are invoking this function on the current contract
            NO_DEPOSIT,                // don't attach any deposit
            NFT_MAKE_COLLECTION_COMPLETION_GAS, // GAS attached to the completion call
        ))
    }

    pub fn fpo_accept_proposals(
        &mut self,
        nft_contract_id: AccountId,
        collection_id: NftCollectionId,
        accepted_proposals_count: u64,
    ) {
        let offering_id = OfferingId {
            nft_contract_id,
            collection_id,
        };

        // get the FPO
        let mut fpo = self
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not find NFT listing");

        // make sure it's the offeror who's calling this
        assert!(
            env::predecessor_account_id() == fpo.offeror_id,
            "Only the offeror can accept proposals"
        );

        // make sure there's enough proposals
        let num_proposals = fpo.proposals.len();
        assert!(
            num_proposals >= accepted_proposals_count,
            "There's not enough proposals ({})",
            num_proposals
        );

        // accept best proposals
        let mut proposals_vec = fpo.proposals.to_vec();
        let first_accepted_proposal_index = (num_proposals - accepted_proposals_count) as usize;

        let best_proposals_iter =
            proposals_vec.drain(first_accepted_proposal_index..(num_proposals as usize));
        for proposal_being_accepted_id in best_proposals_iter {
            // TODO:
            // let proposal_being_accepted = fpo
            //     .proposals
            //     .get(&proposal_being_accepted_id)
            //     .expect("Proposal being accepted is missing, inconsistent state");
            // let proposer_id = proposal_being_accepted.proposer_id;

            // TODO:

            // TODO: make more specific callback function to rollback
            // self.fpo_process_purchase(
            //     offering_id.clone(),
            //     proposer_id.clone(),
            //     proposal_being_accepted.price_yocto.clone(),
            // );

            // TODO: move these to fpo_process_purchase resolve
            // let _removed_proposal = fpo
            //     .proposals
            //     .remove(&proposal_being_accepted_id)
            //     .expect("Could not find proposal");

            // let mut proposals_by_this_proposer = fpo
            //     .proposals_by_proposer
            //     .get(&proposer_id)
            //     .expect("Could not get proposals for proposer whose proposal is being accepted");
            // let removed = proposals_by_this_proposer.remove(&proposal_being_accepted_id);
            // assert!(removed, "Could not find id for proposer's proposals");
            // if proposals_by_this_proposer.is_empty() {
            //     fpo.proposals_by_proposer.remove(&proposer_id).expect("Could not remove empty array for proposer whose proposals have all been accepted");
            // } else {
            //     fpo.proposals_by_proposer
            //         .insert(&proposer_id, &proposals_by_this_proposer);
            // }
        }

        fpo.proposals.clear();
        fpo.proposals.extend(proposals_vec);

        fpo.supply_left -= accepted_proposals_count; // TODO: move to resolve, one by one
        self.fpos_by_id.insert(&offering_id, &fpo);
    }

    // here the caller will need to cover the refund transfers gas if there's supply left
    // this is because there may be multiple acceptable proposals pending which have active deposits
    // they need to be returned
    // must be called by the offeror!
    pub fn fpo_conclude(&mut self, nft_contract_id: AccountId, collection_id: NftCollectionId) {
        let offering_id = OfferingId {
            nft_contract_id,
            collection_id,
        };

        // get the FPO
        let mut fpo = self
            .fpos_by_id
            .get(&offering_id)
            .expect("Could not find NFT listing");
        fpo.update_status();

        // if there's an end date set, make sure the offering is not running
        // assert!(
        //     fpo.end_timestamp.is_none() || fpo.status == Unstarted || fpo.status == Ended,
        //     "Cannot conclude a time-limited offering while it's running"
        // );

        // make sure it's the offeror who's calling this
        assert!(
            env::predecessor_account_id() == fpo.offeror_id,
            "Only the offeror can conclude"
        );

        // reset supply and refund proposers
        fpo.supply_left = 0;
        fpo.remove_supply_exceeding_proposals_and_refund_proposers();

        // remove FPO and refund the seller
        let storage_before = env::storage_usage();

        let removed_fpo = self.internal_remove_fpo(&offering_id);

        let storage_after = env::storage_usage();
        let storage_freed = storage_before - storage_after;
        let refund = storage_freed as Balance * env::storage_byte_cost();
        if refund > 0 {
            Promise::new(removed_fpo.offeror_id).transfer(refund);
        }
    }
}

#[ext_contract(ext_self_nft)]
trait FPOSellerCallback {
    fn fpo_add_make_collection_completion(
        &mut self,
        nft_account_id: AccountId,
        attached_deposit: U128,
        nft_metadata: NftMetadata,
        supply_total: u64,
        buy_now_price_yocto: U128,
        min_proposal_price_yocto: Option<U128>,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>,
    ) -> NftCollectionId;
}

trait FPOSellerCallback {
    fn fpo_add_make_collection_completion(
        &mut self,
        nft_account_id: AccountId,
        attached_deposit: U128,
        nft_metadata: NftMetadata,
        supply_total: u64,
        buy_now_price_yocto: U128,
        min_proposal_price_yocto: Option<U128>,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>,
    ) -> NftCollectionId;
}

#[near_bindgen]
impl FPOSellerCallback for MarketplaceContract {
    #[private]
    fn fpo_add_make_collection_completion(
        &mut self,
        nft_account_id: AccountId,
        attached_deposit: U128,
        nft_metadata: NftMetadata,
        supply_total: u64,
        buy_now_price_yocto: U128,
        min_proposal_price_yocto: Option<U128>,
        start_timestamp: Option<i64>,
        end_timestamp: Option<i64>,
    ) -> NftCollectionId {
        assert_eq!(env::promise_results_count(), 1, "Too many data receipts");
        match env::promise_result(0) {
            PromiseResult::NotReady | PromiseResult::Failed => {
                let refund = attached_deposit.0;
                if refund > 0 {
                    Promise::new(env::signer_account_id()).transfer(refund);
                }
                panic!("NFT make_collection failed");
            }
            PromiseResult::Successful(val) => {
                let (collection_id, nft_storage_usage) =
                    near_sdk::serde_json::from_slice::<(NftCollectionId, u64)>(&val)
                        .expect("NFT make_collection returned unexpected value");
                let offeror_id = env::signer_account_id();
                let offering_id = OfferingId {
                    nft_contract_id: nft_account_id.clone(),
                    collection_id,
                };
                let offering_id_hash = hash_offering_id(&offering_id);
                let fpo = FixedPriceOffering {
                    offering_id: offering_id,
                    offeror_id,
                    nft_metadata,
                    supply_total,
                    buy_now_price_yocto: buy_now_price_yocto.0,
                    min_proposal_price_yocto: if let Some(min_proposal_price_yocto) =
                        min_proposal_price_yocto
                    {
                        Some(min_proposal_price_yocto.0)
                    } else {
                        None
                    },
                    start_timestamp,
                    end_timestamp,
                    status: Unstarted,
                    supply_left: supply_total,
                    proposals: Vector::new(
                        FixedPriceOfferingStorageKey::Proposals { offering_id_hash }
                            .try_to_vec()
                            .unwrap(),
                    ),
                    next_proposal_id: 0,
                };

                let marketplace_storage_before = env::storage_usage();

                self.internal_add_fpo(&fpo);

                let storage_byte_cost = env::storage_byte_cost();
                let marketplace_storage_usage = env::storage_usage() - marketplace_storage_before;
                let required_storage_deposit =
                    (nft_storage_usage + marketplace_storage_usage) as Balance * storage_byte_cost;
                assert!(
                    attached_deposit.0 >= required_storage_deposit,
                    "The attached deposit of ({} yN) is insufficient to cover the storage costs of {} yN",
                    attached_deposit.0,
                    required_storage_deposit,
                );
                let refund = attached_deposit.0 - required_storage_deposit;
                if refund > 0 {
                    Promise::new(env::signer_account_id()).transfer(refund as Balance);
                }

                collection_id
            }
        }
    }
}

fn fpo_add_buy_now_only_storage(
    title: &str,
    media: &str,
    start_timestamp: &Option<String>,
    end_timestamp: &Option<String>,
) -> u64 {
    return 725
        + title.len() as u64
        + media.len() as u64
        + if start_timestamp.is_some() { 8 } else { 0 }
        + if end_timestamp.is_some() { 8 } else { 0 };
}

fn fpo_add_accepting_proposals_storage(
    title: &str,
    media: &str,
    start_timestamp: &Option<String>,
) -> u64 {
    return 725
        + title.len() as u64
        + media.len() as u64
        + if start_timestamp.is_some() { 8 } else { 0 }
        + 8; // end date is always some
}

fn nft_make_collection_storage(title: &str, media: &str) -> u64 {
    return 152 + title.len() as u64 + 2u64 * media.len() as u64;
}

fn nft_mint_storage(title: &str, media_url: &str, receiver_id: &str) -> u64 {
    return 635 + title.len() as u64 + media_url.len() as u64 + 2u64 * receiver_id.len() as u64;
}
