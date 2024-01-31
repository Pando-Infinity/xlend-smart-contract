use anchor_lang::{error_code, prelude::{borsh, AnchorDeserialize, AnchorSerialize, InitSpace, Pubkey}, event};

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, InitSpace)]
pub enum LendOrderStatus {
    Created,
}

#[error_code]
pub enum LendOrderError {
    #[msg("Lender does not have enough assets")]
    NotEnoughAmount
}

#[event]
// #[derive(Clone, Debug, Default)]
pub struct CreateLendOrderEvent {
    pub owner: Pubkey,
    pub interest: f64,
    pub lender_fee: u64,
    pub order_id: String,
    pub amount: u64
}