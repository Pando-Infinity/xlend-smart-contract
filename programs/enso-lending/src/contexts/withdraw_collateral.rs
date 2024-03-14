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
    LOAN_OFFER_ACCOUNT_SEED, constant::{MIN_BORROW_HEALTH_RATIO}, WithdrawCollateralEvent,
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

    let remaining_collateral = self.loan_offer.collateral_amount - withdraw_amount;

    let remaining_collateral_in_usd = convert_to_usd_price(&self.collateral_price_feed_account.to_account_info(), remaining_collateral).unwrap();

    let health_ratio = remaining_collateral_in_usd.checked_div(lend_amount_to_usd).unwrap() as f64;

    if health_ratio < MIN_BORROW_HEALTH_RATIO {
      return Err(LoanOfferError::HealthRatioLimit)?;
    }

    let current_timestamp = Clock::get().unwrap().unix_timestamp;
    let borrowed_timestamp = self.loan_offer.started_at;

    let duration_seconds = current_timestamp - borrowed_timestamp;

    if duration_seconds > (self.setting_account.duration as i64) * 60 * 60 * 24 {
      return Err(LoanOfferError::DurationLoanOfferInvalid)?;
    }

    self.loan_offer.collateral_amount = remaining_collateral;

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