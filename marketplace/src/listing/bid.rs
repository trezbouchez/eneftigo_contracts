use crate::*;

use std::cmp::Ordering;
use std::fmt;

pub type BidId = u64;

#[derive(BorshDeserialize, BorshSerialize, Eq)]
pub struct Bid {
    pub id: BidId,
    pub bidder_id: AccountId,
    pub amount_yocto: u128,
}

impl Ord for Bid {
    // best proposal comes first
    fn cmp(&self, other: &Self) -> Ordering {
        if self.amount_yocto < other.amount_yocto {
            Ordering::Greater
        } else if self.amount_yocto == other.amount_yocto {
            self.id.cmp(&other.id)      // earlier comes first
        } else {
            Ordering::Less
        }
    }
}

impl PartialOrd for Bid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Bid {
    fn eq(&self, other: &Self) -> bool {
        self.amount_yocto == other.amount_yocto && self.id == other.id
    }
}

impl fmt::Display for Bid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ id: {}, bidder_id: {}, amount_yocto: {} }}",
            self.id, self.bidder_id, self.amount_yocto
        )
    }
}