use anchor_lang::prelude::*;

use crate::helpers::tick::{MAX_TICK_INDEX, MIN_TICK_INDEX};
use crate::pool::*;
use crate::utils::errors::SureError;
use crate::utils::liquidity::calculate_new_liquidity;
use std::cell::RefMut;
use std::mem::size_of;

pub const NUM_TICKS_IN_TICK_ARRAY: i32 = 64;
pub const NUM_TICKS_IN_TICK_ARRAY_USIZE: usize = 64;

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
    /// Amount of liquidity added (removed, if neg)
    /// when the tick is crossed going left to right.
    pub liquidity_net: i64, // 8 bytes

    /// The total amount of liquidity
    /// If liquidity_net=0, liquidity_gross indicates
    /// whether the tick is referenced by a position
    pub liquidity_gross: u64, // 8 bytes

    /// Locked liquidity indicates how much of the
    /// liquidity is locked in long term commitments
    pub liquidity_locked: u64, // 8 bytes

    /// Fee growth
    pub fee_growth_outside_0_x32: u64, // 8 bytes
    pub fee_growth_outside_1_x32: u64, // 8 bytes
}

impl Tick {
    pub const SIZE: usize = 8 + 8 + 8 + 8;

    /// Update tick with a NewTick object
    pub fn update(&mut self, new_tick: &NewTick) {
        self.liquidity_net = new_tick.liquidity_net;
        self.liquidity_gross = new_tick.liquidity_gross;
        self.liquidity_locked = new_tick.liquidity_gross;
        self.fee_growth_outside_0_x32 = new_tick.fee_growth_outside_0_x32;
        self.fee_growth_outside_1_x32 = new_tick.fee_growth_outside_1_x32;
    }

    /// Check if the given tick_index is valid
    pub fn is_valid_tick(tick_index: i32, tick_spacing: u16) -> bool {
        if tick_index < MIN_TICK_INDEX || tick_index > MAX_TICK_INDEX {
            return false;
        }

        tick_index % tick_spacing as i32 == 0
    }

    pub fn is_initialized(&self) -> bool {
        if self.liquidity_gross == 0 {
            false
        } else {
            true
        }
    }

    pub fn is_available_liquidity(&self) -> bool {
        let potential_available_liquidity = self.get_available_liquidity();
        if potential_available_liquidity > 0 {
            true
        } else {
            false
        }
    }

    pub fn get_available_liquidity(&self) -> u64 {
        let liquidity_net_abs = self.liquidity_net.abs() as u64;
        self.liquidity_gross + liquidity_net_abs - self.liquidity_locked
    }

    pub fn update_tick(
        &mut self,
        tick_index: i32,
        tick_current_index: i32,
        fee_growth_global_0_x32: u64,
        fee_growth_global_1_x32: u64,
        liquidity_delta: i64,
        is_upper_tick: bool,
    ) -> Result<()> {
        if liquidity_delta == 0 {
            return Ok(());
        }

        let liquidity_gross = calculate_new_liquidity(self.liquidity_gross, liquidity_delta)?;
        if liquidity_gross == 0 {
            return Ok(());
        }

        // tick is uninitialized
        let (fee_growth_outside_0_x32, fee_growth_outside_1_x32) = if self.liquidity_gross == 0 {
            if tick_current_index >= tick_index {
                (fee_growth_global_0_x32, fee_growth_global_1_x32)
            } else {
                (0, 0)
            }
        } else {
            (self.fee_growth_outside_0_x32, self.fee_growth_outside_1_x32)
        };

        // Calculate the net liquidity
        let liquidity_net = if is_upper_tick {
            self.liquidity_net
                .checked_sub(liquidity_delta)
                .ok_or(SureError::LiquidityOverflow)?
        } else {
            self.liquidity_net
                .checked_add(liquidity_delta)
                .ok_or(SureError::LiquidityOverflow)?
        };

        let liquidity_locked = 0;
        self.update(&NewTick {
            liquidity_net,
            liquidity_gross,
            liquidity_locked,
            fee_growth_outside_0_x32,
            fee_growth_outside_1_x32,
        });

        Ok(())
    }

    /// Calculate the next fee growth within the
    /// range (lower,upper)
    ///
    /// returns: cumulative fees per share in the (lower, upper)
    /// for token 0 and token 1
    pub fn calculate_next_fee_growth(
        &self,
        tick_lower_index: i32,
        tick_upper: &Tick,
        tick_upper_index: i32,
        tick_current_index: i32,
        fee_growth_global_0_x32: u64, // token 0
        fee_growth_global_1_x32: u64, // token 1
    ) -> Result<(u64, u64)> {
        if tick_lower_index > tick_upper_index {
            return Err(SureError::InvalidTickIndexProvided.into());
        }
        let (fee_growth_below_a, fee_growth_below_b) = if !self.is_initialized() {
            (fee_growth_global_0_x32, fee_growth_global_1_x32)
        } else if tick_current_index < tick_lower_index {
            (
                fee_growth_global_0_x32.wrapping_sub(self.fee_growth_outside_0_x32),
                fee_growth_global_1_x32.wrapping_sub(self.fee_growth_outside_1_x32),
            )
        } else {
            (self.fee_growth_outside_0_x32, self.fee_growth_outside_1_x32)
        };
        // By convention, when initializing a tick, no fees have been earned above the tick.
        let (fee_growth_above_a, fee_growth_above_b) = if !tick_upper.is_initialized() {
            (0, 0)
        } else if tick_current_index < tick_upper_index {
            (
                tick_upper.fee_growth_outside_0_x32,
                tick_upper.fee_growth_outside_1_x32,
            )
        } else {
            (
                fee_growth_global_0_x32.wrapping_sub(tick_upper.fee_growth_outside_0_x32),
                fee_growth_global_1_x32.wrapping_sub(tick_upper.fee_growth_outside_1_x32),
            )
        };
        Ok((
            fee_growth_global_0_x32
                .wrapping_sub(fee_growth_below_a)
                .wrapping_sub(fee_growth_above_a),
            fee_growth_global_1_x32
                .wrapping_sub(fee_growth_below_b)
                .wrapping_sub(fee_growth_above_b),
        ))
    }
}

pub struct NewTick {
    pub liquidity_net: i64,
    pub liquidity_gross: u64,
    pub liquidity_locked: u64,
    pub fee_growth_outside_0_x32: u64,
    pub fee_growth_outside_1_x32: u64,
}

impl NewTick {
    pub fn from(tick: &Tick) -> NewTick {
        NewTick {
            liquidity_net: tick.liquidity_net,
            liquidity_gross: tick.liquidity_gross,
            liquidity_locked: tick.liquidity_locked,
            fee_growth_outside_0_x32: tick.fee_growth_outside_0_x32,
            fee_growth_outside_1_x32: tick.fee_growth_outside_1_x32,
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
    pub start_tick_index: i32,                        // 4 bytes
    pub ticks: [Tick; NUM_TICKS_IN_TICK_ARRAY_USIZE], // NUM_TICKS_IN_TICK_ARRAY*size_of::<Tick>
    pub pool: Pubkey,                                 // 32 bytes
}

impl Default for TickArray {
    #[inline]
    fn default() -> TickArray {
        TickArray {
            pool: Pubkey::default(),
            ticks: [Tick::default(); NUM_TICKS_IN_TICK_ARRAY_USIZE],
            start_tick_index: 0,
        }
    }
}

impl TickArray {
    pub const SIZE: usize = 4 + NUM_TICKS_IN_TICK_ARRAY_USIZE * size_of::<Tick>() + 32;

    pub fn initialize(&mut self, pool: &Account<Pool>, start_tick_index: i32) -> Result<()> {
        if !Tick::is_valid_tick(start_tick_index, pool.tick_spacing) {
            return Err(SureError::InvalidTick.into());
        }
        self.start_tick_index = start_tick_index;
        self.pool = pool.key();
        Ok(())
    }

    pub fn get_max_tick_index(&self, tick_spacing: u16) -> i32 {
        self.start_tick_index + tick_spacing as i32 * NUM_TICKS_IN_TICK_ARRAY
    }

    /// Check if tick is in the tick array
    pub fn validate_tick_index(&self, tick_index: i32, tick_spacing: u16) -> bool {
        let lower_tick_index = self.start_tick_index;
        let upper_tick_index =
            self.start_tick_index + NUM_TICKS_IN_TICK_ARRAY * tick_spacing as i32;
        tick_index >= lower_tick_index && tick_index <= upper_tick_index
    }

    /// Get the index [0,64] in the tick array where the
    /// tick is located
    pub fn get_tick_location(&self, tick_index: i32, tick_spacing: u16) -> Result<i32> {
        let tick_diff = tick_index - self.start_tick_index;

        if tick_diff % tick_spacing as i32 != 0 {
            return Err(SureError::InvalidTick.into());
        }

        if tick_spacing == 0 {
            return Err(SureError::InvalidTickSpacing.into());
        }
        let tick_location = tick_diff / tick_spacing as i32;
        if tick_location < 0 || tick_location >= NUM_TICKS_IN_TICK_ARRAY {
            return Err(SureError::InvalidTick.into());
        }
        Ok(tick_location)
    }

    pub fn find_next_conditional_tick(
        &self,
        tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
        condition: fn(&Tick) -> bool,
    ) -> Result<Option<i32>> {
        if !self.validate_tick_index(tick_index, tick_spacing) {
            return Err(SureError::InvalidTick.into());
        }

        // tick_index location in tick array
        let mut tick_array_loc = self.get_tick_location(tick_index, tick_spacing)?;

        if !a_to_b {
            tick_array_loc += 1;
        }

        while tick_array_loc >= 0 && tick_array_loc < NUM_TICKS_IN_TICK_ARRAY {
            let current_tick = self.ticks[tick_array_loc as usize];
            if condition(&current_tick) {
                return Ok(Some(
                    (tick_array_loc * tick_spacing as i32) + self.start_tick_index,
                ));
            }
            tick_array_loc = if a_to_b {
                tick_array_loc - 1
            } else {
                tick_array_loc + 1
            }
        }

        Ok(None)
    }

    /// Find the next intialized tick
    /// in the tick array.
    pub fn find_next_initialized_tick(
        &self,
        tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
    ) -> Result<Option<i32>> {
        self.find_next_conditional_tick(tick_index, tick_spacing, a_to_b, |tick: &Tick| {
            tick.is_initialized()
        })
    }

    /// Find the next initialized tick
    /// in the tick array based on the
    /// amount of free/available liquidity
    pub fn find_next_available_tick(
        &self,
        tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
    ) -> Result<Option<i32>> {
        self.find_next_conditional_tick(tick_index, tick_spacing, a_to_b, |tick: &Tick| {
            tick.is_available_liquidity()
        })
    }

    pub fn get_tick(&mut self, tick_index: i32, tick_spacing: u16) -> Result<&Tick> {
        if !self.validate_tick_index(tick_index, tick_spacing)
            || !Tick::is_valid_tick(tick_index, tick_spacing)
        {
            return Err(SureError::InvalidTick.into());
        }

        let tick_location = self.get_tick_location(tick_index, tick_spacing)?;
        Ok(&self.ticks[tick_location as usize])
    }
}

/// Tick Array Pool
///
/// Used for combining tick arrays and perform
/// operations on top of each
pub struct TickArrayPool<'info> {
    pub arrays: Vec<RefMut<'info, TickArray>>,
}

impl<'info> TickArrayPool<'info> {
    pub fn new(
        ta0: RefMut<'info, TickArray>,
        ta1: Option<RefMut<'info, TickArray>>,
        ta2: Option<RefMut<'info, TickArray>>,
    ) -> Self {
        let tick_array_pool = Vec::with_capacity(3);
        tick_array_pool.push(ta0);
        if ta1.is_some() {
            tick_array_pool.push(ta1.unwrap());
        }
        if ta2.is_some() {
            tick_array_pool.push(ta2.unwrap())
        }

        Self {
            arrays: tick_array_pool,
        }
    }

    /// Find the next tick with free/available liquidity that contains
    /// available liquidity
    pub fn find_next_free_tick_index(
        &self,
        current_tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
        current_array_index: usize,
    ) -> Result<(usize, i32)> {
        let tick_array_width_in_ticks = NUM_TICKS_IN_TICK_ARRAY * tick_spacing as i32;
        let mut tick_index_point = current_tick_index;
        let mut tick_array_point = current_array_index;

        loop {
            let tick_array = match self.arrays.get(tick_array_point) {
                Some(array) => array,
                None => return Err(SureError::InvalidTickArrayIndexInTickArrayPool.into()),
            };

            let tick_index =
                tick_array.find_next_available_tick(tick_index_point, tick_spacing, false)?;

            match tick_index {
                Some(tick_index) => return Ok((tick_array_point, tick_index)),
                None => {
                    // If last tick array
                    if tick_array_point + 1 != self.arrays.len() {
                        return Ok((
                            tick_array_point,
                            tick_array.get_max_tick_index(tick_spacing),
                        ));
                    }

                    tick_index_point = if a_to_b {
                        tick_array.start_tick_index - 1
                    } else {
                        tick_array.start_tick_index + tick_array_width_in_ticks - 1
                    };

                    tick_array_point += 1;
                }
            }
        }
    }
}
