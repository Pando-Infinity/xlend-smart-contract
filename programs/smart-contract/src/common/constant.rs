use anchor_lang::{error_code, prelude::{AnchorSerialize, AnchorDeserialize, borsh, InitSpace}};

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, InitSpace)]
pub enum LendOrderStatus {
    Created,
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq)]
pub enum NumberLendOrderCreate {
    One,
    Two,
    Five,
    Ten
}

#[error_code]
pub enum LendOrderError {
    #[msg("Second lend order not provided")]
    SecondLendOrderNotProvide
}