pub use anchor_lang::prelude::*;

use crate::LendOfferStatus;

#[account]
#[derive(InitSpace)]
pub struct LendOfferAccount {
  pub interest: f64,
  pub lender_fee: u64,
  pub duration: u64,
  #[max_len(50)]
  pub offer_id: String,
  pub lender_pubkey: Pubkey,
  pub loan_mint_token: Pubkey,
  pub amount: u64,
  pub bump: u8,
  pub status: LendOfferStatus
}
