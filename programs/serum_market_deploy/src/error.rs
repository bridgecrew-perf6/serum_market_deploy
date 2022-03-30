use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Coin and price mints shouldn't match")]
    MatchingMints,
    #[msg("Initialize market error")]
    InitializeMarketError,
    #[msg("Borrowing data from immutable account")]
    ImmutableDataBorrow,
    #[msg("No valid nonces were found")]
    NonceNotFound,
}
