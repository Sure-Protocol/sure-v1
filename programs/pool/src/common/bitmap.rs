///! Basic implementation of a U256 bitmap for storing binary state
use super::uint::U256;
use std::ops::BitXor;

#[derive(Copy, Clone)]
pub struct C256(pub U256);
impl C256 {
    pub fn init() -> C256 {
        C256(U256::from(0 as i16))
    }

    pub fn flip_bit(&mut self, id: u8) {
        let mask = U256::from(1 as i16) << id;
        self.0 = self.0.bitxor(mask);
    }

    pub fn is_on(&self, id: u8) -> bool {
        let bit_on = self.0 & (U256::from(1 as i16) << id);
        bit_on != U256::default()
    }

    pub fn next_position(&self) -> u8 {
        let idx = 255 - self.0.leading_zeros() as u8;
        if idx == 255 {
            return 0;
        } else {
            return idx + 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_me() {
        let mut bm = C256::init();
        assert_eq!(bm.0, U256::from(0 as i16));

        // check some random position
        assert_eq!(bm.is_on(100), false);

        // Flip bit

        let next_pos = bm.next_position();
        println!(
            "leading zeros: {}, trailing: {}",
            bm.0.leading_zeros(),
            bm.0.trailing_zeros()
        );
        assert_eq!(next_pos, 0);
        bm.flip_bit(next_pos);
        assert_eq!(bm.is_on(next_pos), true);
        assert_eq!(bm.is_on(next_pos + 1), false);

        let new_pos = bm.next_position();
        println!("bm: {}", bm.0.leading_zeros());
        assert_eq!(new_pos, 1);

        bm.flip_bit(next_pos);
        assert_eq!(bm.is_on(next_pos), false);
    }
}
