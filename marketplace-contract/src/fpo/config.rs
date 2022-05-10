pub const TOTAL_SUPPLY_MAX: u64 = 100;
pub const MIN_BUY_NOW_PRICE_YOCTO: u128 = 1000;
pub const PRICE_STEP_YOCTO: u128 = 10;

// these define the allowed offering lifetime
// maximum duration is only applicable to proposal-accepting offering
// the rationale here is to avoid keeping proposers escrows for too long
pub const MIN_DURATION_NANO: i64 = 3600000000000;       // 1 hour
pub const MAX_DURATION_NANO: i64 = 3600000000000 * 24 * 14;       // 2 weeks

// we act as an escrow when placing proposals; users must deposit
// the full price of the proposal must be deposited in order to be 
// accepted and will be either paid to the seller or returned;
// also, we allow proposers to revoke their proposal at the cost of
// the penalty which is set (in percentage) by this constant
pub const PROPOSAL_REVOKE_PENALTY_RATE: u128 = 10;     // percent

// there are situations where we reserve the right to keep some Near as
// our immediate profit, such as when a proposer revokes their proposal
// (they are charged penalty)
// in such cases Near will be transfered to this account
pub const ENEFTIGO_PROFIT_ACCOUNT_ID: &str = "profit.eneftigo.near";