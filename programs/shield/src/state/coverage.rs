use agnostic_orderbook::instruction::cancel_order::Params;
use anchor_lang::prelude::*;
use sure_common::token::Seeds;

use crate::{instructions::provide_coverage::OrderParams, utils::SURE_SHIELD};

/// coverage position
/// holds information about the
#[account]
pub struct CoveragePosition {
    /// pda bump
    pub bump: u8, // 1 byte
    pub bump_array: [u8; 1],

    /// mint of position
    pub mint: Pubkey, // 32 bytes

    /// pending coverage
    pub pending_coverage: u64,

    /// provided coverage
    pub provided_coverage: u64,

    pub premium: u64,
}

pub struct CoverageChange {
    pub provided_coverage_reduction: u64,
    pub burn_amount: u64,
}

impl CoveragePosition {
    pub const SPACE: usize = 0;

    pub fn initialize(&mut self, bump: u8, mint: &Pubkey, pending_coverage: u64) {
        self.bump = bump;
        self.bump_array = [bump; 1];
        self.mint = *mint;
        self.pending_coverage = 0;
    }

    pub fn provide_coverage(&mut self, coverage: u64, premium: u64) {
        if coverage > 0 {
            self.provided_coverage = coverage;
            self.premium = premium;
        }
    }

    // decrease the coverage
    pub fn decrease_coverage(&mut self, amount: u64) -> Result<CoverageChange> {
        if amount < self.pending_coverage {
            self.pending_coverage = self.pending_coverage - amount;
            return Ok(CoverageChange {
                provided_coverage_reduction: 0,
                burn_amount: amount,
            });
        }

        let provided_coverage_reduction = amount - self.pending_coverage;

        // update state
        self.pending_coverage = 0;
        self.provided_coverage = self.provided_coverage - provided_coverage_reduction;

        Ok(CoverageChange {
            provided_coverage_reduction: provided_coverage_reduction,
            burn_amount: amount,
        })
    }
}

impl Seeds for CoveragePosition {
    fn seeds(&self) -> Box<[&[u8]]> {
        Box::new([
            &SURE_SHIELD.as_bytes() as &[u8],
            self.mint.as_ref(),
            self.bump_array.as_ref(),
        ])
    }
}
