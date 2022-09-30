use crate::*;

use std::cmp::Ordering;
use std::fmt;

pub type ProposalId = u64;

#[derive(BorshDeserialize, BorshSerialize, Eq)]
pub struct Proposal {
    pub id: ProposalId,
    pub proposer_id: AccountId,
    pub price_yocto: u128,
}

impl Ord for Proposal {
    // best proposal comes first
    fn cmp(&self, other: &Self) -> Ordering {
        if self.price_yocto < other.price_yocto {
            Ordering::Greater
        } else if self.price_yocto == other.price_yocto {
            self.id.cmp(&other.id)      // earlier comes first
        } else {
            Ordering::Less
        }
    }
}

impl PartialOrd for Proposal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Proposal {
    fn eq(&self, other: &Self) -> bool {
        self.price_yocto == other.price_yocto && self.id == other.id
    }
}

impl fmt::Display for Proposal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ id: {}, proposer_id: {}, price_yocto: {} }}",
            self.id, self.proposer_id, self.price_yocto
        )
    }
}