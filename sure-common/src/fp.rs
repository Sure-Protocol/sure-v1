//! floating point arithmetic

use std::ops::Mul;

pub const POW32: f64 = 4294967296.0;

pub fn fp_from_float(n: f32) -> u64 {
    let n_f64 = n as f64;
    return n_f64.mul(POW32).floor() as u64;
}
