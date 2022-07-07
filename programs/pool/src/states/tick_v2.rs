use anchor_lang::prelude::*;

use crate::common::errors::SureError;
use crate::common::liquidity::calculate_new_liquidity;
use crate::common::tick_math::*;
use crate::pool::*;
use std::cell::RefMut;

use std::mem::size_of;

pub const NUM_TICKS_IN_TICK_ARRAY: i32 = 64;
pub const NUM_TICKS_IN_TICK_ARRAY_USIZE: usize = 64;
pub const MAX_PROTOCOL_FEE: u64 = 10_000;
pub const MAX_100th_BP: u64 = 1_000_000;

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

    /// Calculate coverage delta
    ///
    /// Coverage is bought from each tick.
    /// Calculate
    ///     - Amount insured
    ///     - Premium
    ///     - Total fee amount
    ///     - Protocol fee
    ///     - Founder fee
    ///
    /// Returns:
    ///     - fee_amount: the amount to be used to pay fees
    ///     - amount_in: the amount to pay into vault 0
    ///     - amount_out: the amount to withdraw from vault 1
    ///
    /// <checkpoint>
    /// TODO: rewrite to use amount in and amount out
    pub fn calculate_coverage_delta(
        &self,
        tick_index: i32,
        coverage_delta: u64,         // the change in coverage > 0
        current_covered_amount: u64, // Current covered amount
        fee_rate: u16,
        current_start_ts: i64,
        expiry_ts: i64,
        increase: bool,
    ) -> Result<(u64, u64, u64)> {
        let available_liquidity = self.get_available_liquidity();

        // calculate premium
        let sqrt_price_x32 = get_sqrt_ratio_at_tick(tick_index)?;

        let remaining_premium =
            calculate_premium(sqrt_price_x32, current_covered_amount, expiry_ts)?;
        // calculates premium
        let (increase_premium, premium_delta) =
            calculate_premium_diff(remaining_premium, sqrt_price_x32, coverage_delta, expiry_ts)?;

        // calculate base fee amount of amount
        let fee_amount = coverage_delta
            .wrapping_mul(fee_rate as u64)
            .wrapping_div(MAX_100th_BP - fee_rate as u64);

        let (amount_in, amount_out) = if increase_premium {
            (premium_delta, 0)
        } else {
            (0, premium_delta)
        };

        Ok((fee_amount, amount_in, amount_out))
    }

    /// Calculate Coverage update
    pub fn calculate_coverage_update( &self,
        increase_coverage: bool,
        current_liquidity: u64,
        coverage_tick_delta: u64,
        fee_growth_global_0_x32: u64,
        fee_growth_global_1_x32: u64,
    ) -> Result<(TickUpdate,u64)> {
        let liquidity_locked = if increase_coverage {
            self.liquidity_locked.checked_add(coverage_tick_delta).ok_or(SureError::AdditionQ3232OverflowError)?
        }else{
            self.liquidity_locked.checked_sub(coverage_tick_delta).ok_or(SureError::SubtractionQ3232Error)?
        };

        let fee_growth_outside_0_x32 =
            fee_growth_global_0_x32.wrapping_sub(self.fee_growth_outside_0_x32);
        let fee_growth_outside_1_x32 =
            fee_growth_global_1_x32.wrapping_sub(self.fee_growth_outside_1_x32);
        let next_liquidity = current_liquidity;
        
        Ok((TickUpdate{
            initialized: true,
            liquidity_net: self.liquidity_net,
            liquidity_gross: self.liquidity_gross,
            liquidity_locked: liquidity_locked,
            fee_growth_outside_0: fee_growth_outside_0_x32,
            fee_growth_outside_1: fee_growth_global_1_x32,
        },next_liquidity))
    }  
    

    pub fn calculate_swap_update(
        &self,
        a_to_b: bool,
        current_liquidity: u64,
        fee_growth_global_0_x32: u64,
        fee_growth_global_1_x32: u64,
    ) -> Result<(TickUpdate,u64)>{
        let liquidity_net: i64 = if a_to_b {
            -self.liquidity_net
        } else {
            self.liquidity_net
        };

        let fee_growth_outside_0_x32 =
            fee_growth_global_0_x32.wrapping_sub(self.fee_growth_outside_0_x32);
        let fee_growth_outside_1_x32 =
            fee_growth_global_1_x32.wrapping_sub(self.fee_growth_outside_1_x32);
        let next_liquidity = add_liquidity_delta(current_liquidity, liquidity_net)?;
        
        Ok((TickUpdate{
            initialized: true,
            liquidity_net: liquidity_net,
            liquidity_gross: self.liquidity_gross,
            liquidity_locked: self.liquidity_locked,
            fee_growth_outside_0: fee_growth_outside_0_x32,
            fee_growth_outside_1: fee_growth_global_1_x32,
        },next_liquidity))
    }

 
    /// Update Tick
    ///
    /// update the tick liquidity is added or subtracted
    pub fn update_tick(
        &mut self,
       tick_update:TickUpdate,
    ) -> Result<()> {
        self.liquidity_net = tick_update.liquidity_net;
        self.liquidity_gross = tick_update.liquidity_gross;
        self.liquidity_locked = tick_update.liquidity_locked;
        self.fee_growth_outside_0_x32 = tick_update.fee_growth_outside_0;
        self.fee_growth_outside_1_x32 = tick_update.fee_growth_outside_1;

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

/// Add Liquidity Delta
///
/// Add the delta to the liquidity depending on the
/// delta sign
pub fn add_liquidity_delta(liquidity: u64, delta: i64) -> Result<u64> {
    if delta == 0 {
        return Ok(liquidity);
    }
    if delta > 0 {
        liquidity
            .checked_add(delta as u64)
            .ok_or(SureError::LiquidityOverflow.into())
    } else {
        liquidity
            .checked_sub(delta.abs() as u64)
            .ok_or(SureError::LiquidityUnderflow.into())
    }
}

/// Calculate the sub fee rate on an amount
pub fn calculate_sub_fee(amount: u64, fee_rate: u16) -> Result<u64> {
    if fee_rate > 0 {
        return amount
            .checked_mul(fee_rate as u64)
            .ok_or(SureError::MultiplictationQ3232Overflow)?
            .checked_div(MAX_PROTOCOL_FEE)
            .ok_or(SureError::DivisionQ3232Error.into());
    }
    Ok(0)
}

/// Calculate fee amounts
///
/// Based on the total fee amount
/// calculate the
///     - protocol fee
///     - founders fee
pub fn calculate_fees(
    fee_amount: u64,
    protocol_fee_rate: u16,
    founders_fee_rate: u16,
    current_liquidity: u64,
    current_protocol_fee: u64,
    current_founders_fee: u64,
    current_fee_growth: u64,
) -> Result<(u64, u64, u64)> {
    let mut next_protocol_fee = current_protocol_fee;
    let mut next_founders_fee = current_founders_fee;
    let mut remaining_fee = fee_amount;
    let mut next_fee_growth = current_fee_growth;
    let step_protocol_fee = calculate_sub_fee(fee_amount, protocol_fee_rate)?;
    let step_founders_fee = calculate_sub_fee(fee_amount, founders_fee_rate)?;
    next_protocol_fee += step_protocol_fee;
    next_founders_fee += step_founders_fee;
    remaining_fee -= next_protocol_fee + next_founders_fee;

    if current_liquidity > 0 {
        next_fee_growth = next_fee_growth.wrapping_add(remaining_fee);
    }
    Ok((next_protocol_fee, next_founders_fee, next_fee_growth))
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


    // Get the maximum tick index in the array
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

    /// Is last tick
    /// Checks if the given tick index is the last/upper tick in the
    /// array
    pub fn is_last_tick(&self, tick_index: i32, tick_spacing: u16) -> Result<bool> {
        let tick_location = get_tick_location(self.start_tick_index, tick_index, tick_spacing)?;
        Ok(tick_location == NUM_TICKS_IN_TICK_ARRAY)
    }

    /// Find the next conditional tick index
    ///
    /// Returns:
    /// - Some if tick_index is located
    /// - None of the tick_index is out of bounds
    pub fn find_next_conditional_tick_index(
        &self,
        tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
        condition: fn(&Tick) -> bool,
    ) -> Result<Option<i32>> {
        if !self.validate_tick_index(tick_index, tick_spacing) {
            return Err(SureError::InvalidTick.into());
        }

        // Find the location of the tick_index in the array [0,64]
        let mut tick_index_location =
            get_tick_location(self.start_tick_index, tick_index, tick_spacing)?;

        // if b to a then search to right
        if !a_to_b {
            tick_index_location += 1;
        }

        // Move through the tick array
        while tick_index_location >= 0 && tick_index_location < NUM_TICKS_IN_TICK_ARRAY {
            let current_tick = self.ticks[tick_index_location as usize];
            // if condition is met return the tick_index
            if condition(&current_tick) {
                return Ok(Some(
                    (tick_index_location * tick_spacing as i32) + self.start_tick_index,
                ));
            }
            // price = b/a, a_to_b = true -> new_price = (b-e1)/(a+e0) < a/b = price.
            // price moves left when swapping a to b
            // a*price = b
            tick_index_location = if a_to_b {
                tick_index_location - 1
            } else {
                tick_index_location + 1
            }
        }

        Ok(None)
    }

    /// Find the next intialized tick index
    pub fn find_next_initialized_tick_index(
        &self,
        tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
    ) -> Result<Option<i32>> {
        self.find_next_conditional_tick_index(tick_index, tick_spacing, a_to_b, |tick: &Tick| {
            tick.is_initialized()
        })
    }

    /// Find the next available tick
    ///
    /// In the tick array based on the
    /// amount of free/available liquidity
    pub fn find_next_available_tick_index(
        &self,
        tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
    ) -> Result<Option<i32>> {
        self.find_next_conditional_tick_index(tick_index, tick_spacing, a_to_b, |tick: &Tick| {
            tick.is_available_liquidity()
        })
    }

    /// Get next tick index
    pub fn get_next_tick_index(
        &self,
        tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
    ) -> Result<Option<i32>> {
        self.find_next_conditional_tick_index(tick_index, tick_spacing, a_to_b, |tick: &Tick| true)
    }

    pub fn get_tick(&self, tick_index: i32, tick_spacing: u16) -> Result<&Tick> {
        if !self.validate_tick_index(tick_index, tick_spacing)
            || !Tick::is_valid_tick(tick_index, tick_spacing)
        {
            return Err(SureError::InvalidTick.into());
        }

        let tick_location = get_tick_location(self.start_tick_index, tick_index, tick_spacing)?;
        Ok(&self.ticks[tick_location as usize])
    }


    pub fn update_tick(&mut self,   tick_index: i32,
        tick_spacing: u16, tick_update: TickUpdate) -> Result<()>{
        if !self.validate_tick_index(tick_index, tick_spacing)
        || !Tick::is_valid_tick(tick_index, tick_spacing)
    {
        return Err(SureError::InvalidTick.into());
    }
    let tick_location = get_tick_location(self.start_tick_index, tick_index, tick_spacing)?;

    self.ticks.get_mut(tick_location as usize).unwrap().update_tick(tick_update)?;
    Ok(())

    }
}

pub struct TickUpdate{
    pub initialized: bool,
    pub liquidity_net: i64,
    pub liquidity_gross: u64,
    pub liquidity_locked: u64,
    pub fee_growth_outside_0: u64,
    pub fee_growth_outside_1: u64,
}

/// Get the index [0,64] in the tick array where the
/// tick is located
pub fn get_tick_location(start_tick_index: i32, tick_index: i32, tick_spacing: u16) -> Result<i32> {
    if tick_index < start_tick_index {
        return Err(SureError::InvalidTick.into());
    }
    let tick_diff = tick_index - start_tick_index;

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
        let mut tick_array_pool = Vec::with_capacity(3);
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

    /// Max tick index
    ///
    /// Find the max tick index in the array
    pub fn max_tick_index(&self, tick_spacing: u16) -> Result<i32> {
        let len_array = self.arrays.len();
        let last_array = self.arrays.get(len_array - 1).unwrap();
        Ok(last_array.get_max_tick_index(tick_spacing))
    }

    /// Max sqrt price
    ///
    /// Calculate the max price in the array
    /// return: sqrt_price: 32.32
    pub fn max_sqrt_price_x32(&self, tick_spacing: u16) -> Result<u64> {
        let tick_index = self.max_tick_index(tick_spacing)?;
        get_sqrt_ratio_at_tick(tick_index)
    }

  

    /// Find the next tick with free/available liquidity that contains
    /// available liquidity
    ///
    /// Returns: (next_tick_array,next_tick_index)
    /// - next_tick_array in [0,2]
    /// - next_tick_index in [-221_818,221_818]
    pub fn find_next_free_tick_index(
        &self,
        current_tick_index: i32,
        tick_spacing: u16,
        a_to_b: bool,
        current_array_index: usize,
    ) -> Result<(usize, i32)> {
        let tick_array_width_in_ticks = NUM_TICKS_IN_TICK_ARRAY * tick_spacing as i32;
        let mut next_tick_index = current_tick_index;
        let mut next_tick_array_index = current_array_index;

        loop {
            let tick_array = match self.arrays.get(next_tick_array_index) {
                Some(array) => array,
                None => return Err(SureError::InvalidTickArrayIndexInTickArrayPool.into()),
            };

            let tick_index =
                tick_array.find_next_available_tick_index(next_tick_index, tick_spacing, a_to_b)?;

            match tick_index {
                Some(tick_index) => return Ok((next_tick_array_index, tick_index)),
                None => {
                    // None when the tick_index is not found in the tick array

                    // if the last tick array
                    if next_tick_array_index + 1 != self.arrays.len() {
                        return Ok((
                            next_tick_array_index,
                            tick_array.get_max_tick_index(tick_spacing),
                        ));
                    }

                    // Check if we are at the boundaries of the tick_index interval
                    if !a_to_b && tick_array.start_tick_index <= MIN_TICK_INDEX {
                        return Ok((next_tick_array_index, MIN_TICK_INDEX));
                    } else if a_to_b
                        && tick_array.get_max_tick_index(tick_spacing) >= MAX_TICK_INDEX
                    {
                        return Ok((next_tick_array_index, MAX_TICK_INDEX));
                    }

                    // If we are at the boundary of the
                    // tick array
                    next_tick_index = if a_to_b {
                        tick_array.start_tick_index + tick_array_width_in_ticks + 1
                    } else {
                        tick_array.start_tick_index - 1
                    };

                    next_tick_array_index += 1;
                }
            }
        }
    }
}
