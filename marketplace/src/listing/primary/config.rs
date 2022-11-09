pub const TOTAL_SUPPLY_MAX: u64 = 100;

// these define the allowed offering lifetime
// maximum duration is only applicable to proposal-accepting offering
// the rationale here is to avoid keeping proposers escrows for too long
pub const PRIMARY_LISTING_MIN_DURATION_NANO: i64 = 3600000000000;       // 1 hour
pub const PRIMARY_LISTING_MAX_DURATION_NANO: i64 = 3600000000000 * 24 * 14;       // 2 weeks

pub const NFT_MAKE_COLLECTION_STORAGE_MAX: u64 = 4376;              // worst case storage
pub const NFT_MINT_STORAGE_MAX: u64 = 3317;                         // worst case storage

pub const PRIMARY_LISTING_ADD_STORAGE_MAX: u64 = 3021;              // worst case storage TODO

// there are situations where we reserve the right to keep some Near as
// our immediate profit, such as when a proposer revokes their proposal
// (they are charged penalty)
// in such cases Near will be transfered to this account
#[allow(dead_code)]
pub const ENEFTIGO_PROFIT_ACCOUNT_ID: &str = "profit.eneftigo.near";