use anchor_lang::{
    event,
    prelude::{borsh, AnchorDeserialize, AnchorSerialize, Pubkey},
};

use crate::LoanOfferStatus;

#[event]
pub struct InitSettingAccountEvent {
    pub amount: u64,
    pub duration: u64,
    pub owner: Pubkey,
    pub receiver: Pubkey,
    pub lend_mint_asset: Pubkey,
    pub collateral_mint_asset: Pubkey,
    pub tier_id: String,
    pub lender_fee_percent: f64,
    pub lend_price_feed: Pubkey,
    pub collateral_price_feed: Pubkey,
}

#[event]
pub struct EditSettingAccountEvent {
    pub receiver: Pubkey,
    pub lend_mint_asset: Pubkey,
    pub collateral_mint_asset: Pubkey,
    pub tier_id: String,
    pub amount: u64,
    pub duration: u64,
    pub lender_fee_percent: f64,
    pub lend_price_feed: Pubkey,
    pub collateral_price_feed: Pubkey,
}

#[event]
pub struct CloseSettingAccountEvent {
    pub tier_id: String,
}

#[event]
pub struct CreateLendOfferEvent {
    pub lender: Pubkey,
    pub interest: f64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub amount: u64,
    pub offer_id: String,
    pub tier_id: String,
}

#[event]
pub struct EditLendOfferEvent {
    pub lender: Pubkey,
    pub interest: f64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub amount: u64,
    pub offer_id: String,
}

#[event]
pub struct LendOfferCancelRequestEvent {
    pub lender: Pubkey,
    pub interest: f64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub amount: u64,
    pub offer_id: String,
}

#[event]
pub struct LendOfferCanceledEvent {
    pub lender: Pubkey,
    pub interest: f64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub amount: u64,
    pub offer_id: String,
}

#[event]
pub struct LoanOfferCreateRequestEvent {
    pub tier_id: String,
    pub lend_offer_id: String,
    pub interest: f64,
    pub borrow_amount: u64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub lend_mint_token: Pubkey,
    pub lender: Pubkey,
    pub offer_id: String,
    pub borrower: Pubkey,
    pub collateral_mint_token: Pubkey,
    pub collateral_amount: u64,
    pub status: LoanOfferStatus,
    pub borrower_fee_percent: f64,
    pub started_at: i64,
}

#[event]
pub struct LoanOfferUpdateEvent {
    pub tier_id: String,
    pub lend_offer_id: String,
    pub interest: f64,
    pub borrow_amount: u64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub lend_mint_token: Pubkey,
    pub lender: Pubkey,
    pub offer_id: String,
    pub borrower: Pubkey,
    pub collateral_mint_token: Pubkey,
    pub collateral_amount: u64,
    pub status: LoanOfferStatus,
    pub borrower_fee_percent: f64,
    pub started_at: i64,
}

#[event]
pub struct WithdrawCollateralEvent {
    pub borrower: Pubkey,
    pub withdraw_amount: u64,
    pub loan_offer_id: String,
    pub remaining_amount: u64,
}

#[event]
pub struct DepositCollateralLoanOfferEvent {
    pub tier_id: String,
    pub lend_offer_id: String,
    pub interest: f64,
    pub borrow_amount: u64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub lend_mint_token: Pubkey,
    pub lender: Pubkey,
    pub offer_id: String,
    pub borrower: Pubkey,
    pub collateral_mint_token: Pubkey,
    pub collateral_amount: u64,
    pub status: LoanOfferStatus,
    pub borrower_fee_percent: f64,
    pub started_at: i64,
}

#[event]
pub struct RepayLoanOfferEvent {
  pub borrower: Pubkey,
  pub loan_offer_id: String,
  pub repay_amount: u64,
  pub borrower_fee_percent: f64,
   pub status: LoanOfferStatus,
}

#[event]
pub struct LiquidatingCollateralEvent {
    pub offer_id: String,
    pub liquidating_price: u64,
    pub liquidating_at: u64,
}

#[event]
pub struct LiquidatedCollateralEvent {
    pub offer_id: String,
    pub liquidated_price: u64,
    pub liquidated_tx: String,
}

#[event]
pub struct SystemRepayLoanOfferEvent {
    pub system: Pubkey,
    pub lender: Pubkey,
    pub borrower: Pubkey,
    pub total_repay_to_lender: u64,
    pub loan_amount: u64,
    pub collateral_amount: u64,
    pub waiting_interest: u64,
    pub loan_offer_id: String,
    pub tier_id: String,
    pub status: LoanOfferStatus,
}