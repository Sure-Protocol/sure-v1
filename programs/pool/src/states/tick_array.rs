use anchor_lang::prelude::*;
use std::ops::BitXor;

use crate::helpers::tick::*;
use crate::pool::*;
use crate::utils::errors::SureError;
use crate::utils::uint::U256;
/// Tick Array
///
/// An array of bits holding information about
/// where liquidity is provided.
///
#[account(zero_copy)]
#[derive(Default)]
#[repr(packed)]
pub struct TickArray {
    /// Bump
    pub bump: u8, // 1 byte

    /// Word pos identifies where on the pool tick range
    /// the tick array is located
    /// Assuming the lowest price is -2**32 the MIN_TICK_INDEX
    /// is t_min = ln(-2**32)/ln(1.0001) < -221818
    /// and t_max = ln(2**32)/ln(1.0001) > 221818
    ///
    /// ex: with a tick space of 1 the
    /// MIN WORD POS = (-221 818)/1 >> 8 = -866
    /// also 255/1 >> 8 = 0 while 256/1 >> 8 = 1
    pub word_pos: i16, // 2 bytes

    /// Word is an array of 64*4 = 256 bits
    /// each bit indicates if a tick is initialized
    pub word: [u64; 4], // 8*4 = 32 bytes
}

pub struct NextBit {
    pub next: u8,

    pub initialized: bool,
}

pub struct Position {
    word_pos: i16,

    /// Position n the word where the tick flag is stored
    bit_pos: u8,
}

pub fn position(tick_by_spacing: i32) -> Position {
    Position {
        word_pos: (tick_by_spacing >> 8) as i16,
        bit_pos: (tick_by_spacing % 256) as u8,
    }
}

pub fn least_significant_bit(x: U256) -> u8 {
    assert!(x > U256::default());
    x.trailing_zeros() as u8
}

pub fn most_significant_bit(x: U256) -> u8 {
    assert!(x > U256::default());
    255 - x.leading_zeros() as u8
}

impl TickArray {
    pub const SPACE: usize = 1 + 2 + 2 + 32;

    /// Initialize a new tick array
    ///
    pub fn initialize(&mut self, pool: Pool, bump: u8, word_pos: i16) -> Result<()> {
        let max_word_pos = ((MAX_TICK_INDEX / pool.tick_spacing as i32) >> 8) as i16;
        let min_word_post = ((MIN_TICK_INDEX / pool.tick_spacing as i32) >> 8) as i16;
        if word_pos > max_word_pos {
            return Err(SureError::TooLargeWordPosition.into());
        }

        if word_pos < min_word_post {
            return Err(SureError::TooSmallWordPosition.into());
        }

        self.bump = bump;
        self.word_pos = word_pos;

        Ok(())
    }

    pub fn flip_bit(&mut self, tick: i32, tick_spacing: u16) -> Result<()> {
        let tick_ratio = tick / tick_spacing as i32;
        let current_position = position(tick_ratio);
        if current_position.word_pos != self.word_pos {
            return Err(SureError::InvalidTickArrayWord.into());
        }

        let mask = U256::from(1 as i16) << current_position.bit_pos;
        let word = U256(self.word);
        self.word = word.bitxor(mask).0;
        Ok(())
    }

    pub fn is_initialized(&self, tick: u16, tick_spacing: u16) -> bool {
        let tick_ratio = tick / tick_spacing;
        let Position { word_pos, bit_pos } = position(tick_ratio);

        let next_bit = self.next_initialized_tick(tick, tick_spacing, true);
        next_bit.next == bit_pos && next_bit.initialized
    }

    pub fn next_initialized_tick(&self, tick: u16, tick_spacing: u16, lte: bool) -> NextBit {
        let tick_ratio = tick / tick_spacing;

        if lte {
            let Position { word_pos, bit_pos } = position(tick_ratio);
            let word = U256::from(word_pos);
            // all the 1s at or to the right of the current bit_pos
            let mask =
                (U256::from(1 as i16) << bit_pos) - (1 as i16) + (U256::from(1 as i16) << bit_pos);
            let masked = word & mask;
            let initialized = masked != U256::default();

            // if there are no initialized ticks to the right of or at the current tick, return rightmost in the word
            let next = if initialized {
                most_significant_bit(masked)
            } else {
                0
            };

            NextBit { next, initialized }
        } else {
            let Position { word_pos, bit_pos } = position(tick_ratio - 1);
            let word = U256::from(word_pos);
            // all the 1s at or to the left of the bit_pos
            let mask = !((U256::from(1 as i16) << bit_pos) - (1 as i16));
            let masked = word & mask;
            let initialized = masked != U256::default();

            // if there are no initialized ticks to the left of the current tick, return leftmost in the word
            let next = if initialized {
                least_significant_bit(masked)
            } else {
                u8::MAX
            };

            NextBit { next, initialized }
        }
    }
}
