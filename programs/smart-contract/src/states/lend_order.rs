pub use anchor_lang::prelude::*;

use crate::LendOrderStatus;

#[account]
#[derive(InitSpace)]
pub struct LendOrderAccount {
  pub interest: f64,
  pub lender_fee: u64,
  #[max_len(50)]
  pub order_id: String,
  pub lender_pubkey: Pubkey,
  pub loan_mint_token: Pubkey,
  pub amount: u64,
  pub bump: u8,
  pub status: LendOrderStatus
}
