use super::errors::SureError;
use anchor_lang::prelude::*;

const Q3232_FP_SHIFT: u8 = 32;

/// Multiply a u64 with a Q32.32 and round down
/// to nearest u64
///
/// * Input
/// ui<u64>: unsigned integer
/// fp<Q32.32>: floating point 32.32
///
/// returns: u64
pub fn mul_round_down_Q3232(ui: u64, fp: u64) -> Result<u64> {
    if ui == 0 || fp == 0 {
        return Ok(0);
    }
    let product = ui
        .checked_mul(fp)
        .ok_or(SureError::MultiplictationQ3232Overflow)?;
    let result = (product >> Q3232_FP_SHIFT) as u64;
    Ok(result)
}
