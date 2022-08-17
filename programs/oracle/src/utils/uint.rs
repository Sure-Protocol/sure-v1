///! Efficient macro to define a U256 rather than
///! using the internal u256.
use uint::construct_uint;
construct_uint! {
    pub struct U256(4);
}