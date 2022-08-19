use std::ops::Mul;

use crate::{
    instructions::DIV_LN2_X64,
    utils::{convert_f64_q64, convert_q16_f16},
};

/// Calculates exp(x) = 2^(x * (1/ln(2)))
///
/// ### Arguments
/// * x: Q32.32
///
/// ### Returns
/// * exp(x): Q64.64
pub fn calculate_exp(x: u64, negative: bool) -> u128 {
    println!("calculate_exp");
    // Q32.32 x Q16.16 -> Q48.48 >> 32 => Q16.16
    let exponent = (x.mul(DIV_LN2_X64) >> 32) as u32;
    println!("exponent: {}", exponent);
    let exponent_f = convert_q16_f16(exponent);
    println!("exponentf: {}", exponent_f);
    let exponent_sign = if negative { -exponent_f } else { exponent_f };
    let exp_x = 2_f32.powf(exponent_sign) as f64;
    convert_f64_q64(exp_x)
}

#[cfg(test)]
pub mod test_exponential_distribution {
    use super::*;

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
