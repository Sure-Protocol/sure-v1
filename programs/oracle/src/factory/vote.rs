use std::ops::Div;

use crate::utils::{convert_x32_to_u64, VOTE_STAKE_RATE};

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
pub fn calculate_stake(vote_power: u64, decimals: u8) -> u64 {
    convert_x32_to_u64(calculate_stake_x32(vote_power), decimals)
}

/// calculate stake based on vote power
///
/// ### Arguments
/// * vote_power: Q32.32
///
/// ### Returns
/// vote_power  / 100
pub fn calculate_stake_x32(vote_power: u64) -> u64 {
    vote_power.div(VOTE_STAKE_RATE as u64)
}
