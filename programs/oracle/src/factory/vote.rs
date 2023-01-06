use anchor_lang::prelude::*;
use std::ops::Div;

use crate::utils::{convert_x32_to_u64, SureError, VOTE_STAKE_RATE};

/// Calculate the necessary stake
///
/// the stake is some % of the voting power
///
/// ### Arguments
/// * vote_power: Q32.32
/// * decimals: u8
///
/// ### Result
///
pub fn calculate_stake(vote_power: u64, vote_stake_rate: u32) -> u64 {
    calculate_stake_x32(vote_power, vote_stake_rate).unwrap()
}

/// calculate stake based on vote power
///
/// ### Arguments
/// * vote_power: Q32.32
///
/// ### Returns
/// vote_power  / 100
pub fn calculate_stake_x32(vote_power: u64, vote_stake_rate: u32) -> Result<u64> {
    let res = match vote_power.checked_div(vote_stake_rate as u64) {
        Some(val) => val,
        None => return Err(SureError::DivideOperationFailure.into()),
    };
    Ok(res)
}
