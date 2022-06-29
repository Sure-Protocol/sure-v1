use crate::utils::errors::SureError;
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
pub const MIN_TICK_INDEX: i32 = -221818;

/// max tick is calculated in the same way as with
/// the min tick, except for p=p_max=2**32
pub const MAX_TICK_INDEX: i32 = 221818;

/// MIN and MAX sqrt ratios
/// Calculated as
/// 1.0001**(tick/2)*(2**32)
pub const MIN_SQRT_RATIO: u64 = 65537;
pub const MAX_SQRT_RATIO: u64 = 281472331703918;

pub const BASE_FACTOR: f64 = 1.0001;

pub fn get_sqrt_ratio_at_tick(tick: i32) -> Result<u64> {
    let base2 = 2_f64;
    let factor = base2.powf(32.0);
    let sqrt_ratio = BASE_FACTOR.powf((tick / 2).into()) * factor;
    Ok(sqrt_ratio as u64)
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

    mod get_sqrt_ratio_at_tick {

        use super::*;

        #[test]
        #[should_panic]
        fn throws_for_too_low() {
            get_sqrt_ratio_at_tick(MIN_TICK_INDEX - 1).unwrap();
        }

        #[test]
        #[should_panic]
        fn throws_for_too_high() {
            get_sqrt_ratio_at_tick(MAX_TICK_INDEX + 1).unwrap();
        }

        #[test]
        fn MIN_TICK_INDEX() {
            assert_eq!(
                get_sqrt_ratio_at_tick(MIN_TICK_INDEX).unwrap(),
                MIN_SQRT_RATIO
            );
        }

        #[test]
        fn MIN_TICK_INDEX_plus_one() {
            assert_eq!(get_sqrt_ratio_at_tick(MIN_TICK_INDEX + 1).unwrap(), 65540);
        }

        #[test]
        fn MAX_TICK_INDEX() {
            assert_eq!(
                get_sqrt_ratio_at_tick(MAX_TICK_INDEX).unwrap(),
                MAX_SQRT_RATIO
            );
        }

        #[test]
        fn MAX_TICK_INDEX_minus_one() {
            assert_eq!(
                get_sqrt_ratio_at_tick(MAX_TICK_INDEX - 1).unwrap(),
                281458259142766
            );
        }

        #[test]
        fn is_at_most_off_by_a_bip() {
            // 1/100th of a bip condition holds for positive ticks
            let _abs_ticks: Vec<i32> = vec![
                50, 100, 250, 500, 1_000, 2_500, 3_000, 4_000, 5_000, 50_000, 150_000,
            ];

            for tick in MIN_TICK_INDEX..=MAX_TICK_INDEX {
                let result = get_sqrt_ratio_at_tick(tick).unwrap();
                let float_result = (f64::powi(1.0001, tick).sqrt() * u64::pow(2, 32) as f64) as u64;
                let abs_diff = if result > float_result {
                    result - float_result
                } else {
                    float_result - result
                };
                assert!((abs_diff as f64 / result as f64) < 0.0001);
            }
        }

        #[test]
        fn original_tick_can_be_retrieved_from_sqrt_ratio() {
            for tick in MIN_TICK_INDEX..=MAX_TICK_INDEX {
                let sqrt_price_x32 = get_sqrt_ratio_at_tick(tick).unwrap();
                if sqrt_price_x32 < MAX_SQRT_RATIO {
                    let obtained_tick = get_tick_at_sqrt_ratio(sqrt_price_x32).unwrap();
                    assert_eq!(tick, obtained_tick);
                }
            }
        }

        #[test]
        fn sqrt_price_increases_with_tick() {
            let mut prev_price_x32: u64 = 0;
            for tick in MIN_TICK_INDEX..=MAX_TICK_INDEX {
                let sqrt_price_x32 = get_sqrt_ratio_at_tick(tick).unwrap();
                // P should increase with tick
                if prev_price_x32 != 0 {
                    assert!(sqrt_price_x32 > prev_price_x32);
                }
                prev_price_x32 = sqrt_price_x32;
            }
        }
    }

    mod get_tick_at_sqrt_ratio {

        use super::*;

        #[test]
        #[should_panic(expected = "R")]
        fn throws_for_too_low() {
            get_tick_at_sqrt_ratio(MIN_SQRT_RATIO - 1).unwrap();
        }

        #[test]
        #[should_panic(expected = "R")]
        fn throws_for_too_high() {
            get_tick_at_sqrt_ratio(MAX_SQRT_RATIO).unwrap();
        }

        #[test]
        fn ratio_of_MIN_TICK_INDEX() {
            assert_eq!(
                get_tick_at_sqrt_ratio(MIN_SQRT_RATIO).unwrap(),
                MIN_TICK_INDEX
            );
        }

        #[test]
        fn ratio_of_MIN_TICK_INDEX_plus_one() {
            assert_eq!(get_tick_at_sqrt_ratio(65540).unwrap(), MIN_TICK_INDEX + 1);
        }

        #[test]
        fn ratio_of_MAX_TICK_INDEX_minus_one() {
            assert_eq!(
                get_tick_at_sqrt_ratio(281458259142766).unwrap(),
                MAX_TICK_INDEX - 1
            );
        }

        #[test]
        fn ratio_closest_to_MAX_TICK_INDEX() {
            assert_eq!(
                get_tick_at_sqrt_ratio(MAX_SQRT_RATIO - 1).unwrap(),
                MAX_TICK_INDEX - 1
            );
        }
    }
}
