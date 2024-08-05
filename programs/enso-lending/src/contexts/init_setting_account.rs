use std::str::FromStr;

pub use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

use crate::{InitSettingAccountEvent, SettingAccount, common::{constant::OPERATE_SYSTEM_PUBKEY, SettingAccountError, ENSO_SEED, SETTING_ACCOUNT_SEED}};

#[derive(Accounts)]
#[instruction(tier_id: String, amount: u64, duration: u64)]
pub struct InitSettingAccount<'info> {
  #[account(mut)]
  pub owner: Signer<'info>,
  /// CHECK: This is the account used to make a seeds to create ata account for transfer asset from lender to how wallet
  pub receiver: AccountInfo<'info>,
  pub lend_mint_asset: Account<'info, Mint>,
  pub collateral_mint_asset: Account<'info, Mint>,
  /// CHECK: This is the account used to convert lend asset price to USD price
  pub lend_price_feed_account: AccountInfo<'info>,
  /// CHECK: This is the account used to convert collateral asset price to USD price
  pub collateral_price_feed_account: AccountInfo<'info>,
  #[account(
    init,
    payer = owner,
    space = SettingAccount::INIT_SPACE,
    seeds = [
      ENSO_SEED.as_ref(), 
      SETTING_ACCOUNT_SEED.as_ref(),
      tier_id.as_bytes(), 
      crate::ID.key().as_ref(), 
    ],
    bump
  )]
  pub setting_account: Account<'info, SettingAccount>,
  pub system_program: Program<'info, System>,
}

impl<'info> InitSettingAccount<'info> {
    pub fn init_setting_account(&mut self, bumps: &InitSettingAccountBumps, tier_id: String, amount: u64, duration: u64, lender_fee_percent: f64, borrower_fee_percent: f64  ) -> Result<()> {
      if self.owner.key() != Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() {
        return err!(SettingAccountError::InvalidOwner)?;
      }

      self.setting_account.set_inner(SettingAccount {
        amount,
        duration,
        owner: self.owner.key(),
        receiver: self.receiver.key(),
        lend_mint_asset: self.lend_mint_asset.key(),
        collateral_mint_asset: self.collateral_mint_asset.key(),
        tier_id,
        bump: bumps.setting_account,
        lender_fee_percent,
        borrower_fee_percent,
        lend_price_feed: self.lend_price_feed_account.key(),
        collateral_price_feed: self.collateral_price_feed_account.key(),
      });

      msg!("Init Setting Account: {:?}", self.setting_account);

      Ok(())
    }

    pub fn emit_init_setting_account_event(&mut self, label: String) -> Result<()> {
      emit!(InitSettingAccountEvent {
          tier_id: self.setting_account.tier_id.clone(),
          amount: self.setting_account.amount,
          duration: self.setting_account.duration,
          collateral_mint_asset: self.setting_account.collateral_mint_asset,
          lend_mint_asset: self.setting_account.lend_mint_asset,
          owner: self.setting_account.owner,
          receiver: self.setting_account.receiver,
          lender_fee_percent: self.setting_account.lender_fee_percent,
          borrower_fee_percent: self.setting_account.borrower_fee_percent,
          collateral_price_feed: self.setting_account.collateral_price_feed,
          lend_price_feed: self.setting_account.lend_price_feed
      });
      
      msg!(&label.clone());
      
      Ok(())
  }
}