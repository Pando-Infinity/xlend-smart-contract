use anchor_lang::{prelude::*};
use anchor_spl::token::Mint;

use crate::{SettingAccount, EditSettingAccountEvent, SettingAccountError};

#[derive(Accounts)]
#[instruction(tier_id: String, amount: Option<f64>, duration: Option<u64>, lender_fee_percent: Option<f64>)]
pub struct EditSettingAccount<'info> {
  #[account(mut)]
  pub owner: Signer<'info>,
  /// CHECK: This is the account used to make a seeds to create ata account for transfer asset from lender to how wallet
  pub receiver: AccountInfo<'info>,
  pub lend_mint_asset: Account<'info, Mint>,
  pub collateral_mint_asset: Account<'info, Mint>,
  #[account(
    mut,
    has_one = owner,
    constraint = setting_account.tier_id == tier_id @ SettingAccountError::InvalidTierId,
    seeds = [
      b"enso".as_ref(), 
      tier_id.as_bytes(), 
      crate::ID.key().as_ref(), 
      owner.key().as_ref()
    ],
    bump
  )]
  pub setting_account: Account<'info, SettingAccount>,
  pub system_program: Program<'info, System>,
}

impl<'info> EditSettingAccount<'info> {
  pub fn edit_setting_account(
    &mut self, tier_id: String,
    amount: Option<f64>,
    duration: Option<u64>,
    lender_fee_percent: Option<f64>
  ) -> Result<()>  {
    let setting_account = &mut self.setting_account;
    setting_account.tier_id = tier_id;
    if let Some(amount) = amount {
      setting_account.amount = amount;
    }
    if let Some(duration) = duration {
      setting_account.duration = duration;
    }
    if let Some(lender_fee_percent) = lender_fee_percent {
      setting_account.lender_fee_percent = lender_fee_percent;
    }
    setting_account.receiver = self.receiver.key();
    setting_account.lend_mint_asset = self.lend_mint_asset.key();
    setting_account.collateral_mint_asset = self.collateral_mint_asset.key();
    Ok(())
  }

  pub fn emit_event_edit_setting_account(
    &mut self,
    label: String,
    tier_id: String,
    amount: Option<f64>,
    duration: Option<u64>,
    lender_fee_percent: Option<f64>
  ) -> Result<()> {
    emit!(EditSettingAccountEvent {
      receiver: self.receiver.key(),
      lend_mint_asset: self.lend_mint_asset.key(),
      collateral_mint_asset: self.collateral_mint_asset.key(),
      tier_id,
      amount,
      duration,
      lender_fee_percent
    });

    msg!(&label.clone());

    Ok(())
  }
}