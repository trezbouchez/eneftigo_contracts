use crate::{
    constants::*,
    external::{nft_contract, NftMetadata},
    listing::{
        constants::*,
        secondary::{
            config::*, internal::hash_secondary_listing_id, lib::SecondaryListingStorageKey,
        },
        status::ListingStatus,
    },
    *,
};
use chrono::DateTime;
use near_sdk::{collections::Vector, json_types::U128};
use url::Url;

// const NFT_MAKE_COLLECTION_GAS: Gas = Gas(5_000_000_000_000); // highest measured 3_920_035_683_889
// const NFT_MAKE_COLLECTION_COMPLETION_GAS: Gas = Gas(6_000_000_000_000); // highest measured 5_089_357_803_858

#[near_bindgen]
impl MarketplaceContract {
    pub(crate) fn secondary_listing_add_buy_now_only(
        &mut self,
        title: String,
        nft_contract_id: AccountId,
        approval_id: u64,
        token_id: NftId,
        nft_metadata: NftMetadata,
        buy_now_price_yocto: U128,
        start_date: Option<String>, // if missing, it's start accepting bids when this transaction is mined
        end_date: Option<String>,
    ) {
        let seller_id = env::predecessor_account_id();

        // Is deposit sufficient to cover the storage in the worst-case scenario?
        let storage_byte_cost = env::storage_byte_cost();
        let current_deposit: Balance = self.storage_deposits.get(&seller_id).unwrap_or(0);
        let marketplace_worst_case_storage_cost =
            SECONDARY_LISTING_ADD_STORAGE_MAX as Balance * storage_byte_cost;
        let worst_case_storage_cost = marketplace_worst_case_storage_cost;
        assert!(
            current_deposit >= worst_case_storage_cost,
            "Your storage deposit is too low. Must be {} yN to process transaction. Please increase your deposit.",
            worst_case_storage_cost
        );

        // Is listing length ok?
        assert!(
            title.len() <= MAX_LISTING_TITLE_LEN,
            "Title length cannot exceed {} characters",
            MAX_LISTING_TITLE_LEN
        );

        // Is URL present and valid?
        let media_url = nft_metadata.media.clone().expect("Missing NFT media");
        assert!(
            Url::parse(&media_url).is_ok(),
            "NFT media URL is invalid"
        );

        // Isn't the price too low?
        assert!(
            buy_now_price_yocto.0 >= MIN_BUY_NOW_PRICE_YOCTO,
            "Price cannot be lower than {} yoctoNear",
            MIN_BUY_NOW_PRICE_YOCTO
        );

        // Is the price multiple of marketplace price unit?
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
                assert!(
                    duration >= SECONDARY_LISTING_MIN_DURATION_NANO,
                    "Listing duration too short"
                );
            } else {
                let current_block_timestamp = env::block_timestamp() as i64;
                let duration = end_timestamp - current_block_timestamp;
                assert!(
                    duration >= SECONDARY_LISTING_MIN_DURATION_NANO,
                    "Listing duration too short"
                );
            }
        }

        let listing_id = SecondaryListingId {
            nft_contract_id,
            token_id,
        };
        let listing_id_hash = hash_secondary_listing_id(&listing_id);
        let listing = SecondaryListing {
            id: listing_id,
            seller_id: seller_id.clone(),
            approval_id,
            nft_metadata,
            buy_now_price_yocto: buy_now_price_yocto.0,
            min_proposal_price_yocto: None,
            start_timestamp,
            end_timestamp,
            status: ListingStatus::Unstarted,
            proposals: Vector::new(
                SecondaryListingStorageKey::Proposals { listing_id_hash }
                    .try_to_vec()
                    .unwrap(),
            ),
            next_proposal_id: 0,
        };

        let marketplace_storage_before = env::storage_usage();

        self.internal_add_secondary_listing(&listing);

        let storage_byte_cost = env::storage_byte_cost();
        let marketplace_storage = env::storage_usage() - marketplace_storage_before;
        let marketplace_storage_cost = marketplace_storage as Balance * storage_byte_cost;
        let nft_storage_cost: Balance = 0u128;  // TODO: 
        let total_storage_cost = marketplace_storage_cost + nft_storage_cost;
        let current_deposit = self
            .storage_deposits
            .get(&seller_id)
            .expect("Could not find seller storage deposit record");
        let updated_deposit = if current_deposit >= total_storage_cost {
            current_deposit - total_storage_cost
        } else {
            0 // should never happen; TODO: log warning to review storage deposit logic
        };
        self.storage_deposits.insert(&seller_id, &updated_deposit);
    }
}
