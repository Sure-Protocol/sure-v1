use anchor_lang::prelude::Pubkey;
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy, Zeroable, Pod, PartialEq)]
#[repr(C)]
pub struct CallbackInfo {
    pk: Pubkey,
}

impl CallbackInfo {
    pub fn new(pk: Pubkey) -> Self {
        return Self { pk };
    }
}

impl agnostic_orderbook::state::orderbook::CallbackInfo for CallbackInfo {
    type CallbackId = Pubkey;

    fn as_callback_id(&self) -> &Self::CallbackId {
        return &self.pk;
    }
}
