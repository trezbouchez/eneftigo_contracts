pub const SECONDARY_LISTING_ADD_STORAGE_MAX: u64 = 3021;            // worst case storage TODO

// these define the allowed offering lifetime
// maximum duration is only applicable to proposal-accepting offering
// the rationale here is to avoid keeping proposers escrows for too long
pub const SECONDARY_LISTING_MIN_DURATION_NANO: i64 = 3600000000000;       // 1 hour
pub const SECONDARY_LISTING_MAX_DURATION_NANO: i64 = 3600000000000 * 24 * 14;       // 2 weeks
