use std::ops::{Div, Mul};

use anchor_lang::{prelude::*, solana_program::clock::SECONDS_PER_DAY};
use anchor_spl::token::Mint;

use crate::utils::VOTING_FRACTION_REQUIRED;

#[account]
pub struct Config {
    pub bump: u8, //                        1 byte

    /// voting period in seconds
    pub voting_length_seconds: i64, //      8 bytes

    /// the lenght of the reveal period
    /// in seconds
    pub reveal_length_seconds: i64, //      8 bytes

    /// the default required votes to reach
    /// quorum.
    pub default_required_votes: u64, //     8 bytes

    /// the minimum amount of tokens that must
    /// be staked on a proposal
    pub minimum_proposal_stake: u64, //     8 bytes

    /// the 1/x of the voting power that needs
    /// to be staked in order to vote
    pub vote_stake_rate: u32, //            4 bytes

    /// the 1/x of the total voting escrow
    /// that's going to the protocol
    pub protocol_fee_rate: u32, //          4 bytes

    /// official mint of pool
    pub token_mint: Pubkey, //              32 bytes

    /// who can collect the rewards
    pub protocol_authority: Pubkey, //      32 bytes

    pub initialized: bool, //               1 byte
}

impl Config {
    pub const SPACE: usize = 1 + 4 * 8 + 2 * 4 + 2 * 32 + 1;

    pub fn initialize(&mut self, token_mint: &Account<Mint>, protocol_authority: Pubkey) {
        let mint = token_mint.key();
        let token_supply = token_mint.supply;
        let decimals = token_mint.decimals;

        // default voting and reveal time is one day
        self.voting_length_seconds = SECONDS_PER_DAY as i64;
        self.reveal_length_seconds = SECONDS_PER_DAY as i64;

        self.default_required_votes = token_supply.div(VOTING_FRACTION_REQUIRED);
        self.minimum_proposal_stake = 10_u64.mul(10_u64.pow(decimals as u32));

        // default to 1%
        self.vote_stake_rate = 100;

        // default to 0.02 of vote pool
        self.protocol_fee_rate = 50;

        self.token_mint = mint;
        self.protocol_authority = protocol_authority;
        self.initialized = true;
    }

    /// update
    ///
    /// allow owner of config to update parameters
    ///
    pub fn update(
        &mut self,
        voting_length_seconds: i64,
        reveal_length_seconds: i64,
        required_votes: u64,
        minimum_proposal_stake: u64,
        vote_stake_rate: u32,
        protocol_fee_rate: u32,
    ) {
        self.voting_length_seconds = voting_length_seconds;
        self.reveal_length_seconds = reveal_length_seconds;

        self.default_required_votes = required_votes;
        self.minimum_proposal_stake = minimum_proposal_stake;

        // default to 1%
        self.vote_stake_rate = vote_stake_rate;

        // default to 0.02 of vote pool
        self.protocol_fee_rate = protocol_fee_rate;
    }

    /// update voting length
    pub fn update_voting_lengths(
        &mut self,
        voting_length_seconds: i64,
        reveal_length_seconds: i64,
    ) -> Result<()> {
        self.voting_length_seconds = voting_length_seconds;
        self.reveal_length_seconds = reveal_length_seconds;
        Ok(())
    }

    pub fn update_voting_length(&mut self, voting_length_seconds: i64) -> Result<()> {
        self.voting_length_seconds = voting_length_seconds;
        Ok(())
    }

    pub fn update_reveal_length(&mut self, reveal_length_seconds: i64) -> Result<()> {
        self.reveal_length_seconds = reveal_length_seconds;
        Ok(())
    }

    /// update required votes
    pub fn update_required_votes(&mut self, required_votes: u64) -> Result<()> {
        self.default_required_votes = required_votes;
        Ok(())
    }

    pub fn update_proposal_minimum_stake(&mut self, minimum_proposal_stake: u64) -> Result<()> {
        self.minimum_proposal_stake = minimum_proposal_stake;
        Ok(())
    }

    pub fn update_vote_stake_rate(&mut self, vote_stake_rate: u32) -> Result<()> {
        self.vote_stake_rate = vote_stake_rate;
        Ok(())
    }

    pub fn update_protocol_fee_rate(&mut self, protocol_fee_rate: u32) -> Result<()> {
        self.protocol_fee_rate = protocol_fee_rate;
        Ok(())
    }
}
