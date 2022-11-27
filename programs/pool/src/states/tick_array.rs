use anchor_lang::prelude::*;

use super::*;
use crate::pool::*;
use crate::utils::product::ProductType;
use crate::utils::tick_math::{get_sqrt_ratio_at_tick, MAX_TICK_INDEX, MIN_TICK_INDEX};
use crate::utils::*;

use std::cell::RefMut;
use std::mem::size_of;

pub const NUM_TICKS_IN_TICK_ARRAY: i32 = 64;
pub const NUM_TICKS_IN_TICK_ARRAY_USIZE: usize = 64;
pub const MAX_PROTOCOL_FEE: usize = 10_000;
pub const MAX_100th_BP: usize = 1_000_000;

// _____________ v2 _________________
// Instead of recording 256 ticks in each tick array
// we would rather store less ticks for ease of use and
// improved UX for users. The v2 is highly influenced by
// orca tick array and tick design
//
// One of the drawbacks is the cost of initializing a
// TickArray as it will be around 10kb.

/// Tick
#[zero_copy]
#[repr(packed)]
#[derive(Default, Debug, PartialEq)]
pub struct Tick {
    pub bump: u8,

    /// Amount of liquidity added (removed, if neg)
    /// when the tick is crossed going left to right.
    pub liquidity_gross: u128, // 16 bytes

    /// Locked liquidity indicates how much of the
    /// liquidity is locked in long term commitments
    pub liquidity_used: u128, // 16 bytes
}

impl Tick {
    pub const SIZE: usize = 16 + 16 + 16 + 16 + 16;

    /// Update tick with a NewTick object
    pub fn update(&mut self, new_tick: &NewTick) {
        self.liquidity_gross = new_tick.liquidity_gross;
        self.liquidity_used = new_tick.liquidity_used;
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

    pub fn get_available_liquidity(&self) -> u128 {
        self.liquidity_gross - self.liquidity_used
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
        target_tick_index: i32,
        coverage_delta: u128,         // the change in coverage > 0
        current_covered_amount: u128, // Current covered amount
        fee_rate: u16,
        current_start_ts: i64,
        expiry_ts: i64,
        increase: bool,
    ) -> Result<(u128, u128, u128)> {
        // available liquidity at tick
        let available_liquidity = self.get_available_liquidity();

        // calculate premium
        let sqrt_price_x64 = get_sqrt_ratio_at_tick(tick_index);
        let sqrt_price_target = get_sqrt_ratio_at_tick(target_tick_index);

        // TODO: calculate
        let remaining_premium = 0;
        // calculates premium
        let (increase_premium, premium_delta) = (true, 0);

        // calculate base fee amount of amount
        let fee_amount = coverage_delta
            .wrapping_mul(fee_rate as u128)
            .wrapping_div(MAX_100th_BP as u128 - fee_rate as u128);

        let (amount_in, amount_out) = if increase_premium {
            (premium_delta, 0)
        } else {
            (0, premium_delta)
        };

        Ok((fee_amount, amount_in, amount_out))
    }

    /// Calculate Coverage update
    pub fn calculate_coverage_update(
        &self,
        increase_coverage: bool,
        current_liquidity: u128,
        coverage_tick_delta: u128,
        fee_growth_global_0_x64: u128,
        fee_growth_global_1_x64: u128,
    ) -> Result<(TickUpdate, u128)> {
        let liquidity_used = if increase_coverage {
            self.liquidity_used
                .checked_add(coverage_tick_delta)
                .ok_or(SureError::AdditionQ3232OverflowError)?
        } else {
            self.liquidity_used
                .checked_sub(coverage_tick_delta)
                .ok_or(SureError::SubtractionQ3232Error)?
        };

        let next_liquidity = current_liquidity;

        Ok((
            TickUpdate {
                initialized: true,
                liquidity_gross: self.liquidity_gross,
                liquidity_used: liquidity_used,
                fee_growth_outside_0: 0,
                fee_growth_outside_1: 0,
            },
            next_liquidity,
        ))
    }

    /// Update Tick
    ///
    /// update the tick liquidity is added or subtracted
    pub fn update_tick(&mut self, tick_update: &TickUpdate) -> Result<()> {
        self.liquidity_gross = tick_update.liquidity_gross;
        self.liquidity_used = tick_update.liquidity_used;

        Ok(())
    }
}

/// Add Liquidity Delta
///
/// Add the delta to the liquidity depending on the
/// delta sign
pub fn add_liquidity_delta(liquidity: u128, delta: i128) -> Result<u128> {
    if delta == 0 {
        return Ok(liquidity);
    }
    if delta > 0 {
        liquidity
            .checked_add(delta as u128)
            .ok_or(SureError::LiquidityOverflow.into())
    } else {
        liquidity
            .checked_sub(delta.abs() as u128)
            .ok_or(SureError::LiquidityUnderflow.into())
    }
}

/// Calculate the sub fee rate on an amount
pub fn calculate_sub_fee(amount: u128, fee_rate: u16) -> Result<u128> {
    if fee_rate > 0 {
        return amount
            .checked_mul(fee_rate as u128)
            .ok_or(SureError::MultiplictationQ3232Overflow)?
            .checked_div(MAX_PROTOCOL_FEE as u128)
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
    fee_amount: u128,
    protocol_fee_rate: u16,
    founders_fee_rate: u16,
    current_liquidity: u128,
    current_protocol_fee: u128,
    current_founders_fee: u128,
    current_fee_growth: u128,
) -> Result<(u128, u128, u128)> {
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
    pub liquidity_gross: u128,
    pub liquidity_used: u128,
}

impl NewTick {
    pub fn from(tick: &Tick) -> NewTick {
        NewTick {
            liquidity_gross: tick.liquidity_gross,
            liquidity_used: tick.liquidity_used,
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
    pub const SIZE: usize = 4 + 64 * 40 + 32;

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
        self.start_tick_index + tick_spacing as i32 * (NUM_TICKS_IN_TICK_ARRAY - 1)
    }

    pub fn get_min_tick_index(&self, tick_spacing: u16) -> i32 {
        self.start_tick_index
    }

    /// Check if tick is in the tick array
    pub fn validate_tick_index(&self, tick_index: i32, tick_spacing: u16) -> bool {
        let lower_tick_index = self.start_tick_index;
        let upper_tick_index =
            self.start_tick_index + NUM_TICKS_IN_TICK_ARRAY * tick_spacing as i32;
        msg!(&format!(
            "validate tick index > tick index: {} lower_tick_index: {}, upper_tick_index {} ",
            tick_index, lower_tick_index, upper_tick_index
        ));
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
            return Err(SureError::TickOutOfRange.into());
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

    pub fn update_tick(
        &mut self,
        tick_index: i32,
        tick_spacing: u16,
        tick_update: &TickUpdate,
    ) -> Result<()> {
        if !self.validate_tick_index(tick_index, tick_spacing)
            || !Tick::is_valid_tick(tick_index, tick_spacing)
        {
            return Err(SureError::InvalidTick.into());
        }
        let tick_location = get_tick_location(self.start_tick_index, tick_index, tick_spacing)?;

        self.ticks
            .get_mut(tick_location as usize)
            .unwrap()
            .update_tick(tick_update)?;
        Ok(())
    }
}

#[derive(Default, Debug, PartialEq)]
pub struct TickUpdate {
    pub initialized: bool,
    pub liquidity_gross: u128,
    pub liquidity_used: u128,
    pub fee_growth_outside_0: u128,
    pub fee_growth_outside_1: u128,
}

impl TickUpdate {
    pub fn from(tick: &Tick) -> Self {
        TickUpdate {
            initialized: tick.is_initialized(),
            liquidity_gross: tick.liquidity_gross,
            liquidity_used: tick.liquidity_used,
            fee_growth_outside_0: 0,
            fee_growth_outside_1: 0,
        }
    }
}

/// Get the index [0,64] in the tick array where the
/// tick is located
pub fn get_tick_location(start_tick_index: i32, tick_index: i32, tick_spacing: u16) -> Result<i32> {
    if tick_index < start_tick_index {
        return Err(SureError::TickLtTickArray.into());
    }
    let tick_diff = tick_index - start_tick_index;

    if tick_spacing == 0 {
        return Err(SureError::InvalidTickSpacing.into());
    }

    if tick_diff % tick_spacing as i32 != 0 {
        return Err(SureError::TickOutsideSpacing.into());
    }

    let tick_location = tick_diff / tick_spacing as i32;
    println!("tick location: {}", tick_location);
    if tick_location < 0 || tick_location >= NUM_TICKS_IN_TICK_ARRAY {
        return Err(SureError::TickOutOfRange.into());
    }
    Ok(tick_location)
}

/// Tick Array Pool
///
/// Used for combining tick arrays and perform
/// operations on top of each
pub struct TickArrayPool<'info> {
    arrays: Vec<RefMut<'info, TickArray>>,
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

    /// Min tick index
    ///
    /// Find the minimum tick index in the array
    pub fn min_tick_index(&self, tick_spacing: u16) -> Result<i32> {
        let first_array = self.arrays.get(0).unwrap();
        Ok(first_array.get_min_tick_index(tick_spacing))
    }

    /// Max sqrt price
    ///
    /// Calculate the max price in the array
    /// return: sqrt_price: 32.32
    pub fn max_sqrt_price_x32(&self, tick_spacing: u16) -> Result<u128> {
        let tick_index = self.max_tick_index(tick_spacing)?;
        Ok(get_sqrt_ratio_at_tick(tick_index))
    }

    /// Get tick
    ///
    /// get the tick in the sequence
    pub fn get_tick(
        &mut self,
        array_index: usize,
        tick_index: i32,
        tick_spacing: u16,
    ) -> Result<&Tick> {
        let array = self.arrays.get_mut(array_index).unwrap();
        array.get_tick(tick_index, tick_spacing)
    }

    /// Is Last tick in array
    ///
    pub fn is_last_tick_index_in_array(
        &mut self,
        array_index: usize,
        tick_index: i32,
        tick_spacing: u16,
    ) -> Result<bool> {
        let array = self.arrays.get_mut(array_index).unwrap();
        array.is_last_tick(tick_index, tick_spacing)
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
        println!("| find_next_free_tick_index");
        let tick_array_width_in_ticks = NUM_TICKS_IN_TICK_ARRAY * tick_spacing as i32;
        let mut next_tick_index = current_tick_index;
        let mut next_tick_array_index = current_array_index;

        loop {
            println!(
                "> loop next_tick_array_index: {} , next_tick_index {}",
                next_tick_array_index, next_tick_index
            );
            let tick_array = match self.arrays.get(next_tick_array_index) {
                Some(array) => array,
                None => return Err(SureError::InvalidTickArrayIndexInTickArrayPool.into()),
            };

            let tick_index =
                tick_array.find_next_available_tick_index(next_tick_index, tick_spacing, a_to_b)?;

            match tick_index {
                Some(tick_index) => return Ok((next_tick_array_index, tick_index)),
                None => {
                    println!(" > no liquidity in array");
                    // None when the tick_index is not found in the tick array

                    // if the last tick array
                    if next_tick_array_index + 1 == self.arrays.len() {
                        println!("> no liquidity in pool");
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

                    next_tick_index = if !a_to_b {
                        tick_array.start_tick_index + tick_array_width_in_ticks
                    } else {
                        tick_array.start_tick_index - 1
                    };

                    next_tick_array_index += 1;
                    println!(
                        "> next_tick_array_index {} , next_tick_index: {}",
                        next_tick_array_index, next_tick_index
                    );
                }
            }
        }
    }

    pub fn update_tick(
        &mut self,
        array_index: usize,
        tick_index: i32,
        tick_spacing: u16,
        tick_update: &TickUpdate,
    ) -> Result<()> {
        let tick_array = self.arrays.get_mut(array_index).unwrap();
        tick_array.update_tick(tick_index, tick_spacing, tick_update)
    }

    /// Update Ticks with new liquidity
    /// TODO: implement
    pub fn update_liquidity(liquidity_target: u128, current_tick: i32) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
pub mod tick_testing {
    use super::*;

    #[derive(Default)]
    pub struct TickProto {
        pub liquidity_gross: u128,          // 16 bytes
        pub liquidity_used: u128,           // 16 bytes
        pub fee_growth_outside_0_x64: u128, // 16 bytes
        pub fee_growth_outside_1_x64: u128, // 16 bytes
    }

    impl TickProto {
        pub fn new() -> Self {
            Self {
                ..Default::default()
            }
        }

        pub fn liquidity_gross(mut self, liquidity_gross: u128) -> Self {
            self.liquidity_gross = liquidity_gross;
            self
        }

        pub fn liquidity_used(mut self, liquidity_used: u128) -> Self {
            self.liquidity_used = liquidity_used;
            self
        }

        pub fn build(self) -> Tick {
            Tick {
                bump: 0,
                liquidity_gross: self.liquidity_gross,
                liquidity_used: self.liquidity_used,
            }
        }
    }

    #[test]
    pub fn test_is_valid_tick() {
        let configs = [(40, 20, true), (30, 20, false)];
        for (tick, tick_spacing, expected) in configs {
            assert_eq!(Tick::is_valid_tick(tick, tick_spacing), expected);
        }
    }

    #[test]
    pub fn test_is_initilized() {
        let tick = TickProto::new().liquidity_gross(10).build();
        assert_eq!(tick.is_initialized(), true);
    }

    #[test]
    pub fn test_calculate_next_liquidity_update() {
        #[derive(Default)]
        pub struct Test<'a> {
            test_name: &'a str,
            tick: Tick,
            tick_index: i32,
            current_tick: i32,
            fee_growth_global_0_x64: u128,
            fee_growth_global_1_x64: u128,
            liquidity_delta: i128,
            product_type: ProductType,
            is_upper_tick: bool,
            expected_tick_update: TickUpdate,
        }

        let test_data = [
            Test {
                test_name: "1. empty tick. Current tick below lower tick",
                tick: TickProto::new().liquidity_gross(0).build(),
                tick_index: 100,
                current_tick: 20,
                fee_growth_global_0_x64: 0,
                fee_growth_global_1_x64: 0,
                liquidity_delta: 24000,
                product_type: ProductType::Coverage,
                is_upper_tick: false,
                expected_tick_update: TickUpdate {
                    initialized: true,
                    liquidity_gross: 24000,
                    liquidity_used: 0,
                    fee_growth_outside_0: 0,
                    fee_growth_outside_1: 0,
                },
            },
            Test {
                test_name: "2. empty tick. Current tick above lower tick",
                tick: TickProto::new().liquidity_gross(0).build(),
                tick_index: 100,
                current_tick: 120,
                fee_growth_global_0_x64: 100,
                fee_growth_global_1_x64: 100,
                liquidity_delta: 24000,
                product_type: ProductType::Coverage,
                is_upper_tick: false,
                expected_tick_update: TickUpdate {
                    initialized: true,
                    liquidity_gross: 24000,
                    liquidity_used: 0,
                    fee_growth_outside_0: 100,
                    fee_growth_outside_1: 100,
                },
            },
            Test {
                test_name: "3. lower tick, initialized tick. Current tick above lower tick",
                tick: TickProto::new()
                    .liquidity_gross(100)
                    .build(),
                tick_index: 100,
                current_tick: 20,
                fee_growth_global_0_x64: 100,
                fee_growth_global_1_x64: 100,
                liquidity_delta: 24000,
                product_type: ProductType::Coverage,
                is_upper_tick: false,
                expected_tick_update: TickUpdate {
                    initialized: true,
                    liquidity_gross: 24100,
                    liquidity_used: 0,
                    fee_growth_outside_0: 0,
                    fee_growth_outside_1: 0,
                },
            },
            Test {
                test_name: "4. upper tick, initialized tick. Upper tick above current tick. Should subtract from net and add to gross",
                tick: TickProto::new()
                    .liquidity_gross(100000)
                    .build(),
                tick_index: 100,
                current_tick: 20,
                fee_growth_global_0_x64: 100,
                fee_growth_global_1_x64: 100,
                liquidity_delta: 24000,
                product_type: ProductType::Coverage,
                is_upper_tick: true,
                expected_tick_update: TickUpdate {
                    initialized: true,
                    liquidity_gross: 100000 + 24000,
                    liquidity_used: 0,
                    fee_growth_outside_0: 0,
                    fee_growth_outside_1: 0,
                },
            },
            Test {
                test_name: "5. upper initialized tick. Upper tick above current tick. Reduce position, should subtract from gross and add to net",
                tick: TickProto::new()
                    .liquidity_gross(100000)
                    .build(),
                tick_index: 100,
                current_tick: 20,
                fee_growth_global_0_x64: 100,
                fee_growth_global_1_x64: 100,
                liquidity_delta: -100000,
                product_type: ProductType::Coverage,
                is_upper_tick: true,
                expected_tick_update: TickUpdate {
                    initialized: false,
                    liquidity_gross: 0,
                    liquidity_used: 0,
                    fee_growth_outside_0: 0,
                    fee_growth_outside_1: 0,
                },
            },
        ];

        for test in test_data {}
    }

    #[test]
    pub fn test_calculate_coverage_delta() {
        #[derive(Default)]
        pub struct ExpectedOutput {
            fee_amount: u128,
            amount_in: u128,
            amount_out: u128,
        }
        #[derive(Default)]
        pub struct Test<'a> {
            test_name: &'a str,
            tick: Tick,
            tick_index: i32,
            coverage_delta: u128,
            current_covered_amount: u128,
            fee_rate: u16,
            current_start_ts: i64,
            expiry_ts: i64,
            increase: bool,
            expected_output: ExpectedOutput,
        }
    }
}

#[cfg(test)]
pub mod tick_array_testing {
    use super::*;
    use std::borrow::BorrowMut;
    use std::cell::RefCell;

    // tick array proto
    pub struct TickArrayProto {
        pub start_tick_index: i32,                        // 4 bytes
        pub ticks: [Tick; NUM_TICKS_IN_TICK_ARRAY_USIZE], // NUM_TICKS_IN_TICK_ARRAY*size_of::<Tick>
    }

    impl TickArrayProto {
        pub fn new() -> Self {
            Self {
                start_tick_index: 0,
                ticks: [Tick::default(); NUM_TICKS_IN_TICK_ARRAY_USIZE],
            }
        }

        pub fn set_start_tick_index(mut self, start_tick_index: i32) -> Self {
            self.start_tick_index = start_tick_index;
            self
        }

        pub fn build<'info>(&self) -> TickArray {
            let tick_array = TickArray {
                start_tick_index: self.start_tick_index,
                ticks: self.ticks,
                ..Default::default()
            };
            tick_array
        }
    }
}

#[cfg(test)]
pub mod tick_array_pool_testing {
    use super::{tick_array_testing::TickArrayProto, *};
    use std::cell::RefCell;
    pub struct TickArrayPoolProto<'info> {
        arrays: Vec<RefMut<'info, TickArray>>,
    }
    impl<'info> TickArrayPoolProto<'info> {
        pub fn new(start_tick_index: i32, tick_spacing: u16) -> Vec<RefCell<TickArray>> {
            let ta0 = TickArrayProto::new()
                .set_start_tick_index(start_tick_index)
                .build();
            let ta1 = TickArrayProto::new()
                .set_start_tick_index(ta0.get_max_tick_index(tick_spacing))
                .build();
            let ta2 = TickArrayProto::new()
                .set_start_tick_index(ta1.get_max_tick_index(tick_spacing))
                .build();

            let mut tick_array_pool = Vec::with_capacity(3);
            tick_array_pool.push(RefCell::new(ta0));
            tick_array_pool.push(RefCell::new(ta1));
            tick_array_pool.push(RefCell::new(ta2));
            tick_array_pool
        }

        pub fn build(self) -> TickArrayPool<'info> {
            TickArrayPool {
                arrays: self.arrays,
            }
        }
    }
}
