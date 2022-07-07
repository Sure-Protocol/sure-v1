use crate::common::errors::SureError;
use anchor_lang::prelude::*;
/// the minimum tick i is calculated as
/// i_min = ln(p_min)/ln(1.0001)
/// where
///     p_i = sqrt(1.0001)^i, which ensures each
///     tick corrensponds to 1bp = 0.01%
///     p_min can be set to -2**32
/// thus
/// i_min = ln(-2**32)/ln(1.0001) > 221818
/// credit to Cykura protocol for optimizing variables
pub const MIN_TICK_INDEX: i32 = -221_818;

/// max tick is calculated in the same way as with
/// the min tick, except for p=p_max=2**32
pub const MAX_TICK_INDEX: i32 = 221_818;

/// MIN and MAX sqrt ratios
/// Calculated as
/// 1.0001**(tick/2)*(2**32)
pub const MIN_SQRT_RATIO: u64 = 65537;
pub const MAX_SQRT_RATIO: u64 = 281472331703918;

pub const UNIX_TIME_IN_YEARS: i128 = 31556926;
pub const BASE_FACTOR: f64 = 1.0001;
pub const Q32_RESOLUTION: u8 = 32;

/// Calculate the sqrt price ratio at
/// the given tick_index
pub fn get_sqrt_ratio_at_tick(tick_index: i32) -> Result<u64> {
    if tick_index > MAX_TICK_INDEX || tick_index < MIN_TICK_INDEX {
        return Err(SureError::TickOutOfRange.into());
    }
    let base2 = 2_f64;
    let factor = base2.powf(32.0);
    let sqrt_ratio = BASE_FACTOR.powf((tick_index / 2).into()) * factor;
    Ok(sqrt_ratio as u64)
}

/// Calculate premium difference
///
/// Premium are given in bp 0.01% = 0.0001
/// yearly premium
/// P_a = A*sqrt(P)^2/10_000 , A: amount u64, P: price Q32.32
///
/// O_1 = sqrt(P)/100
/// P_a = A*O_1^2
///
pub fn calculate_yearly_premium(sqrt_price_x32: u64, amount: u64) -> Result<u64> {
    let O1_x32 = sqrt_price_x32.wrapping_div(100);

    let O1_x32_2 = O1_x32
        .checked_mul(O1_x32)
        .ok_or(SureError::MultiplictationQ3232Overflow)?;
    // u64*32.32 = 32.32
    let premium_x32 = amount
        .checked_mul(O1_x32_2)
        .ok_or(SureError::MultiplictationQ3232Overflow)?;
    let premium = (premium_x32 >> 32) as u64;

    Ok(premium)
}

/// Calculate the premium change
///
/// ### Arguments
/// - prev_premium: the previous premium for the period t_0 t_1
/// - sqrt_price_x32: constant. The sqrt price
/// - amount: new amount to be covered
/// - expiry_ts: the new expiry time
///
/// Returns: tuple (Increased premium, premium change)
///
/// ### Calculation:
/// - premium^(A_0)_(t_0,t_1) -  premium^(A_1)_(t_0+e,t_1)
///
/// where t_0 + e > t_0 i.e. e > 0
pub fn calculate_premium_diff(
    remaining_premium: u64,
    sqrt_price_x32: u64,
    amount: u64,
    expiry_ts: i64,
) -> Result<(bool, u64)> {
    let new_premium = calculate_premium(sqrt_price_x32, amount, expiry_ts)?;
    let (increase_premium, premium_delta) = if new_premium > remaining_premium {
        (true, new_premium - remaining_premium)
    } else {
        (false, remaining_premium - new_premium)
    };
    return Ok((increase_premium, premium_delta));
}

pub fn calculate_premium(sqrt_price_x32: u64, amount: u64, expiry_ts: i64) -> Result<u64> {
    let yearly_premium = calculate_yearly_premium(sqrt_price_x32, amount)?;
    let time = Clock::get()?;
    let t0 = time.unix_timestamp;
    let premium = time_fraction(yearly_premium, t0, expiry_ts)?;
    Ok(premium)
}

/// Time fraction
///
/// calculates
///     num * (t0-t1)/SECONDS_IN_YEAR
pub fn time_fraction(num: u64, t0: i64, t1: i64) -> Result<u64> {
    if t0 >= t1 {
        return Err(SureError::InvalidTimestamp.into());
    }
    let t0_xi64 = (t0 as i128) << 64;
    let t1_xi64 = (t1 as i128) << 64;
    let time_frac_xi64 = (t1_xi64 - t0_xi64)
        .checked_div(UNIX_TIME_IN_YEARS)
        .ok_or(SureError::DivisionQ3232Error)?;
    let num_xu64 = (num as u128) << 64;
    let num_frac_xu64 = num_xu64
        .checked_mul(time_frac_xi64.try_into().unwrap())
        .ok_or(SureError::MultiplictationQ3232Overflow)?;
    Ok((num_frac_xu64 >> 64) as u64)
}

/// Calculate
///
/// tick = 2*ln(sp_x32/2^32) / ln(1.0001)
///
pub fn get_tick_at_sqrt_ratio(sqrt_price_x32: u64) -> Result<i32> {
    if sqrt_price_x32 < MIN_SQRT_RATIO || sqrt_price_x32 > MAX_SQRT_RATIO {
        return Err(SureError::SqrtRatioNotWithinRange.into());
    }

    let sqrt_base = f64::sqrt(1.0001);
    let sqrt_price: f64 = (sqrt_price_x32 as f64) / f64::powf(2.0, 32.0);
    let ln_sqrt_price = sqrt_price.ln();

    let tick = 2.0 * ln_sqrt_price / BASE_FACTOR.ln();
    let tick_i32 = tick.floor() as i32;
    Ok(tick_i32)
}

#[cfg(test)]
mod tests {
    use super::*;
}
