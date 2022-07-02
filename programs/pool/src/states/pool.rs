use crate::{
    fee,
    helpers::tick::{get_sqrt_ratio_at_tick, get_tick_at_sqrt_ratio},
    utils::{errors::SureError, liquidity::calculate_new_liquidity},
};
use anchor_lang::{prelude::*, Result};

use super::{liquidity::LiquidityPosition, seeds::SURE_TOKEN_POOL_SEED};

/// Product Pool
/// The product pool holds information regarding the specific product
///
#[account]
pub struct ProductPool {
    // Product
    pub productId: u8, // 1

    /// Pools in the ProductPool
    pub pools: Vec<Pubkey>, // 4 + 32*64, 64 pools for each product
}

impl ProductPool {
    const MAX_POOLS: usize = 64;
    pub const SPACE: usize = 1 + 4 + 32 * 64;

    pub fn initialize(&mut self, productId: u8) -> Result<()> {
        self.productId = productId;
        self.pools = Vec::new();
        Ok(())
    }

    pub fn add_pool(&mut self, pool_pk: Pubkey) -> Result<()> {
        if self.pools.len() >= ProductPool::MAX_POOLS {
            return Err(SureError::PoolsInProductPoolExceeded.into());
        }
        self.pools.push(pool_pk);
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
pub struct Pool {
    /// bump
    pub bump: u8, // 1 byte

    /// ProductId
    pub productId: u8,

    /// Name of pool
    pub name: String, // 4 + 200 bytes

    /// Founder of pool
    pub founder: Pubkey,

    /// space between each tick in basis point
    pub tick_spacing: u16,

    /// fee rate for each transaction in pool
    // hundreth of a basis point i.e. fee_rate = 1 = 0.01 bp = 0.00001%
    pub fee_rate: u16,

    /// Protocol fee of transaction
    /// (1/x)% of fee_rate
    pub protocol_fee_rate: u16,

    /// Founder fee of transaction
    /// (1/x)% of fee_rate
    pub founders_fee_rate: u16,

    /// Liquidity in Pool
    pub liquidity: u64, // 8 bytes

    /// The current market price as
    /// use Q32.32 meaning 32 bytes at each
    /// side of decimal point
    pub sqrt_price_x32: u64, // 16bytes

    /// Tokens in vault a that is owed to the sure
    pub fees_token_a_owed: u64,
    /// total fees in vault a collected per unit of liquidity
    pub fee_growth_0_x32: u64,

    /// Tokens in vault a that is owed to the sure
    pub fees_token_b_owed: u64,
    /// total fees collected in vault b per unit of liquidity
    pub fee_growth_1_x32: u64,

    /// Current tick index corrensponding to sqrt price
    pub current_tick_index: i32,

    /// Token mint A of pool
    pub token_mint_a: Pubkey, // 32 bytes
    pub pool_vault_a: Pubkey, //32 bytes

    /// Token mint B of pool
    pub token_mint_b: Pubkey, // 32 bytes
    pub pool_vault_b: Pubkey, //32 bytes

    /// Used liquidity
    pub used_liquidity: u64, // 8 bytes
}

impl Pool {
    pub const SPACE: usize = 1 + 32 + 32 + 8 + 8 + 4 + 200;

    pub fn seeds(&self) -> [&[u8]; 5] {
        [
            &SURE_TOKEN_POOL_SEED.as_bytes() as &[u8],
            self.token_mint_a.as_ref(),
            self.token_mint_b.as_ref(),
            self.tick_spacing.to_le_bytes().as_ref(),
            &[self.bump],
        ]
    }

    pub fn initialize(
        &mut self,
        bump: u8,
        productId: u8,
        name: String,
        founder: Pubkey,
        tick_spacing: u16,
        fee_package: fee::FeePackage,
        sqrt_price_x32: u64,
        token_mint_a: Pubkey,
        token_mint_b: Pubkey,
        pool_vault_a: Pubkey,
        pool_vault_b: Pubkey,
    ) -> Result<()> {
        self.bump = bump;
        self.productId = productId;
        self.name = name;
        self.founder = founder;
        self.tick_spacing = tick_spacing;
        fee_package.validate_fee_rates()?;

        self.fee_rate = fee_package.fee_rate;
        self.protocol_fee_rate = fee_package.protocol_fee_rate;
        self.founders_fee_rate = fee_package.founders_fee_rate;

        self.liquidity = 0;
        self.current_tick_index = get_tick_at_sqrt_ratio(sqrt_price_x32)?;
        self.sqrt_price_x32 = sqrt_price_x32;
        if token_mint_a.ge(&token_mint_b) {
            return Err(SureError::WrongTokenMintOrder.into());
        }
        self.token_mint_a = token_mint_a;
        self.token_mint_b = token_mint_b;
        self.pool_vault_a = pool_vault_a;
        self.pool_vault_b = pool_vault_b;

        Ok(())
    }

    /// Update fees collected by the pool
    /// Should happen after a transactions
    pub fn update_post_transaction(
        &mut self,
        liquidity: u64,
        tick: i32,
        fee_growth: u64,
        protocol_fee: u64,
        is_fee_in_a: bool,
    ) -> Result<()> {
        self.liquidity = liquidity;
        self.sqrt_price_x32 = get_sqrt_ratio_at_tick(tick)?;
        if is_fee_in_a {
            self.fee_growth_0_x32 = fee_growth;
            self.fees_token_a_owed += protocol_fee;
        } else {
            self.fee_growth_1_x32 = fee_growth;
            self.fees_token_b_owed += protocol_fee;
        }

        Ok(())
    }

    pub fn update_liquidity(&mut self, liquidity: u64) -> Result<()> {
        self.liquidity = liquidity;
    }

    /// Get the current tick index from the
    /// current sqrt price
    pub fn get_current_tick_index(&self) -> Result<i32> {
        get_tick_at_sqrt_ratio(self.sqrt_price_x32)
    }
    /// Update the fee package
    pub fn update_fee_package(&mut self, fee_package: fee::FeePackage) -> Result<()> {
        fee_package.validate_fee_rates()?;
        self.fee_rate = fee_package.fee_rate;
        self.protocol_fee_rate = fee_package.protocol_fee_rate;
        self.founders_fee_rate = fee_package.founders_fee_rate;
        Ok(())
    }

    pub fn is_position_in_range(&self, position: &LiquidityPosition) -> Result<bool> {
        let current_tick_index = get_tick_at_sqrt_ratio(self.sqrt_price_x32)?;
        if current_tick_index >= position.tick_index_lower
            && current_tick_index < position.tick_index_upper
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn next_pool_liquidity(&self, position: &LiquidityPosition, delta: i64) -> Result<u64> {
        if self.is_position_in_range(position)? {
            calculate_new_liquidity(self.liquidity, delta)
        } else {
            Ok(self.liquidity)
        }
    }
}

#[event]
pub struct CreatePool {
    #[index]
    pub name: String,
    pub smart_contract: Pubkey,
    pub insurance_fee: u16,
}
