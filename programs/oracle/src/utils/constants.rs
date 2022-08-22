use anchor_lang::solana_program::clock::SECONDS_PER_DAY;

// Seeds
pub const SURE_ORACLE_SEED: &str = "sure-oracle";
pub const SURE_ORACLE_VOTE_SEED: &str = "sure-oracle-vote";

// voting
pub const VOTING_LENGTH_SECONDS: i64 = SECONDS_PER_DAY as i64;
// voting fraction required in 1/x
pub const VOTING_FRACTION_REQUIRED: u64 = 10;

// vote stake calculation as 1/x
pub const VOTE_STAKE_RATE: f64 = 100.; // 1%

pub const TEST_START_TIME: i64 = 1660681219;
