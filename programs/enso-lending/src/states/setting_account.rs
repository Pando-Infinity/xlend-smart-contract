pub use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Debug)]
pub struct SettingAccount {
  pub amount: f64,
  pub duration: u64,
  pub owner: Pubkey,
  pub receiver: Pubkey,
  pub lend_mint_asset: Pubkey,
  pub collateral_mint_asset: Pubkey,
  #[max_len(50)]
  pub tier_id: String,
  pub bump: u8
}