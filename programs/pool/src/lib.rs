pub mod instructions;
pub mod managers;
pub mod states;
pub mod utils;

use crate::states::*;
use crate::utils::errors::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program;
use anchor_spl::*;
use mpl_token_metadata::instruction::{create_metadata_accounts_v2, update_metadata_accounts_v2};
use mpl_token_metadata::state::Creator;
use vipers::prelude::*;

use crate::instructions::*;

declare_id!("D47wvD2bTDXR9XqqHdP8bwYSXu2QPMW6fGHg2aEBKunM");

#[program]
pub mod sure_pool {

    use super::*;

    // ------------ Pool -----------------------------------------------
    /// Create an insurance pool for a smart contract
    /// also create an associated vault to hold the tokens
    ///
    /// # Arguments
    /// * ctx:
    /// * insurance_fee: fee taken on each insurance bought. In basis points (1bp = 0.01%)
    /// * range_size: The size of the ranges in which users can provide insurance
    /// * name: [optional] Name of the pool
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        name: String,
        tick_spacing: u16,
    ) -> Result<()> {
        instructions::initialize_pool::handler(ctx, name, tick_spacing)
    }
}
