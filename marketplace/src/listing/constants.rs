pub const MAX_LISTING_TITLE_LEN: usize = 128;

pub const MIN_PRICE_YOCTO: u128 = 1000;
pub const PRICE_STEP_YOCTO: u128 = 1000;

pub const MIN_BID_YOCTO: u128 = 1000;
pub const BID_STEP_YOCTO: u128 = 1000;

// we act as an escrow when placing proposals; users must deposit
// the full price of the proposal must be deposited in order to be 
// accepted and will be either paid to the seller or returned;
// also, we allow proposers to revoke their proposal at the cost of
// the penalty which is set (in percentage) by this constant
pub const PROPOSAL_REVOKE_FEE_RATE: u128 = 10;     // percent


