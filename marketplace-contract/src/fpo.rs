use crate::*;

use chrono::{DateTime, TimeZone, Utc, Duration};

// use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
// use near_sdk::serde::{Deserialize, Serialize};
// use near_sdk::json_types::{U128, U64};

// TODO: tweak these if needed
//const STORAGE_PER_FIXED_PRICE_OFFERING: u128 = 1000 * STORAGE_PRICE_PER_BYTE;   // TODO: measure and tweak
const NFT_MAX_SUPPLY_LIMIT: u128 = 100;
const FPO_MIN_PRICE_YOCTO: u128 = 1;

// contains fixed-price offering parameters

#[derive(BorshDeserialize, BorshSerialize/*, Serialize, Deserialize*/)]
// #[serde(crate = "near_sdk::serde")]
pub struct FixedPriceProposal {
    pub proposer_id: AccountId,
    pub price_yocto: u128,
}

#[derive(BorshDeserialize, BorshSerialize/*, Serialize, Deserialize*/)]
// #[serde(crate = "near_sdk::serde")]
pub struct FixedPriceOffering {
    pub nft_contract_id: AccountId,
    pub offeror_id: AccountId,
    pub nft_max_supply: u128,
    pub price_yocto: u128,
    pub end_timestamp: Option<i64>,         // nanoseconds since 1970-01-01
    pub nft_metadata: NFTMetadata,
    pub nft_supply_sold: u128,
    pub next_proposal_price_yocto: u128,
    pub winning_proposals: UnorderedSet<FixedPriceProposal>,
    pub loosing_proposals: UnorderedSet<FixedPriceProposal>,
}

#[ext_contract(ext_self)]
trait ExtSelf {
    fn fpo_resolve_purchase(
        &mut self,
        buyer_id: AccountId,
        price: U128,
    ) -> Promise;
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
        nft_max_supply: U128,
        price_yocto: U128,
        nft_metadata: NFTMetadata,
        duration_days: Option<U64>,         // if duration is set, end_date must be None 
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
            nft_max_supply.0 > 0 && nft_max_supply.0 <= NFT_MAX_SUPPLY_LIMIT,
            "Max NFT supply must be between 1 and {}.", NFT_MAX_SUPPLY_LIMIT
        );

        // make sure it's not yet listed
        assert!(
            self.fpos_by_contract_id .get(&nft_contract_id).is_none(),
            "Already listed"
        );

        // TODO: price cannot be too low:
        assert!(
            price_yocto.0 >= FPO_MIN_PRICE_YOCTO,
            "Price cannot be lower than {} yocto Near", FPO_MIN_PRICE_YOCTO
        );
        
        // get initial storage
        let initial_storage_usage = env::storage_usage();

        // end timestamp
        let end_timestamp: Option<i64>;
        if let Some(duration_days) = duration_days {
            assert!(end_date.is_none(), "Either duration or end date can be provided, not both.");
            let current_block_timestamp_nanos = env::block_timestamp() as i64;
            let current_block_datetime = Utc.ymd(1970, 1, 1).and_hms(0, 0, 0) + Duration::nanoseconds(current_block_timestamp_nanos);
            let end_datetime = current_block_datetime + Duration::days(duration_days.0 as i64);
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

        let contract_id_hash = hash_account_id(&nft_contract_id);
        let fpo = FixedPriceOffering {
            nft_contract_id,
            offeror_id,
            nft_max_supply: nft_max_supply.0,
            price_yocto: price_yocto.0,
            nft_metadata,
            end_timestamp,
            nft_supply_sold: 0,
            next_proposal_price_yocto: 1,
            winning_proposals: UnorderedSet::new(StorageKey::FPOInnerWinning {
                account_id_hash: contract_id_hash,
            }),
            loosing_proposals: UnorderedSet::new(StorageKey::FPOInnerLoosing {
                account_id_hash: contract_id_hash,
            }),
        };

        self.fpos_by_contract_id.insert(&fpo.nft_contract_id, &fpo);

        self.internal_add_fpo_to_offeror(&fpo.offeror_id, &fpo.nft_contract_id);

        // calculate the extra storage used by FPO entries
        let required_storage_in_bytes = env::storage_usage() - initial_storage_usage;

        // refund any excess storage if the user attached too much. Panic if they didn't attach enough to cover what's required.
        refund_deposit(required_storage_in_bytes);
    }

    #[payable]
    pub fn fpo_buy(
        &mut self,
        nft_contract_id: AccountId,
        buyer_id: AccountId,
    ) {
        // make sure it's called by marketplace 
        assert_eq!(
            &env::predecessor_account_id(),
            &self.owner_id,
            "Only Eneftigo Marketplace owner can place order."
        );

        // get FPO
        let mut fpo = self.fpos_by_contract_id.get(&nft_contract_id).expect("Could not find NFT listing");

        // ensure there's supply left
        assert!(
            fpo.nft_supply_sold < fpo.nft_max_supply,
            "You are late. All NFTs have been sold."
        );

        // ensure the attached balance is sufficient
        let deposit_yocto = env::attached_deposit();
        assert!(
            deposit_yocto >= fpo.price_yocto, 
            "Attached deposit must be sufficient to pay the price: {:?} yocto Near", fpo.price_yocto
        );

        // process the purchase
        self.fpo_process_purchase(
            nft_contract_id,
            fpo.nft_supply_sold.to_string(),
            fpo.nft_metadata,
            deposit_yocto,
            buyer_id,
        );

        //get the refund amount from the attached deposit - required cost
        // let refund = attached_deposit - required_cost;
    
        //if the refund is greater than 1 yocto NEAR, we refund the predecessor that amount
        // if refund > 1 {
        //     Promise::new(env::predecessor_account_id()).transfer(refund);
        // }
    }

    // will transfer and get the payout from the nft contract, will remove listing if no supply left
    #[private]
    pub fn fpo_process_purchase(
        &mut self,
        nft_contract_id: AccountId,
        token_id: TokenId,
        nft_metadata: NFTMetadata,
        price_yocto: u128,
        buyer_id: AccountId,
    ) -> Promise {
        // initiate a cross contract call to the nft contract. This will mint the token, transfer it to the buyer
        //a payout object used for the market to distribute funds to the appropriate accounts.
        ext_contract::nft_mint(
            token_id,
            nft_metadata,
            buyer_id.clone(),
            None,       // TODO: setup perpetual royalties
            nft_contract_id,
            1,
            GAS_FOR_NFT_MINT,
        )
        .then(
            ext_self::fpo_resolve_purchase(
                buyer_id,
                U128(price_yocto),
                env::current_account_id(), // we are invoking this function on the current contract
                NO_DEPOSIT, // don't attach any deposit
                GAS_FOR_NFT_MINT, // GAS attached to the mint call
            )
        )
    }

    #[private]
    pub fn fpo_resolve_purchase(
        &mut self,
        buyer_id: AccountId,
        price: U128,
    ) {
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