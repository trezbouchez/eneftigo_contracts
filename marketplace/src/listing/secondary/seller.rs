use crate::{
    // constants::*,
    external::NftMetadata,
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
use near_sdk::{
    collections::Vector,
    json_types::{U128},
};
use url::Url;

// const NFT_MAKE_COLLECTION_GAS: Gas = Gas(5_000_000_000_000); // highest measured 3_920_035_683_889
// const NFT_MAKE_COLLECTION_COMPLETION_GAS: Gas = Gas(6_000_000_000_000); // highest measured 5_089_357_803_858

#[near_bindgen]
impl MarketplaceContract {
    pub(crate) fn secondary_listing_add(
        &mut self,
        owner_id: AccountId,
        nft_contract_id: AccountId,
        approval_id: u64,
        token_id: NftId,
        nft_metadata: NftMetadata,
        price_yocto: Option<U128>,
        min_bid_yocto: Option<U128>, // if None, only buy now is allowed
        start_date: Option<String>, // if missing, it'll start accepting bids when this transaction is mined
        end_date: Option<String>,
    ) {
        let price_yocto = price_yocto.map(|p| p.0);
        let min_bid_yocto = min_bid_yocto.map(|b| b.0);

        let seller_id = env::predecessor_account_id();

        // Is deposit sufficient to cover the storage in the worst-case scenario?
        let storage_byte_cost = env::storage_byte_cost();
        let current_deposit: Balance = self.storage_deposits.get(&owner_id).unwrap_or(0);
        let marketplace_worst_case_storage_cost =
            SECONDARY_LISTING_ADD_STORAGE_MAX as Balance * storage_byte_cost;
        let worst_case_storage_cost = marketplace_worst_case_storage_cost;
        assert!(
            current_deposit >= worst_case_storage_cost,
            "Your storage deposit is too low. Must be {} yN to process transaction. Please increase your deposit.",
            worst_case_storage_cost
        );

        // Has title of ok length?
        let title = nft_metadata.title.clone().expect("Token must have a title");
        assert!(
            title.len() <= MAX_LISTING_TITLE_LEN,
            "Title length cannot exceed {} characters",
            MAX_LISTING_TITLE_LEN
        );

        // Is URL present and valid?
        let media_url = nft_metadata.media.clone().expect("Missing NFT media");
        assert!(Url::parse(&media_url).is_ok(), "NFT media URL is invalid");

        // Is the price ok?
        if let Some(price_yocto) = price_yocto {
            assert!(
                price_yocto >= MIN_PRICE_YOCTO,
                "Price cannot be lower than {} yoctoNear",
                MIN_PRICE_YOCTO
            );

            // Is the price multiple of marketplace price unit?
            assert!(
                price_yocto % PRICE_STEP_YOCTO == 0,
                "Price must be integer multiple of {} yoctoNear",
                PRICE_STEP_YOCTO
            );
        }

        // Is min bid ok?
        if let Some(min_bid_yocto) = min_bid_yocto {
            assert!(
                min_bid_yocto >= MIN_BID_YOCTO,
                "Bid cannot be lower than {} yoctoNear",
                MIN_BID_YOCTO
            );

            // Is the price multiple of marketplace price unit?
            assert!(
                min_bid_yocto % BID_STEP_YOCTO == 0,
                "Bid must be integer multiple of {} yoctoNear",
                BID_STEP_YOCTO
            );
        }

        // the logic here is:
        // - if start timestamp is missing, current block timestamp is used
        // - for bid-accepting listings the end date must be set and the duration cannot exceed
        //   max allowed (this is to prevent keeping bid deposits indefinitely)
        // - for buy-now-only listings, there's no upper limit on duration

        let is_accepting_bids = min_bid_yocto.is_some();
        let current_block_timestamp = env::block_timestamp() as i64;

        let start_timestamp = if let Some(start_date_str) = start_date {
            let start_datetime = DateTime::parse_from_rfc3339(&start_date_str).expect(
                "Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)",
            );
            let start_timestamp = start_datetime.timestamp_nanos();
            assert!(
                start_timestamp >= current_block_timestamp,
                "Start date into the past"
            );
            start_timestamp
        } else {
            current_block_timestamp
        };

        let end_timestamp: Option<i64> = if let Some(end_date_str) = end_date {
            let end_datetime = DateTime::parse_from_rfc3339(&end_date_str).expect(
                "Wrong date format. Must be ISO8601/RFC3339 (f.ex. 2022-01-22T11:20:55+08:00)",
            );
            let end_timestamp = end_datetime.timestamp_nanos();
            let current_block_timestamp = env::block_timestamp() as i64;
            assert!(
                end_timestamp >= current_block_timestamp,
                "End date into the past"
            );
            Some(end_timestamp)
            // let end_datetime_str = (Utc.ymd(1970, 1, 1).and_hms(0, 0, 0) + Duration::nanoseconds(end_timestamp_nanos)).to_rfc3339();
            // env::log_str(&end_datetime_str);
        } else {
            None
        };

        if let Some(end_timestamp) = end_timestamp {
            // end timestamp set
            let duration = end_timestamp - start_timestamp;
            assert!(
                duration >= SECONDARY_LISTING_MIN_DURATION_NANO,
                "Listing duration too short"
            );
            if is_accepting_bids {
                assert!(
                    duration <= SECONDARY_LISTING_MAX_DURATION_NANO,
                    "Listing duration too long"
                );
            }
        } else {
            assert!(
                !is_accepting_bids,
                "End date must be set for bid-accepting listing"
            );
        }

        let listing_id = SecondaryListingId {
            nft_contract_id,
            token_id,
        };
        let listing_id_hash = hash_secondary_listing_id(&listing_id);
        let listing = SecondaryListing {
            id: listing_id,
            seller_id: owner_id.clone(),
            approval_id,
            nft_metadata,
            price_yocto,
            min_bid_yocto,
            start_timestamp,
            end_timestamp,
            status: ListingStatus::Unstarted,
            bids: Vector::new(
                SecondaryListingStorageKey::Bids { listing_id_hash }
                    .try_to_vec()
                    .unwrap(),
            ),
            next_bid_id: 0,
        };

        let marketplace_storage_before = env::storage_usage();

        self.internal_add_secondary_listing(&listing);

        let storage_byte_cost = env::storage_byte_cost();
        let marketplace_storage = env::storage_usage() - marketplace_storage_before;
        let marketplace_storage_cost = marketplace_storage as Balance * storage_byte_cost;
        let nft_storage_cost: Balance = 0u128; // TODO:
        let total_storage_cost = marketplace_storage_cost + nft_storage_cost;
        let current_deposit = self
            .storage_deposits
            .get(&owner_id)
            .expect("Could not find seller storage deposit record");
        let updated_deposit = if current_deposit >= total_storage_cost {
            current_deposit - total_storage_cost
        } else {
            0 // should never happen; TODO: log warning to review storage deposit logic
        };
        self.storage_deposits.insert(&owner_id, &updated_deposit);
    }

    pub fn secondary_listing_conclude(
        &mut self,
        owner_id: AccountId,
        nft_contract_id: AccountId,
        token_id: String,
    ) {
        // make sure it's the seller who's calling this
        assert!(
            env::predecessor_account_id() == owner_id,
            "Only the seller can conclude a listing"
        );

        let listing_id = SecondaryListingId {
            nft_contract_id,
            token_id,
        };

        let listing = self
            .secondary_listings_by_id
            .get(&listing_id)
            .expect("Could not find this listing");

        let storage_before = env::storage_usage();

        let removed_listing = self.internal_remove_secondary_listing(&listing_id);

        let storage_after = env::storage_usage();
        let storage_freed = storage_before - storage_after;
        let refunded_deposit = storage_freed as Balance * env::storage_byte_cost();
        let current_deposit = self
            .storage_deposits
            .get(&removed_listing.seller_id)
            .expect("Could not find seller's storage deposit record");
        let updated_deposit = current_deposit + refunded_deposit;
        self.storage_deposits
            .insert(&removed_listing.seller_id, &(updated_deposit));
    }
}
