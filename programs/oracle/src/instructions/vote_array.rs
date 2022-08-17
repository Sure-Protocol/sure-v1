use std::{
    cell::RefMut,
    ops::{Mul, Sub},
};

use anchor_lang::prelude::*;

use crate::utils::SureError;

use super::VoteAccount;
pub const NUM_VOTES_IN_ARRAY_USIZE: usize = 620;
pub const NUM_VOTES_IN_ARRAY: u16 = 620;

#[account(zero_copy)]
#[repr(packed)]
#[derive(Debug, PartialEq)]
pub struct RevealedVoteArray {
    pub proposal: Pubkey,                                 // 32
    pub weighted_votes: [i128; NUM_VOTES_IN_ARRAY_USIZE], // 8*64
    pub last_index: i16,
}

impl Default for RevealedVoteArray {
    #[inline]
    fn default() -> Self {
        Self {
            proposal: Pubkey::default(),
            weighted_votes: [0; NUM_VOTES_IN_ARRAY_USIZE],
            last_index: -1,
        }
    }
}

impl RevealedVoteArray {
    pub const SPACE: usize = 32 + 16 * NUM_VOTES_IN_ARRAY_USIZE + 2;

    pub fn initialize(&mut self, proposal: Pubkey) {
        self.proposal = proposal;
        self.last_index = 0;
    }

    /// Reveal the vote and store the result in the array
    /// NOTE: tested
    pub fn reveal_vote(&mut self, vote: VoteAccount) -> Result<()> {
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
    pub fn calculate_sum_squared_difference(&self, consensus: u64) -> u128 {
        let res = self.weighted_votes[..(self.last_index + 1) as usize]
            .iter()
            .map(|w| {
                println!("w: {}", w);
                println!("c: {}", consensus);
                println!("w - X: {}", w.sub(consensus as i128));
                w.sub(consensus as i128).mul(w.sub(consensus as i128)) as u128
            })
            .sum();
        res
    }
}

#[cfg(test)]
pub mod test_revealed_vote_array {
    use anchor_lang::prelude::Pubkey;

    use crate::instructions::{
        test_vote, vote_account_proto, RevealedVoteArray, VoteAccount, NUM_VOTES_IN_ARRAY_USIZE,
    };

    /// Happy path
    #[test]
    pub fn test_reveal_vote_happy() {
        pub struct ExpectedResult {
            weighted_votes: [i128; NUM_VOTES_IN_ARRAY_USIZE],
            last_index: i16,
            sum_squared_difference: u128,
        }
        impl ExpectedResult {
            pub fn initialize() -> Self {
                Self {
                    weighted_votes: [0; NUM_VOTES_IN_ARRAY_USIZE],
                    last_index: -1,
                    sum_squared_difference: 0,
                }
            }

            pub fn set_weighted_vote(mut self, index: usize, weighted_vote: i128) -> Self {
                self.weighted_votes[index] = weighted_vote;
                self
            }

            pub fn set_last_index(mut self, index: i16) -> Self {
                self.last_index = index;
                self
            }

            pub fn set_sum_squared_difference(mut self, ssd: u128) -> Self {
                self.sum_squared_difference = ssd;
                self
            }
        }

        pub struct Test {
            name: String,
            consensus: u64,
            votes: Vec<VoteAccount>,
            expected_result: ExpectedResult,
        }

        let test_data = [
            Test {
                name: "1. Initialize and reveal vote ".to_string(),
                votes: [vote_account_proto::VoteAccountProto::initialize()
                    .set_vote_power(3_000_000)
                    .set_vote(300)
                    .build()]
                .to_vec(),
                consensus: 300,
                expected_result: ExpectedResult::initialize()
                    .set_weighted_vote(0 as usize, 900_000_000)
                    .set_sum_squared_difference(809999460000090000)
                    .set_last_index(0),
            },
            Test {
                name: "2. Reveal multiple votes".to_string(),
                votes: [
                    vote_account_proto::VoteAccountProto::initialize()
                        .set_vote_power(3_000_000)
                        .set_vote(300)
                        .build(),
                    vote_account_proto::VoteAccountProto::initialize()
                        .set_vote_power(4_000_000)
                        .set_vote(400)
                        .build(),
                ]
                .to_vec(),
                consensus: 357,
                expected_result: ExpectedResult::initialize()
                    .set_weighted_vote(0 as usize, 900_000_000)
                    .set_weighted_vote(1, 1_600_000_000)
                    .set_sum_squared_difference(3369998215000254898)
                    .set_last_index(1),
            },
        ];

        for test in test_data {
            let mut vote_array = RevealedVoteArray::default();
            for vote in test.votes {
                vote_array.reveal_vote(vote).unwrap();
            }

            assert_eq!(
                vote_array.weighted_votes, test.expected_result.weighted_votes,
                "{}: test_updated_weights",
                test.name
            );

            assert_eq!(
                vote_array.last_index, test.expected_result.last_index,
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
