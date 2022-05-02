/// Bitmap
/// 
#[account]
pub struct BitMap {
    /// Bump
    pub bump: u8, // 1 byte

    /// Map 
    pub map: [u64; 4], // 
}