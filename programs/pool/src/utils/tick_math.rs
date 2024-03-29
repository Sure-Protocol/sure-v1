use std::ops::Shr;

use super::{uint::U256, SureError};
use anchor_lang::prelude::*;
/// the minimum tick i is calculated as
/// i_min = ln(p_min)/ln(1.0001)
/// where
///     p_i = sqrt(1.0001)^i, which ensures each
///     tick corrensponds to 1bp = 0.01%
///     p_min can be set to -2**64
/// thus
/// i_min = ln(-2**64)/ln(1.0001) > 443636
pub const MIN_TICK_INDEX: i32 = -443_636;

/// max tick is calculated in the same way as with
/// the min tick, except for p=p_max=2**64
pub const MAX_TICK_INDEX: i32 = 443_636;

/// MIN and MAX sqrt ratios
/// Calculated as
/// 1.0001**(tick/2)*(2**64)
pub const MIN_SQRT_RATIO: u128 = 4295048016;
pub const MAX_SQRT_RATIO: u128 = 79226673515401279992447579055;

pub const UNIX_TIME_IN_YEARS: i128 = 31556926;
pub const BASE_FACTOR: f64 = 1.0001;
pub const Q32_RESOLUTION: u8 = 32;

/// Get the price at a given tick
///
/// Assume the relation
///  - P = 1.0001^(t)
/// where P is the price and t is the tick
pub fn get_price_ratio_at_tick(tick: i32) -> u128 {
    1 as u128
}

/// TODO: find the correct ratios
fn get_price_ratio_at_negative_tick() -> u128 {
    return 0;
}

/// Calculate the sqrt price
///
/// Calculates √1.0001^tick as
///     √1.0001^(sign(tick) x (1 + 2 + 4 + ... + 2^n)), where sign(tick) x sum_0^n 2^n = tick
///    =√1.0001^(sign(tick) x 1) x √1.0001^(sign(tick) x 2) x ... x √1.0001^(sign(tick) x 2^n)
///
/// When sign(tick) == -1, √1.0001^(2^m) is represented as Q64.64 by
/// K_m = √1.0001^(2^m) x 2^64
///
/// When sign(tick) == 1, √1.0001^(2^m) is represented as Q32.96 as
/// Q_m = √1.0001^(2^m) x 2^96
pub fn get_sqrt_ratio_at_tick(tick: i32) -> u128 {
    if tick >= 0 {
        get_sqrt_ratio_at_positive_tick(tick)
    } else {
        get_sqrt_ratio_at_negative_tick(tick)
    }
}

pub fn get_sqrt_ratio_at_negative_tick(tick_i: i32) -> u128 {
    let tick = tick_i.abs();
    let mut ratio: u128 = if tick & 1 != 0 {
        18445821805675392311
    } else {
        18446744073709551616
    };

    if tick & 2 != 0 {
        ratio = (ratio * 18444899583751176498) >> 64
    }
    if tick & 4 != 0 {
        ratio = (ratio * 18443055278223354162) >> 64
    }
    if tick & 8 != 0 {
        ratio = (ratio * 18439367220385604838) >> 64
    }
    if tick & 16 != 0 {
        ratio = (ratio * 18431993317065449817) >> 64
    }
    if tick & 32 != 0 {
        ratio = (ratio * 18417254355718160513) >> 64
    }
    if tick & 64 != 0 {
        ratio = (ratio * 18387811781193591352) >> 64
    }
    if tick & 128 != 0 {
        ratio = (ratio * 18329067761203520168) >> 64
    }
    if tick & 256 != 0 {
        ratio = (ratio * 18212142134806087854) >> 64
    }
    if tick & 512 != 0 {
        ratio = (ratio * 17980523815641551639) >> 64
    }
    if tick & 1024 != 0 {
        ratio = (ratio * 17526086738831147013) >> 64
    }
    if tick & 2048 != 0 {
        ratio = (ratio * 16651378430235024244) >> 64
    }
    if tick & 4096 != 0 {
        ratio = (ratio * 15030750278693429944) >> 64
    }
    if tick & 8192 != 0 {
        ratio = (ratio * 12247334978882834399) >> 64
    }
    if tick & 16384 != 0 {
        ratio = (ratio * 8131365268884726200) >> 64
    }
    if tick & 32768 != 0 {
        ratio = (ratio * 3584323654723342297) >> 64
    }
    if tick & 65536 != 0 {
        ratio = (ratio * 696457651847595233) >> 64
    }
    if tick & 131072 != 0 {
        ratio = (ratio * 26294789957452057) >> 64
    }
    if tick & 262144 != 0 {
        ratio = (ratio * 37481735321082) >> 64
    }

    ratio
}

/// Calculate the sqrt price ratio at
/// the given tick_index
/// If tick > 0, need less precision in decimals, consider Q32.96 x Q32.96, where 96 is the binary scaling factor
/// then rsh with 32 to get Q64.64
pub fn get_sqrt_ratio_at_positive_tick(tick: i32) -> u128 {
    let mut ratio: U256 = if tick & 1 != 0 {
        U256::from(79232123823359799118286999567 as u128)
    } else {
        U256::from(79228162514264337593543950336 as u128)
    };

    if tick & 2 != 0 {
        ratio = ratio
            .checked_mul(U256::from(79236085330515764027303304731 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 4 != 0 {
        ratio = ratio
            .checked_mul(U256::from(79244008939048815603706035061 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 8 != 0 {
        ratio = ratio
            .checked_mul(U256::from(79259858533276714757314932305 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 16 != 0 {
        ratio = ratio
            .checked_mul(U256::from(79291567232598584799939703904 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 32 != 0 {
        ratio = ratio
            .checked_mul(U256::from(79355022692464371645785046466 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 64 != 0 {
        ratio = ratio
            .checked_mul(U256::from(79482085999252804386437311141 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 128 != 0 {
        ratio = ratio
            .checked_mul(U256::from(79736823300114093921829183326 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 256 != 0 {
        ratio = ratio
            .checked_mul(U256::from(80248749790819932309965073892 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 512 != 0 {
        ratio = ratio
            .checked_mul(U256::from(81282483887344747381513967011 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 1024 != 0 {
        ratio = ratio
            .checked_mul(U256::from(83390072131320151908154831281 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 2048 != 0 {
        ratio = ratio
            .checked_mul(U256::from(87770609709833776024991924138 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 4096 != 0 {
        ratio = ratio
            .checked_mul(U256::from(97234110755111693312479820773 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 8192 != 0 {
        ratio = ratio
            .checked_mul(U256::from(119332217159966728226237229890 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 16384 != 0 {
        ratio = ratio
            .checked_mul(U256::from(179736315981702064433883588727 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 32768 != 0 {
        ratio = ratio
            .checked_mul(U256::from(407748233172238350107850275304 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 65536 != 0 {
        ratio = ratio
            .checked_mul(U256::from(2098478828474011932436660412517 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 131072 != 0 {
        ratio = ratio
            .checked_mul(U256::from(55581415166113811149459800483533 as u128))
            .unwrap()
            .shr(96 as u128)
    }
    if tick & 262144 != 0 {
        ratio = ratio
            .checked_mul(U256::from(38992368544603139932233054999993551 as u128))
            .unwrap()
            .shr(96 as u128)
    }

    // Return result as Q64.64 by shifting(div) Q32.96 by 32 bytes
    ratio.shr(32 as u128).as_u128()
}

/// Calculate premium difference
///
/// # Arguments
/// - sqrt_price_x64: <U64.64>, price in %
/// - amount: Q64.64
/// Premium are given in bp 0.01% = 0.0001
/// yearly premium
/// P_a = A*sqrt(P)^2/10_000 , A: amount u64, P: price Q32.32
///
/// O_1 = sqrt(P)/100
/// P_a = A*O_1^2
///
/// Is a BtoA trade -> price increasing. Should choose sqrt_price_target to
/// be the smallest sqrt_price in tick sequence
pub fn calculate_yearly_premium(
    sqrt_price_target: u128,
    sqrt_price_current: u128,
    amount: u128,
) -> Result<u64> {
    //get_conditional_delta_amount_1(sqrt_price_target, sqrt_price_current, amount as i128)
    Ok(2)
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
    remaining_premium: u128,
    sqrt_price_target: u128,
    sqrt_price_current: u128,
    amount: u128,
    expiry_ts: i64,
) -> Result<(bool, u128)> {
    let new_premium = calculate_premium(sqrt_price_target, sqrt_price_current, amount, expiry_ts)?;
    let (increase_premium, premium_delta) = if new_premium > remaining_premium {
        (true, new_premium - remaining_premium)
    } else {
        (false, remaining_premium - new_premium)
    };
    return Ok((increase_premium, premium_delta));
}

pub fn calculate_premium(
    sqrt_price_target: u128,
    sqrt_price_current: u128,
    amount: u128,
    expiry_ts: i64,
) -> Result<u128> {
    let yearly_premium = calculate_yearly_premium(sqrt_price_target, sqrt_price_current, amount)?;
    let time = Clock::get()?;
    let t0 = time.unix_timestamp;
    let premium = time_fraction(yearly_premium, t0, expiry_ts)?;
    Ok(premium)
}

/// Time fraction
///
/// calculates
///     num * (t0-t1)/SECONDS_IN_YEAR
pub fn time_fraction(num: u64, t0: i64, t1: i64) -> Result<u128> {
    if t0 >= t1 {
        return Err(SureError::InvalidTimestamp.into());
    }
    let t0_xi64 = (t0 as i128) << 64;
    let t1_xi64 = (t1 as i128) << 64;
    let time_frac_xi64 = (t1_xi64 - t0_xi64)
        .checked_div(UNIX_TIME_IN_YEARS)
        .ok_or(SureError::DivisionQ3232Error)?;
    let num_xu64 = (num as u128) << 64;
    let num_frac_x64 = num_xu64
        .checked_mul(time_frac_xi64.try_into().unwrap())
        .ok_or(SureError::MultiplictationQ3232Overflow)?;
    Ok(num_frac_x64 >> 64)
}

/// Calculate
///
/// tick = 2*ln(sp_x32/2^32) / ln(1.0001)
///
pub fn get_tick_at_sqrt_ratio(sqrt_price_x64: u128) -> Result<i32> {
    if sqrt_price_x64 < MIN_SQRT_RATIO || sqrt_price_x64 > MAX_SQRT_RATIO {
        return Err(SureError::SqrtRatioNotWithinRange.into());
    }

    let mut r = sqrt_price_x64;
    let mut msb = 0;

    // Binary search method from 2^64, 2^32, 2^16,2^8,2^5
    let mut f: u8 = ((r >= 0x10000000000000000) as u8) << 6; // if r >= 2^64, f=64 else 0
    msb |= f; // add f to msb
    r >>= f;

    f = ((r >= 0x100000000) as u8) << 5; // If r >= 2^32, f = 32 else 0
    msb |= f; // Add f to MSB
    r >>= f; // Right shift by f

    f = ((r >= 0x10000) as u8) << 4; // 2^16
    msb |= f;
    r >>= f;

    f = ((r >= 0x100) as u8) << 3; // 2^8
    msb |= f;
    r >>= f;

    f = ((r >= 0x10) as u8) << 2; // 2^4
    msb |= f;
    r >>= f;

    f = ((r >= 0x4) as u8) << 1; // 2^2
    msb |= f;
    r >>= f;

    f = ((r >= 0x2) as u8) << 0; // 2^0
    msb |= f;

    // log2 (m x 2^e) = log2 (m) + e
    // For U64.64, e = -64. Subtract by 64 to remove x64 notation.
    // Then left shift by 16 bits to convert into U96.32 form
    let mut log_2_x32 = (msb as i128 - 64) << 32;

    // ------------------------------------------------------
    // Fractional part of logarithm

    // Set r = r / 2^n as a Q33.31 number, where n stands for msb
    r = if msb >= 64 {
        sqrt_price_x64 >> (msb - 63)
    } else {
        sqrt_price_x64 << (63 - msb)
    };

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 31; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 30; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 29; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 28; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 27; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 26; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 25; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 24; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 23; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 22; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 21; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 20; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 19; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 18; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 17; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 16; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 15; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    r = (r * r) >> 63; // r^2 as U33.31
    f = (r >> 64) as u8; // MSB of r^2 (0 or 1)
    log_2_x32 |= (f as i128) << 14; // Add f at 1st fractional place
    r >>= f; // Divide r by 2 if MSB of f is non-zero

    // 14 bit refinement gives an error margin of 2^-14 / log2 (√1.0001) = 0.8461 < 1
    // Since tick is a decimal, an error under 1 is acceptable

    // Change of base rule: multiply with 2^32 / log2 (√1.0001)
    let log_sqrt_10001_x64 = log_2_x32 * 59543866431248i128;

    // tick - 0.01
    let tick_low = ((log_sqrt_10001_x64 - 184467440737095516i128) >> 64) as i32;

    // tick + (2^-14 / log2(√1.0001)) + 0.01
    let tick_high = ((log_sqrt_10001_x64 + 15793534762490258745i128) >> 64) as i32;

    Ok(if tick_low == tick_high {
        tick_low
    } else if get_sqrt_ratio_at_tick(tick_high) <= sqrt_price_x64 {
        tick_high
    } else {
        tick_low
    })
}

#[cfg(test)]
mod tests {
    use std::{
        default,
        ops::{BitAnd, BitXor},
    };

    use super::*;

    #[test]
    fn test_get_tick_at_sqrt_price() {
        // @checkpoint tick index is off by one!
        let options = [
            ((4 as u128) << 64, 27726),
            ((5 as u128) << 64, 32189),
            ((6 as u128) << 64, 35835),
            ((7 as u128) << 64, 38919),
        ];
        // sqrt price of 36 as Q.64.64

        for (sqrt_price_x64, expected_tick) in options {
            let tick_index = get_tick_at_sqrt_ratio(sqrt_price_x64).unwrap();
            assert_eq!(
                tick_index,
                expected_tick + 1,
                "compare tick index and expected tick index"
            );
        }
    }

    #[test]
    fn test_convert_back_forth() {
        let (expected_sqrt_price, expected_tick_index) = (9078618265828848800676189u128, 262144i32);
        let tick_index = get_tick_at_sqrt_ratio(expected_sqrt_price).unwrap();
        assert_eq!(tick_index, expected_tick_index);
        // fails!
        let sqrt_price = get_sqrt_ratio_at_tick(expected_tick_index);
        println!("(6 as u128) << 64: {}", (6 as u128) << 64);
        assert_eq!(sqrt_price, expected_sqrt_price);

        // and back
        let sqrt_price_back = get_sqrt_ratio_at_tick(tick_index);
        assert_eq!(sqrt_price_back, expected_sqrt_price);
    }

    #[test]
    fn test_get_sqrt_price_at_tick() {
        let options = [
            (
                0i32,
                18446744073709551616u128,
                18446744073709551616u128,
                "0x0",
            ),
            (
                1i32,
                18447666387855959850u128,
                18445821805675392311u128,
                "0x1",
            ),
            (
                2i32,
                18448588748116922571u128,
                18444899583751176498u128,
                "0x2",
            ),
            (
                4i32,
                18450433606991734263u128,
                18443055278223354162u128,
                "0x4",
            ),
            (
                8i32,
                18454123878217468680u128,
                18439367220385604838u128,
                "0x8",
            ),
            (
                16i32,
                18461506635090006701u128,
                18431993317065449817u128,
                "0x10",
            ),
            (
                32i32,
                18476281010653910144u128,
                18417254355718160513u128,
                "0x20",
            ),
            (
                64i32,
                18505865242158250041u128,
                18387811781193591352u128,
                "0x40",
            ),
            (
                128i32,
                18565175891880433522u128,
                18329067761203520168u128,
                "0x80",
            ),
            (
                256i32,
                18684368066214940582u128,
                18212142134806087854u128,
                "0x100",
            ),
            (
                512i32,
                18925053041275764671u128,
                17980523815641551639u128,
                "0x200",
            ),
            (
                1024i32,
                19415764168677886926u128,
                17526086738831147013u128,
                "0x400",
            ),
            (
                2048i32,
                20435687552633177494u128,
                16651378430235024244u128,
                "0x800",
            ),
            (
                4096i32,
                22639080592224303007u128,
                15030750278693429944u128,
                "0x1000",
            ),
            (
                8192i32,
                27784196929998399742u128,
                12247334978882834399u128,
                "0x2000",
            ),
            (
                16384i32,
                41848122137994986128u128,
                8131365268884726200u128,
                "0x4000",
            ),
            (
                32768i32,
                94936283578220370716u128,
                3584323654723342297u128,
                "0x8000",
            ),
            (
                65536i32,
                488590176327622479860u128,
                696457651847595233u128,
                "0x10000",
            ),
            (
                131072i32,
                12941056668319229769860u128,
                26294789957452057u128,
                "0x20000",
            ),
            (
                262144i32,
                9078618265828848800676189u128,
                37481735321082u128,
                "0x40000",
            ),
        ];

        for (tick, exp_pos_sqrt_price, exp_neg_sqrt_price, frac_bit) in options {
            let pos_sqrt_price = get_sqrt_ratio_at_tick(tick);
            let neg_sqrt_price = get_sqrt_ratio_at_tick(-tick);
            println!("{}, {}", pos_sqrt_price, exp_pos_sqrt_price);
            assert_eq!(
                pos_sqrt_price, exp_pos_sqrt_price,
                "Assert positive tick equals expected value on binary fraction bit {}",
                frac_bit
            );
            assert_eq!(
                neg_sqrt_price, exp_neg_sqrt_price,
                "Assert negative tick equals expected value on binary fraction bit {}",
                frac_bit
            );
        }
    }

    #[test]
    pub fn test_calculate_yearly_premium() {
        const Q64_MASK: u128 = 0xFFFF_FFFF_FFFF_FFFF;
        pub struct ExpectedOutput {
            premium: u64,
        }
        #[derive()]
        pub struct Test<'a> {
            name: &'a str,
            sqrt_price_target: u128,
            sqrt_price_current: u128,
            amount: u128,
            expected_output: ExpectedOutput,
        };

        let test_data = [Test {
            name: "1. assume buying insurance. Sqrt price target < sqrt_price_current",
            sqrt_price_target: get_sqrt_ratio_at_tick(MIN_TICK_INDEX),
            sqrt_price_current: get_sqrt_ratio_at_tick(-40000),
            amount: 10_000,
            expected_output: ExpectedOutput { premium: 128 },
        }];

        for test in test_data {
            let premium = calculate_yearly_premium(
                test.sqrt_price_target,
                test.sqrt_price_current,
                test.amount,
            )
            .unwrap();
            assert_eq!(
                premium,
                test.expected_output.premium,
                "calculate_yearly_premium.{}.fail with \n
                sqrt_price_target: {} \n
                sqrt_price_current: {}
                ",
                test.name,
                test.sqrt_price_target.bitand(Q64_MASK),
                test.sqrt_price_current
            );
        }
    }
}
