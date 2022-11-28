use std::{
    cell::RefMut,
    ops::{Mul, Sub},
};

use anchor_lang::prelude::*;

use crate::utils::SureError;

use super::VoteAccount;
pub const NUM_VOTES_IN_ARRAY_USIZE: usize = 1024;
pub const NUM_VOTES_IN_ARRAY: u16 = 1024;

#[account(zero_copy)]
#[repr(packed)]
#[derive(Debug, PartialEq)]
pub struct RevealedVoteArray {
    pub bump: u8,         // 1
    pub proposal: Pubkey, // 32
    /// Q32.32
    pub weighted_votes: [i64; NUM_VOTES_IN_ARRAY_USIZE], // 8*
    pub last_index: i16,
}

impl Default for RevealedVoteArray {
    #[inline]
    fn default() -> Self {
        Self {
            bump: 0,
            proposal: Pubkey::default(),
            weighted_votes: [0; NUM_VOTES_IN_ARRAY_USIZE],
            last_index: -1,
        }
    }
}

impl RevealedVoteArray {
    pub const SPACE: usize = 1 + 32 + 8 * NUM_VOTES_IN_ARRAY_USIZE + 2;

    pub fn initialize(&mut self, proposal: Pubkey, bump: u8) {
        self.bump = bump;
        self.proposal = proposal;
        self.last_index = 0;
    }

    /// Reveal the vote and store the result in the array
    /// NOTE: tested
    pub fn reveal_vote(&mut self, vote: &VoteAccount) -> Result<()> {
        let next_index = self.last_index + 1;
        if next_index.abs() as u16 > NUM_VOTES_IN_ARRAY {
            return Err(SureError::FullRevealList.into());
        }
        if !vote.revealed_vote {
            return Err(SureError::VoteNotRevealed.into());
        }

        let weighted_vote = vote.calculate_weighted_vote()?;
        self.weighted_votes[next_index as usize] = weighted_vote;
        self.last_index = next_index;
        Ok(())
    }

    /// Calculate
    /// sum_i^n (w_i - x^bar)^2
    /// NOTE: tested
    ///
    /// ### Arguments
    /// * consensus: i32.32
    ///
    /// ### Returns
    /// * sum_i^n (w_i - x^bar)^2:  Q32.32
    pub fn calculate_sum_squared_difference(&self, consensus: i64) -> u64 {
        let mut ssd = 0;
        let mut i = 0;
        while i <= self.last_index + 1 {
            let sub = self.weighted_votes[i as usize].sub(consensus) as i128;
            // i32.32 x i32.32 -> i64.64
            let sub_squared_ix64 = sub.mul(sub) as u128;
            // Q64.64 >> 32 -> Q32.32
            let sub_squared_ix32 = (sub_squared_ix64 >> 32) as u64;
            ssd += sub_squared_ix32;
            i += 1;
        }
        ssd
    }
}

#[cfg(test)]
pub mod test_revealed_vote_array {
    use anchor_lang::prelude::Pubkey;

    use crate::states::{
        vote_account_proto, RevealedVoteArray, VoteAccount, NUM_VOTES_IN_ARRAY_USIZE,
    };

    /// Happy path
    #[test]
    pub fn test_reveal_vote_happy() {
        pub struct ExpectedResult {
            weighted_votes: [i64; NUM_VOTES_IN_ARRAY_USIZE],
            last_index: i16,
            sum_squared_difference: u64,
        }
        impl ExpectedResult {
            pub fn initialize() -> Self {
                Self {
                    weighted_votes: [0; NUM_VOTES_IN_ARRAY_USIZE],
                    last_index: -1,
                    sum_squared_difference: 0,
                }
            }

            pub fn set_weighted_vote(mut self, index: usize, weighted_vote: i64) -> Self {
                self.weighted_votes[index] = weighted_vote;
                self
            }

            pub fn set_last_index(mut self, index: i16) -> Self {
                self.last_index = index;
                self
            }

            pub fn set_sum_squared_difference(mut self, ssd: u64) -> Self {
                self.sum_squared_difference = ssd;
                self
            }
        }

        pub struct Test {
            name: String,
            consensus: i64, // i32.32
            votes: Vec<VoteAccount>,
            expected_result: ExpectedResult,
        }

        let test_data = [
            Test {
                name: "1. Initialize and reveal vote ".to_string(),
                votes: [vote_account_proto::VoteAccountProto::initialize()
                    .set_vote_power(3_000_000, 6)
                    .set_vote(300)
                    .build()]
                .to_vec(),
                consensus: (300 as i64) << 32,
                expected_result: ExpectedResult::initialize()
                    .set_weighted_vote(0 as usize, 900)
                    .set_sum_squared_difference(360000)
                    .set_last_index(0),
            },
            Test {
                name: "2. Reveal multiple votes".to_string(),
                votes: [
                    vote_account_proto::VoteAccountProto::initialize()
                        .set_vote_power(3_000_000, 6)
                        .set_vote(300)
                        .build(),
                    vote_account_proto::VoteAccountProto::initialize()
                        .set_vote_power(4_000_000, 6)
                        .set_vote(400)
                        .build(),
                ]
                .to_vec(),
                consensus: (300 as i64) << 32,
                expected_result: ExpectedResult::initialize()
                    .set_weighted_vote(0 as usize, 900)
                    .set_weighted_vote(1, 1_600)
                    .set_sum_squared_difference(1839898)
                    .set_last_index(1),
            },
        ];

        for test in test_data {
            let mut vote_array = RevealedVoteArray::default();
            for vote in test.votes {
                vote_array.reveal_vote(&vote).unwrap();
            }

            let weighted_votes = vote_array.weighted_votes;
            assert_eq!(
                weighted_votes, test.expected_result.weighted_votes,
                "{}: test_updated_weights",
                test.name
            );

            let last_index = vote_array.last_index;
            assert_eq!(
                last_index, test.expected_result.last_index,
                "{}: test_last_index",
                test.name
            );

            assert_eq!(
                vote_array.calculate_sum_squared_difference(test.consensus),
                test.expected_result.sum_squared_difference,
                "{}: test_expected_ssd",
                test.name
            );
        }
    }
}

pub struct VoteArrayList<'info> {
    arrays: Vec<RefMut<'info, RevealedVoteArray>>,
}

impl<'info> VoteArrayList<'info> {
    pub fn new(
        va0: RefMut<'info, RevealedVoteArray>,
        va1: Option<RefMut<'info, RevealedVoteArray>>,
        va2: Option<RefMut<'info, RevealedVoteArray>>,
    ) -> Self {
        let mut vote_array_list = Vec::with_capacity(3);
        vote_array_list.push(va0);
        if va1.is_some() {
            vote_array_list.push(va1.unwrap());
        }
        if va2.is_some() {
            vote_array_list.push(va2.unwrap())
        }

        Self {
            arrays: vote_array_list,
        }
    }
}
