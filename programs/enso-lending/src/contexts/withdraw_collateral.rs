use anchor_lang::prelude::*;
use anchor_spl::token::{Token};

use crate::{
  convert_to_usd_price,
  states::{
    loan_offer::LoanOfferAccount,
    setting_account::SettingAccount
  },
  LoanOfferStatus,
  LoanOfferError,
  common::{
    ENSO_SEED, SETTING_ACCOUNT_SEED,
    LOAN_OFFER_ACCOUNT_SEED, constant::{MIN_BORROW_HEALTH_RATIO, OFFER_DURATION_DAYS}, WithdrawCollateralEvent,
  }
};


#[derive(Accounts)]
#[instruction(loan_offer_id: String, withdraw_amount: u64)]
pub struct WithdrawCollateral<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,
    #[account(
      seeds = [
          ENSO_SEED.as_ref(), 
          SETTING_ACCOUNT_SEED.as_ref(),
          loan_offer.tier_id.as_bytes(), 
          crate::ID.key().as_ref(), 
      ],
      bump = setting_account.bump
    )]
    pub setting_account: Account<'info, SettingAccount>,
    #[account(
      mut,
      constraint = loan_offer.status == LoanOfferStatus::Matched @ LoanOfferError::LendOfferIsNotAvailable,
      seeds = [
        ENSO_SEED.as_ref(),
        LOAN_OFFER_ACCOUNT_SEED.as_ref(),
        borrower.key().as_ref(),
        loan_offer_id.as_bytes(),
        crate::ID.key().as_ref()
      ],
      bump = loan_offer.bump
    )]
    pub loan_offer: Account<'info, LoanOfferAccount>,
    /// CHECK: This is the account used to convert lend asset price to USD price
    pub lend_price_feed_account: AccountInfo<'info>,
    /// CHECK: This is the account used to convert collateral asset price to USD price
    pub collateral_price_feed_account: AccountInfo<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> WithdrawCollateral<'info> {
  pub fn withdraw_collateral(&mut self, withdraw_amount: u64) -> Result<()> {
    let lend_amount_to_usd = convert_to_usd_price(&self.lend_price_feed_account.to_account_info(), self.setting_account.amount).unwrap();

    let withdraw_amount_in_usd = convert_to_usd_price(&self.collateral_price_feed_account.to_account_info(), withdraw_amount).unwrap();
    let collateral_amount_in_usd = convert_to_usd_price(&self.collateral_price_feed_account.to_account_info(), self.loan_offer.collateral_amount).unwrap();

    let remaining_amount_in_usd = collateral_amount_in_usd - withdraw_amount_in_usd;

    let health_ratio = remaining_amount_in_usd.checked_div(lend_amount_to_usd).unwrap() as f64;

    if health_ratio < MIN_BORROW_HEALTH_RATIO {
      return Err(LoanOfferError::InsufficientBalance)?;
    }

    let current_timestamp = Clock::get().unwrap().unix_timestamp;
    let borrowed_timestamp = self.loan_offer.started_at;

    let duration_seconds = current_timestamp - borrowed_timestamp;
    let duration_days = duration_seconds / (60 * 60 * 24);

    if duration_days > OFFER_DURATION_DAYS as i64 {
      return Err(LoanOfferError::LendOfferIsNotAvailable)?;
    }

    let before_collateral_amount = self.loan_offer.collateral_amount;
    self.loan_offer.collateral_amount = before_collateral_amount - withdraw_amount;

    Ok(())
  }

  pub fn emit_event_withdraw_collateral(&mut self, label: String, loan_offer_id: String, withdraw_amount: u64) -> Result<()> {
    emit!(WithdrawCollateralEvent {
      borrower: self.borrower.key(),
      loan_offer_id,
      remaining_amount: self.loan_offer.collateral_amount,
      withdraw_amount,
    });

    msg!(&label.clone());

    Ok(())
  }
}