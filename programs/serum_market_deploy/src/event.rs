use anchor_lang::prelude::*;

#[event]
pub struct InitializedEvent {
    pub coin_mint: Pubkey,
    pub price_mint: Pubkey,
}
