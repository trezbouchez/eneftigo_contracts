use crate::*;

use std::cmp::Ordering;
use std::fmt;

use near_sdk::collections::Vector;

pub type ProposalId = u128;

// contains fixed-price offering parameters

#[derive(BorshStorageKey, BorshSerialize)]
pub enum FixedPriceOfferingStorageKey {
    Proposals {
        nft_contract_id_hash: CryptoHash,
    },
    ProposalsByProposer {
        nft_contract_id_hash: CryptoHash,
    },
    ProposalsByProposerInner {
        nft_contract_id_hash: CryptoHash,
        proposer_id_hash: CryptoHash,
    },
    AcceptableProposals {
        nft_contract_id_hash: CryptoHash,
    },
}

#[derive(BorshDeserialize, BorshSerialize /*, Serialize, Deserialize*/, Eq)]
// #[serde(crate = "near_sdk::serde")]
pub struct FixedPriceOfferingProposal {
    pub id: ProposalId,
    pub proposer_id: AccountId,
    pub price_yocto: u128,
    pub is_acceptable: bool,
}

impl Ord for FixedPriceOfferingProposal {
    // lower looses, greater wins
    fn cmp(&self, other: &Self) -> Ordering {
        if self.price_yocto < other.price_yocto {
            // lower proposed price
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
        self.price_yocto == other.price_yocto && self.id == other.id
    }
}

impl fmt::Display for FixedPriceOfferingProposal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ id: {}, proposer_id: {}, price_yocto: {}, is_acceptable: {} }}",
            self.id, self.proposer_id, self.price_yocto, self.is_acceptable
        )
    }
}

#[derive(BorshDeserialize, BorshSerialize, PartialEq)]
pub enum FixedPriceOfferingStatus {
    Unstarted,
    Running,
    Ended,
}

impl FixedPriceOfferingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            FixedPriceOfferingStatus::Unstarted => "Unstarted",
            FixedPriceOfferingStatus::Running => "Running",
            FixedPriceOfferingStatus::Ended => "Ended",
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct FixedPriceOffering {
    pub nft_contract_id: AccountId,
    pub offeror_id: AccountId,
    pub supply_total: u64,
    pub buy_now_price_yocto: u128,
    pub min_proposal_price_yocto: Option<u128>, // if None then no proposals will be accepted
    pub start_timestamp: Option<i64>,           // nanoseconds since 1970-01-01
    pub end_timestamp: Option<i64>,             // nanoseconds since 1970-01-01
    pub status: FixedPriceOfferingStatus, // will be updated when any buyer transaction is mined
    // pub nft_metadata: TokenMetadata,
    pub supply_left: u64,
    pub proposals: LookupMap<ProposalId, FixedPriceOfferingProposal>,
    pub proposals_by_proposer: LookupMap<AccountId, UnorderedSet<ProposalId>>,
    pub acceptable_proposals: Vector<ProposalId>, // by ascending price then by ascending id (submission order)
    pub next_proposal_id: u128,
}

impl fmt::Display for FixedPriceOffering {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "(nft_contract_id: {}\n offeror_id{}\n)",
            self.nft_contract_id, self.offeror_id
        )
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
