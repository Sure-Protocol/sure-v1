///! Tick contains methods to manage tick
///!

use anchor_lang::prelude::*;
use crate::states::liquidity;

/// Tick acount (PDA) is used to hold information about 
/// the liquidity at a current tick
#[account]
#[derive(Default)]
pub struct Tick{
    /// The bump identity of the PDA
    pub bump: u8, // 1 byte

    /// The active liquidity at the tick
    pub liquidity: u64, // 8bytes

    /// The tick in basis points
    pub tick: u32, // 8 bytes 

    /// Boolean representing whether the liquidity is active
    pub active: bool, // 1 byte 

    // start liquidity
    pub first_liquidity_position: Pubkey, // 32 bytes

    // last liquidity
    pub last_liquidity_position: Pubkey, // 32 bytes

    /// Current Liquidity
    pub current_liquidity: Pubkey, // 32bytes
}

impl Tick {
    /// Tick is implemented as a stack

    /// Push a new liquidity position onto the stack
    pub fn push(&mut self,liquidity_position: &mut Account<liquidity::LiquidityPosition>) -> Result<()> {
        let previous_last_liquidity_position = self.last_liquidity_position;
        
        self.last_liquidity_position = liquidity_position.key();
        liquidity_position.next_liquidity_position = Pubkey::default();
        liquidity_position.previous_liquidity_position = previous_last_liquidity_position;
        
        Ok(())
    }


    /// Remove a given liquidity position from the stac
    pub fn remove(&mut self,liquidity_position: &liquidity::LiquidityPosition) -> Result<()> {
        Ok(())
    }
}