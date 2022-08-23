use std::ops::{BitAnd, BitOr, Div, Mul};

/// Convert from Q64.64 -> Q64
///
/// TODO: move to common lib
pub fn convert_x64_to_u64(reward: u128, decimals: u8) -> u64 {
    let reward_f = convert_q64_to_f64(reward);
    reward_f.mul(10_u64.pow(decimals as u32) as f64).floor() as u64
}

pub fn convert_x32_to_u64(reward: u64, decimals: u8) -> u64 {
    let reward_f = convert_ix32_f64(reward as i64);
    reward_f.mul(10_u64.pow(decimals as u32) as f64).floor() as u64
}

pub fn convert_q64_to_f64(num: u128) -> f64 {
    let fractional_part = num.bitand(u64::MAX as u128) as u128;
    let integer_part = num.bitor(u64::MAX as u128) as u128;

    let fraction = (fractional_part as f64)
        .div(2_u64.pow(32) as f64)
        .div(2_u64.pow(32) as f64);
    let integer = integer_part >> 64;
    (integer as f64) + fraction
}
/// Convert a f64 to Q64.64
pub fn convert_f64_q64(float: f64) -> u128 {
    float
        .mul(2_u64.pow(32) as f64) // Q32.32
        .mul(2_u64.pow(32) as f64) //Q32.64
        .round() as u128
}

pub fn convert_f32_i64(float: f32) -> i64 {
    float
        .mul(2_u64.pow(32) as f32) // Q32.32
        .round() as i64
}

pub fn convert_f32_x16(float: f32) -> i32 {
    float
        .mul(2_u32.pow(16) as f32) // Q32.32
        .round() as i32
}

pub fn convert_q16_f16(num: u32) -> f32 {
    let fractional_part = num.bitand(u16::MAX as u32) as u32;
    let integer_part = num.bitor(u16::MAX as u32) as u32;

    let fraction = (fractional_part as f32).div(2_u32.pow(16) as f32);
    let integer = integer_part >> 16;
    (integer as f32) + fraction
}

pub fn convert_ix32_f64(num: i64) -> f64 {
    let positive = num > 0;
    let num_q32 = num.abs() as u64;

    let fractional_part = num_q32.bitand(u32::MAX as u64) as u64;
    let integer_part = num_q32.bitor(u32::MAX as u64) as u64;

    let fraction = (fractional_part as f64).div(2_u64.pow(32) as f64);
    let integer = integer_part >> 32;
    if positive {
        (integer as f64) + fraction
    } else {
        -((integer as f64) + fraction)
    }
}

#[cfg(test)]
pub mod test_convert_binary_representation {
    use crate::utils::calculate_exp;

    use super::*;

    #[test]
    pub fn test_convert_x64_to_u64() {
        pub struct Test {
            name: String,
            reward_f64: f64,
            // Q64.64
            reward_x64: u128,
            // Q32.0
            decimals: u8,
            expected_res: u64,
        }

        let tests = [
            Test {
                name: "1. test basic".to_string(),
                reward_f64: 10.4,
                reward_x64: 191846138366579343360, //10.4
                decimals: 6,
                expected_res: 10_400_000,
            },
            Test {
                name: "2. test with different decimal".to_string(),
                reward_f64: 10.4,
                reward_x64: 191846138366579343360, //10.4
                decimals: 12,
                expected_res: 10_400_000_000_000,
            },
            Test {
                name: "3. test with wild precision".to_string(),
                reward_f64: 10.4252435424325234235235324,
                reward_x64: 192311799533345931264, //10.4
                decimals: 12,
                expected_res: 10_425_243_542_432,
            },
        ];

        for test in tests {
            let res = convert_x64_to_u64(test.reward_x64, test.decimals);
            let reward = convert_f64_q64(test.reward_f64);
            println!("res: {}, reward_x64: {}", res, reward);
            assert_eq!(
                res, test.expected_res,
                "{}: test expected result",
                test.name
            );
        }
    }

    #[test]
    pub fn test_calculate_exp() {
        pub struct Test {
            name: String,
            x: u64,
            negative: bool,
            expected_res: u128,
        }

        let tests = [
            Test {
                name: "1. test exp(1) ".to_string(),
                x: 1,
                negative: false,
                expected_res: 50143205794491924480, // exp(1)
            },
            Test {
                name: "2. test exp(1) ".to_string(),
                x: 2804833801914,
                negative: false,
                expected_res: 50143205794491924480, // exp(1)
            },
        ];

        for test in tests {
            let res = calculate_exp(test.x, test.negative);
            assert_eq!(res, test.expected_res, "{}", test.name);
        }
    }
}
