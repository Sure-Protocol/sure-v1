use anchor_lang::prelude::*;

use crate::helpers::tick::{MAX_TICK_INDEX, MIN_TICK_INDEX};
use crate::pool::*;
use crate::utils::errors::SureError;
use std::mem::size_of;
pub const NUM_TICKS_IN_TICK_ARRAY: usize = 64;

// _____________ v2 _________________
// Instead of recording 256 ticks in each tick array
// we would rather store less ticks for ease of use and
// improved UX for users. The v2 is highly influenced by
// orca tick array and tick design
//
// One of the drawbacks is the cost of initializing a
// TickArray as it will be around 10kb.

/// Tick
#[account(zero_copy)]
#[repr(packed)]
#[derive(Default, Debug, PartialEq)]
pub struct Tick {
    /// Amount of total liquidity in the pool
    pub liquidity: u64, // 8 bytes
    /// Utilization of liquidity
    pub used_liquidity: u64, // 8 bytes
    /// Fee growth
    pub fee_growth_outside_a_x32: u64, // 8 bytes
    pub fee_growth_outside_b_x32: u64, // 8 bytes
}

impl Tick {
    pub const SIZE: usize = 8 + 8 + 8 + 8;

    /// Update tick with a NewTick object
    pub fn update(&mut self, new_tick: &NewTick) {
        self.liquidity = new_tick.liquidity;
        self.used_liquidity = new_tick.used_liquidity;
        self.fee_growth_outside_a_x32 = new_tick.fee_growth_outside_a_x32;
        self.fee_growth_outside_b_x32 = new_tick.fee_growth_outside_b_x32;
    }

    /// Check if the given tick_index is valid
    pub fn is_valid_tick(tick_index: i32, tick_spacing: u16) -> bool {
        if tick_index < MIN_TICK_INDEX || tick_index > MAX_TICK_INDEX {
            return false;
        }

        tick_index % tick_spacing as i32 == 0
    }
}

pub struct NewTick {
    pub liquidity: u64,
    pub used_liquidity: u64,
    pub fee_growth_outside_a_x32: u64,
    pub fee_growth_outside_b_x32: u64,
}

impl NewTick {
    pub fn from(tick: &Tick) -> NewTick {
        NewTick {
            liquidity: tick.liquidity,
            used_liquidity: tick.used_liquidity,
            fee_growth_outside_a_x32: tick.fee_growth_outside_a_x32,
            fee_growth_outside_b_x32: tick.fee_growth_outside_b_x32,
        }
    }
}

/// Tick Array
///
/// An array of Ticks with infor
///
#[account(zero_copy)]
#[repr(packed)]
pub struct TickArray {
    pub start_tick_index: i32,                  // 4 bytes
    pub ticks: [Tick; NUM_TICKS_IN_TICK_ARRAY], // NUM_TICKS_IN_TICK_ARRAY*size_of::<Tick>
    pub pool: Pubkey,                           // 32 bytes
}

impl Default for TickArray {
    #[inline]
    fn default() -> TickArray {
        TickArray {
            pool: Pubkey::default(),
            ticks: [Tick::default(); NUM_TICKS_IN_TICK_ARRAY],
            start_tick_index: 0,
        }
    }
}

impl TickArray {
    pub const SIZE: usize = 4 + NUM_TICKS_IN_TICK_ARRAY * size_of::<Tick>() + 32;

    pub fn initialize(&mut self, pool: &Account<Pool>, start_tick_index: i32) -> Result<()> {
        if !Tick::is_valid_tick(start_tick_index, pool.tick_spacing) {
            return Err(SureError::InvalidTick.into());
        }
        self.start_tick_index = start_tick_index;
        self.pool = pool.key();
        Ok(())
    }
}
