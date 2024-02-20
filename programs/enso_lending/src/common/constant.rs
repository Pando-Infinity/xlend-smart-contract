use anchor_lang::{error_code, prelude::{borsh, AnchorDeserialize, AnchorSerialize, InitSpace, Pubkey}, event};

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, InitSpace)]
pub enum LendOfferStatus {
    Created,
}

#[error_code]
pub enum LendOfferError {
    #[msg("Lender does not have enough assets")]
    NotEnoughAmount
}

#[event]
// #[derive(Clone, Debug, Default)]
pub struct CreateLendOfferEvent {
    pub lender: Pubkey,
    pub interest: f64,
    pub lender_fee: u64,
    pub duration: u64,
    pub amount: u64,
    pub offer_id: String,
}