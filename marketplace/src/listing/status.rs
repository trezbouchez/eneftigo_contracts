use crate::*;

#[derive(BorshDeserialize, BorshSerialize, PartialEq)]
#[derive(Serialize,Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ListingStatus {
    Unstarted,
    Running,
    Ended,
}

impl ListingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ListingStatus::Unstarted => "Unstarted",
            ListingStatus::Running => "Running",
            ListingStatus::Ended => "Ended",
        }
    }
}
