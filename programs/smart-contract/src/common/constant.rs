use anchor_lang::{error_code, prelude::{borsh, AnchorDeserialize, AnchorSerialize, InitSpace}};

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, InitSpace)]
pub enum LendOrderStatus {
    Created,
}

#[error_code]
pub enum LendOrderError {
    #[msg("Lender does not have enough assets")]
    NotEnoughAmount
}