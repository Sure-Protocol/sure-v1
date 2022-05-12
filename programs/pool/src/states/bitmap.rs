use anchor_lang::prelude::*;
use std::io;
use std::ops::BitXor;

use crate::accounts;
use crate::utils::uint::U256;

/// Bitmap used to keep track of liquidity at each tick
///
#[account]
pub struct BitMap {
    /// Bump
    pub bump: u8, // 1 byte

    pub word_pos: i16, // 2 bytes

    pub spacing: u16, // 2 byts

    /// Map
    pub word: [u64; 4], // 8*4 = 32 bytes
}

#[derive(Accounts)]
pub struct UpdateBitmap<'info> {
    #[account(mut)]
    pub bitmap: Account<'info, BitMap>,
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

pub fn position(tick_ratio: u16) -> Position {
    Position {
        word_pos: (tick_ratio >> 8) as i16,
        bit_pos: (tick_ratio % 256) as u8,
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

impl BitMap {
    pub const SPACE: usize = 1 + 2 + 2 + 32;

    pub fn flip_bit(&mut self, tick: u16) {
        let tick_ratio = tick / self.spacing;
        let current_position = position(tick_ratio);
        let mask = U256::from(1 as i16) << current_position.bit_pos;
        let word = U256(self.word);
        self.word = word.bitxor(mask).0;
    }

    pub fn is_initialized(&self, tick: u16, tick_spacing: u16) -> bool {
        let tick_ratio = tick / tick_spacing;
        let Position { word_pos, bit_pos } = position(tick_ratio);

        let next_bit = self.next_initialized_tick(tick, true);
        next_bit.next == bit_pos && next_bit.initialized
    }

    pub fn next_initialized_tick(&self, tick: u16, lte: bool) -> NextBit {
        let tick_ratio = tick / self.spacing;

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
