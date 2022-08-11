use std::{borrow::Borrow, cell::RefMut};

use super::{fee::FeePackage, tick_v2::TickUpdate};
use crate::common::{
    errors::SureError, liquidity::calculate_new_liquidity, seeds::*, tick_math::*,
};
use anchor_lang::{prelude::*, Result};

use super::{
    liquidity::LiquidityPosition,
    tick_v2::{calculate_fees, TickArrayPool},
    CoveragePosition,
};

/// Product Pool
/// The product pool holds information regarding the specific product
#[account]
pub struct ProductPool {
    // Product
    pub product_id: u8, // 1

    /// Pools in the ProductPool
    pub pools: Vec<Pubkey>, // 4 + 32*64, 64 pools for each product
}

impl ProductPool {
    const MAX_POOLS: usize = 64;
    pub const SPACE: usize = 1 + 4 + 32 * 64;

    pub fn initialize(&mut self, product_id: u8) -> Result<()> {
        self.product_id = product_id;
        self.pools = Vec::new();
        Ok(())
    }

    pub fn add_pool(&mut self, pool: Pubkey) -> Result<()> {
        if self.pools.len() >= ProductPool::MAX_POOLS {
            return Err(SureError::PoolsInProductPoolExceeded.into());
        }
        self.pools.push(pool);
        Ok(())
    }

    pub fn remove_pool(&mut self, pool_pk: Pubkey) -> Result<()> {
        if self.pools.len() == 0 {
            return Err(SureError::ProductPoolIsEmpty.into());
        }
        if let Some(idx) = self.pools.iter().position(|x| *x == pool_pk) {
            self.pools.remove(idx);
        }
        Ok(())
    }
}

/// Sure Pool
///
/// Sure pool is used to provide liquidity for
/// customers of the given product
#[account]
#[derive(Default)]
pub struct Pool {
    /// bump
    pub bump_array: [u8; 1], // 1 byte

    /// ProductId
    pub product_id: u8, // 1
    pub product_id_seed: [u8; 1],
    /// Name of pool
    pub name: String, // 4 + 200 bytes

    /// Founder of pool
    pub founder: Pubkey, // 32 bytes

    /// space between each tick in basis point
    pub tick_spacing: u16, // 2 bytes
    pub tick_spacing_seed: [u8; 2], // 2*1 = 2 bytes

    /// fee rate for each transaction in pool
    // hundreth of a basis point i.e. fee_rate = 1 = 0.01 bp = 0.00001%
    pub fee_rate: u16, //2 bytes

    /// Protocol fee of transaction
    /// (1/x)% of fee_rate
    pub protocol_fee_rate: u16, // 2 bytes

    /// Founder fee of transaction
    /// (1/x)% of fee_rate
    pub founders_fee_rate: u16, // 2 bytes

    /// Liquidity in Pool
    pub liquidity: u128, // 8 bytes

    /// The current market price as
    /// use Q64.64 - 64 bytes at each
    /// side of decimal point
    pub sqrt_price_x64: u128, // 8 bytes

    /// Current tick index corrensponding to sqrt price
    pub current_tick_index: i32, // 4 bytes

    /// Tokens in vault 0 that is owed to the sure
    pub protocol_fees_owed_0: u128, // 8 bytes
    pub founders_fees_owed_0: u128, // 8 bytes
    /// total fees in vault a collected per unit of liquidity
    pub fee_growth_0_x64: u128, // 8 bytes

    /// Tokens in vault 0 that is owed to the sure
    pub protocol_fees_owed_1: u128, // 8 bytes
    pub founders_fees_owed_1: u128, // 8 bytes
    /// total fees collected in vault b per unit of liquidity
    pub fee_growth_1_x64: u128, // 8 bytes

    /// Token mint A of pool
    pub token_mint_0: Pubkey, // 32 bytes
    pub token_vault_0: Pubkey, //32 bytes

    /// Token mint B of pool
    pub token_mint_1: Pubkey, // 32 bytes
    pub token_vault_1: Pubkey, //32 bytes

    /// Used liquidity
    pub used_liquidity: u128, // 8 bytes
}

impl Pool {
    pub const SPACE: usize = 1
        + 1
        + (4 + 200)
        + 32
        + 2
        + 2
        + 2
        + 2
        + 2
        + 8
        + 8
        + 4
        + 8
        + 8
        + 8
        + 8
        + 8
        + 8
        + 32
        + 32
        + 32
        + 32
        + 8;

    pub fn seeds(&self) -> [&[u8]; 6] {
        [
            &SURE_DOMAIN.as_bytes() as &[u8],
            self.product_id_seed.as_ref(),
            self.token_mint_0.as_ref(),
            self.token_mint_1.as_ref(),
            self.tick_spacing_seed.as_ref(),
            self.bump_array.as_ref(),
        ]
    }

    pub fn initialize(
        &mut self,
        bump: u8,
        product_id: u8,
        name: String,
        founder: Pubkey,
        tick_spacing: u16,
        fee_package: &Account<FeePackage>,
        sqrt_price_x64: u128,
        token_mint_0: Pubkey,
        token_mint_1: Pubkey,
        pool_vault_0: Pubkey,
        pool_vault_1: Pubkey,
    ) -> Result<()> {
        self.bump_array = bump.to_le_bytes();

        self.product_id = product_id;
        self.product_id_seed = product_id.to_le_bytes();
        self.name = name;
        self.founder = founder;
        self.tick_spacing = tick_spacing;
        self.tick_spacing_seed = tick_spacing.to_le_bytes();

        fee_package.validate_fee_rates()?;

        self.fee_rate = fee_package.fee_rate;
        self.protocol_fee_rate = fee_package.protocol_fee_rate;
        self.founders_fee_rate = fee_package.founders_fee_rate;

        self.liquidity = 0;
        self.current_tick_index = get_tick_at_sqrt_ratio(sqrt_price_x64)?;
        self.sqrt_price_x64 = sqrt_price_x64;
        if token_mint_0.ge(&token_mint_1) {
            return Err(SureError::WrongTokenMintOrder.into());
        }
        self.token_mint_0 = token_mint_0;
        self.token_mint_1 = token_mint_1;
        self.token_vault_0 = pool_vault_0;
        self.token_vault_1 = pool_vault_1;
        self.used_liquidity = 0;

        Ok(())
    }

    /// Update fees collected by the pool
    /// Should happen on each tx
    pub fn update_post_transaction(
        &mut self,
        liquidity: u128,
        tick: i32,
        fee_growth: u128,
        protocol_fee: u128,
        is_fee_in_a: bool,
    ) -> Result<()> {
        self.liquidity = liquidity;
        self.sqrt_price_x64 = get_sqrt_ratio_at_tick(tick);
        if is_fee_in_a {
            self.fee_growth_0_x64 = fee_growth;
            self.protocol_fees_owed_0 += protocol_fee;
        } else {
            self.fee_growth_1_x64 = fee_growth;
            self.protocol_fees_owed_1 += protocol_fee;
        }

        Ok(())
    }

    pub fn update_liquidity(&mut self, liquidity: u128) {
        self.liquidity = liquidity;
    }

    /// Update Pool
    pub fn update(&mut self, liquidity: u128, sqrt_price_x64: u128, tick_index: i32) {
        self.update_liquidity(liquidity);
        self.sqrt_price_x64 = sqrt_price_x64;
        self.current_tick_index = tick_index;
    }

    /// Get the current tick index from the
    /// current sqrt price
    pub fn get_current_tick_index(&self) -> Result<i32> {
        get_tick_at_sqrt_ratio(self.sqrt_price_x64)
    }

    /// Update the fee package
    pub fn update_fee_package(&mut self, fee_package: FeePackage) -> Result<()> {
        fee_package.validate_fee_rates()?;
        self.fee_rate = fee_package.fee_rate;
        self.protocol_fee_rate = fee_package.protocol_fee_rate;
        self.founders_fee_rate = fee_package.founders_fee_rate;
        Ok(())
    }

    /// Is position in range
    /// Check to see if the position given by lower and upper
    /// tick is in range
    pub fn is_position_in_range(&self, position: &LiquidityPosition) -> Result<bool> {
        if self.current_tick_index >= position.tick_index_lower
            && self.current_tick_index < position.tick_index_upper
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get next liquidity
    /// If the current tick in the pool is in the position
    /// then calculate the next liquidity
    pub fn get_next_liquidity(&self, position: &LiquidityPosition, delta: i128) -> Result<u128> {
        if self.is_position_in_range(position)? {
            calculate_new_liquidity(self.liquidity, delta)
        } else {
            Ok(self.liquidity)
        }
    }

    /// Update the pool after a coverage change
    pub fn update_after_coverage_change(
        &mut self,
        buy_coverage_result: &BuyCoverageResult,
    ) -> Result<()> {
        self.current_tick_index = buy_coverage_result.next_tick_index;
        self.sqrt_price_x64 = buy_coverage_result.next_sqrt_price;
        self.liquidity = self.liquidity;
        self.fee_growth_0_x64 = buy_coverage_result.fee_growth;
        self.protocol_fees_owed_0 += buy_coverage_result.protocol_fee;
        self.founders_fees_owed_0 += buy_coverage_result.founders_fee;

        Ok(())
    }

    /// Swap for AMM
    /// TODO: Implement
    pub fn swap() -> Result<()> {
        Ok(())
    }

    /// Update coverage
    ///
    /// Start from the current sqrt price and move upwards
    /// until either the liquidity ends or coverage is fullfilled
    ///
    /// The coverage is bought from the tick_array_pool but the
    /// state has to be synced in the pool as well
    ///
    /// The operation will move from the lowest tick array and fill up
    /// ticks until either the coverage amount is met or there is no
    /// more liquidity in the ticks.
    ///
    /// The method will return the calculated premium that the used have
    /// to deposit in the premium vault (token_vault_1)
    ///
    /// The fees will be drawn from the premium
    /// * Parameters
    /// - tick_array_pool: keeps and overview over liquidity at 3 tick arrays
    /// - coverage_position: keeps track of covered amounts at each tick
    /// - coverage_amount_delta: the change in coverage amount
    /// - increase_coverage <bool>
    /// If increase_coverage = true then increase coverage
    pub fn update_coverage(
        &mut self,
        mut tick_array_pool: TickArrayPool,
        mut coverage_position: RefMut<CoveragePosition>,
        coverage_amount_delta: u128,
        expiry_ts: i64,
        increase_coverage: bool,
        is_target_amount: bool,
    ) -> Result<BuyCoverageResult> {
        msg!("update_coverage method");
        // If we are reducing the coverage position
        // we are actually moving downwards from
        // the current coverage position
        // Nothing to cover
        if coverage_amount_delta == 0 {
            return Err(SureError::InvalidAmount.into());
        }

        let mut coverage_amount_remaining = coverage_amount_delta; // given in u64
        let mut coverage_amount: u128 = 0; // amount that is covered
        let mut coverage_premium: u128 = 0; // premium to be deposited

        let mut current_liquidity = self.liquidity; // u64 given in token 0
        let mut current_fee_growth = self.fee_growth_0_x64; //Q64.64
        msg!(&format!(
            "current_tick_index {}, sqrt_price_x64: {}",
            self.current_tick_index, self.sqrt_price_x64
        ));
        // If increase coverage start at the current price, which is the global min
        let mut current_tick_index = coverage_position.last_covered_tick_index;

        msg!(&format!("current_tick_index: {}", current_tick_index));

        let mut current_array_index: usize = 0; // which array in the tick array pool
        let mut current_protocol_fee: u128 = 0; // Q32.32 to represent the fee
        let mut current_founders_fee: u128 = 0; // Q32.32
        let mut current_sqrt_price = get_sqrt_ratio_at_tick(current_tick_index);
        // set the sqrt price limit at either the max of the tick array
        // or the minimum of the coverage position
        let sqrt_price_limit = if increase_coverage {
            tick_array_pool.max_sqrt_price_x32(self.tick_spacing)?
        } else {
            coverage_position.get_lowest_sqrt_price_x32()
        };

        // Fee rates
        let fee_rate = self.fee_rate;

        // while the coverage amount remaining is greater
        // than 0 and there is more liquidity
        msg!(&format!(
            "> buy coverage. coverage_amount_remaining: {}",
            coverage_amount_remaining
        ));
        while coverage_amount_remaining > 0
            && current_sqrt_price != sqrt_price_limit
            && !coverage_position.is_tick_index_out_of_bounds(
                current_tick_index,
                self.tick_spacing,
                true,
            )
        {
            // find the next tick index with enough liquidity
            // if increase_coverage move left -> right
            let (next_array_index, next_tick_index) = tick_array_pool.find_next_free_tick_index(
                current_tick_index,
                self.tick_spacing,
                !increase_coverage,
                current_array_index,
            )?;
            msg!(&format!(
                "> next array index {}, next tick index: {}",
                next_array_index, next_tick_index
            ));

            // find the price/premium at current tick
            let next_sqrt_price_x64 = get_sqrt_ratio_at_tick(next_tick_index);
            println!("next_sqrt_price_x64 {}", next_sqrt_price_x64);
            // @checkpoint : need to calculate exact tick math for buying coverage. Start by unit testing tick
            let current_tick =
                tick_array_pool.get_tick(next_array_index, next_tick_index, self.tick_spacing)?;

            // calculate tick change
            let current_covered_amount =
                coverage_position.get_coverage_at_tick_index(next_tick_index, self.tick_spacing)?;
            let coverage_tick_delta = if increase_coverage {
                current_tick
                    .get_available_liquidity()
                    .min(coverage_amount_remaining)
            } else {
                current_covered_amount.min(coverage_amount_remaining)
            };

            msg!(&format!(
                "> current_covered_amount {}",
                current_covered_amount
            ));
            // Calculate the fee and the in and out amounts
            let (fee_amount, amount_in, amount_out) = current_tick.calculate_coverage_delta(
                next_tick_index,
                coverage_tick_delta,
                current_covered_amount,
                fee_rate,
                coverage_position.expiry_ts,
                expiry_ts,
                increase_coverage,
            )?;

            // calculate remaining coverage
            if increase_coverage {
                coverage_amount_remaining = coverage_amount_remaining
                    .checked_sub(coverage_tick_delta)
                    .ok_or(SureError::OverflowU64)?;
                coverage_amount = coverage_amount
                    .checked_add(coverage_tick_delta)
                    .ok_or(SureError::OverflowU64)?;
            } else {
                coverage_amount_remaining = coverage_amount_remaining
                    .checked_sub(coverage_tick_delta)
                    .ok_or(SureError::OverflowU64)?;
                coverage_amount = coverage_amount
                    .checked_sub(coverage_tick_delta)
                    .ok_or(SureError::OverflowU64)?;
            }

            // calculate premium based on current price
            coverage_premium += if increase_coverage {
                amount_in
            } else {
                amount_out
            };

            // Calculate fees
            let (next_protocol_fee, next_founders_fee, next_fee_growth) = calculate_fees(
                fee_amount,
                self.protocol_fee_rate,
                self.founders_fee_rate,
                self.liquidity,
                current_protocol_fee,
                current_founders_fee,
                current_fee_growth,
            )?;
            current_protocol_fee = next_protocol_fee;
            current_founders_fee = next_founders_fee;
            current_fee_growth = next_fee_growth;

            // Update tick
            let (fee_growth_0, fee_growth_1) = (current_fee_growth, self.fee_growth_1_x64);

            let (tick_update, next_liquidity) = current_tick.calculate_coverage_update(
                increase_coverage,
                current_liquidity,
                coverage_tick_delta,
                fee_growth_0,
                fee_growth_1,
            )?;
            tick_array_pool.update_tick(
                next_array_index,
                next_tick_index,
                self.tick_spacing,
                &tick_update,
            )?;

            current_liquidity = next_liquidity;

            // Update coverage position
            coverage_position.update_coverage_at_tick(
                next_tick_index,
                self.tick_spacing,
                coverage_tick_delta,
                expiry_ts,
                increase_coverage,
            )?;

            // Calculate sub fees
            let last_tick_index_in_array = tick_array_pool.is_last_tick_index_in_array(
                next_array_index,
                next_tick_index,
                self.tick_spacing,
            )?;
            current_array_index = if last_tick_index_in_array {
                current_array_index + 1
            } else {
                current_array_index
            };
            current_tick_index = next_tick_index;
            current_sqrt_price = next_sqrt_price_x64;
        }

        Ok(BuyCoverageResult {
            liquidity: current_liquidity,
            coverage_amount,
            premium: coverage_premium,
            fee_growth: current_fee_growth,
            protocol_fee: current_protocol_fee,
            founders_fee: current_founders_fee,
            next_sqrt_price: current_sqrt_price,
            next_tick_index: current_tick_index,
        })
    }
}

pub struct BuyCoverageResult {
    liquidity: u128,
    coverage_amount: u128,
    premium: u128,
    fee_growth: u128,
    protocol_fee: u128,
    founders_fee: u128,
    next_sqrt_price: u128,
    next_tick_index: i32,
}

impl BuyCoverageResult {
    // TODO: check if cost of updated coverage can be u64 without shifting and casting
    pub fn get_total_cost_of_coverage(&self) -> Result<u64> {
        self.premium
            .checked_add(self.fee_growth)
            .map(|x| (x >> 64) as u64)
            .ok_or(SureError::AdditionQ3232OverflowError.into())
    }

    pub fn get_coverage_amount(&self) -> u128 {
        self.coverage_amount
    }
}

#[event]
pub struct CreatePool {
    #[index]
    pub name: String,
    pub smart_contract: Pubkey,
    pub insurance_fee: u16,
}

#[cfg(test)]
pub mod pool_testing {

    use std::{
        borrow::BorrowMut,
        ops::Add,
        time::{Duration, SystemTime},
    };

    use crate::states::{
        coverage::coverage_testing::CoveragePositionProto,
        tick_v2::{
            tick_array_pool_testing::TickArrayPoolProto, tick_array_testing::TickArrayProto,
        },
    };

    use super::*;

    #[derive(Default)]
    pub struct PoolProto {
        /// ProductId
        pub product_id: u8, // 1
        /// Name of pool
        pub name: String, // 4 + 200 bytes

        /// space between each tick in basis point
        pub tick_spacing: u16, // 2 bytes

        /// fee rate for each transaction in pool
        // hundreth of a basis point i.e. fee_rate = 1 = 0.01 bp = 0.00001%
        pub fee_rate: u16, //2 bytes

        /// Protocol fee of transaction
        /// (1/x)% of fee_rate
        pub protocol_fee_rate: u16, // 2 bytes

        /// Founder fee of transaction
        /// (1/x)% of fee_rate
        pub founders_fee_rate: u16, // 2 bytes

        /// Liquidity in Pool
        pub liquidity: u128, // 8 bytes

        /// The current market price as
        /// use Q64.64 - 64 bytes at each
        /// side of decimal point
        pub sqrt_price_x64: u128, // 8 bytes

        /// Current tick index corrensponding to sqrt price
        pub current_tick_index: i32, // 4 bytes

        /// Tokens in vault 0 that is owed to the sure
        pub protocol_fees_owed_0: u128, // 8 bytes
        pub founders_fees_owed_0: u128, // 8 bytes
        /// total fees in vault a collected per unit of liquidity
        pub fee_growth_0_x64: u128, // 8 bytes

        /// Tokens in vault 0 that is owed to the sure
        pub protocol_fees_owed_1: u128, // 8 bytes
        pub founders_fees_owed_1: u128, // 8 bytes
        /// total fees collected in vault b per unit of liquidity
        pub fee_growth_1_x64: u128, // 8 bytes

        /// Used liquidity
        pub used_liquidity: u128, // 8 bytes
    }

    impl PoolProto {
        pub fn new() -> Self {
            Default::default()
        }

        pub fn set_tick_spacing(mut self, tick_spacing: u16) -> Self {
            self.tick_spacing = tick_spacing;
            self
        }

        pub fn into_pool(self) -> Pool {
            Pool {
                product_id: self.product_id,
                name: self.name,
                tick_spacing: self.tick_spacing,
                fee_rate: self.fee_rate,
                protocol_fee_rate: self.protocol_fee_rate,
                founders_fee_rate: self.founders_fee_rate,
                liquidity: self.liquidity,
                sqrt_price_x64: self.sqrt_price_x64,
                current_tick_index: self.current_tick_index,
                protocol_fees_owed_0: self.protocol_fees_owed_0,
                founders_fees_owed_0: self.founders_fees_owed_0,
                protocol_fees_owed_1: self.protocol_fees_owed_1,
                founders_fees_owed_1: self.founders_fees_owed_1,
                fee_growth_0_x64: self.fee_growth_0_x64,
                fee_growth_1_x64: self.fee_growth_1_x64,
                used_liquidity: self.used_liquidity,
                ..Default::default()
            }
        }
    }

    // Test buying coverage from a pool
    #[test]
    fn test_buy_coverage() {
        let pool_proto = PoolProto::new();
        let mut pool = pool_proto.set_tick_spacing(20).into_pool();
        let tick_sequence = TickArrayPoolProto::new(0, 20);
        let tick_array_pool = TickArrayPool::new(
            tick_sequence[0].borrow_mut(),
            Some(tick_sequence[1].borrow_mut()),
            Some(tick_sequence[2].borrow_mut()),
        );
        let coverage_position_proto = CoveragePositionProto::new();
        let coverage_position = coverage_position_proto.build();
        let coverage_amount_delta = 1_000_000;
        let expiry_ts = 1691749155 as i64;

        let res = pool
            .update_coverage(
                tick_array_pool,
                coverage_position.borrow_mut(),
                coverage_amount_delta,
                expiry_ts,
                true,
                true,
            )
            .unwrap();
        println!("res cover amount: {}", res.coverage_amount);
    }
}
