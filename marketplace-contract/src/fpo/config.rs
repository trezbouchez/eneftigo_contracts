pub const TOTAL_SUPPLY_MAX: u64 = 100;
pub const MIN_PRICE_YOCTO: u128 = 1000;
pub const PRICE_STEP_YOCTO: u128 = 10;

// TODO: it is important to set the minimum effective yocto deposit
// (determined by MIN_PRICE_YOCTO and PROPOSAL_DEPOSIT_RATE)
// to be greater than or equal the storage cost per proposal; this
// prevents the mallicious attempt to drain the contract out of its
// Near balance known as "million cheap data additions" attack
// https://docs.near.org/docs/concepts/storage-staking
pub const PROPOSAL_DEPOSIT_RATE: u128 = 10;     // percent
