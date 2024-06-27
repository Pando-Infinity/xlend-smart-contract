pub use anchor_lang::prelude::*;
use crate::LoanOfferCrossChainStatus;

#[account]
#[derive(InitSpace, Debug)]
pub struct LoanOfferCrossChainAccount {
    #[max_len(50)]
    pub tier_id: String,
    #[max_len(50)]
    pub lend_offer_id: String,
    pub interest: f64,
    pub borrow_amount: u64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub lender: Pubkey,
    #[max_len(50)]
    pub borrower: Pubkey,
    #[max_len(50)]
    pub loan_offer_id: String,
    pub collateral_amount: u64,
    #[max_len(50)]
    pub collateral_token_symbol: String,
    pub collateral_token_decimal: u8,
    pub lend_mint_token: Pubkey,
    pub request_withdraw_amount: Option<u64>,
    pub status: LoanOfferCrossChainStatus,
    pub borrower_fee_percent: f64,
    pub started_at: i64,
    pub liquidating_at: Option<u64>,
    pub liquidating_price: Option<u64>,
    #[max_len(50)]
    pub liquidated_tx: Option<String>,
    pub liquidated_price: Option<u64>,
    pub bump: u8,
}