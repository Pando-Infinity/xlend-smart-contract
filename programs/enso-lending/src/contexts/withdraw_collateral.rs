use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

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
    LOAN_OFFER_ACCOUNT_SEED, constant::MIN_BORROW_HEALTH_RATIO, WithdrawCollateralEvent,
  }
};

#[derive(Accounts)]
#[instruction(loan_offer_id: String, withdraw_amount: u64)]
pub struct WithdrawCollateral<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,
    #[account(
      constraint = collateral_mint_asset.key() == setting_account.collateral_mint_asset @ LoanOfferError::InvalidCollateralMintAsset,
    )]
    pub collateral_mint_asset: Account<'info, Mint>,
    #[account(
      constraint = lend_mint_asset.key() == setting_account.lend_mint_asset @ LoanOfferError::InvalidLendMintAsset,
    )]
    pub lend_mint_asset: Account<'info, Mint>,
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
      constraint = loan_offer.status == LoanOfferStatus::FundTransferred @ LoanOfferError::NotAvailableToWithdraw,
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
  pub fn withdraw_collateral(&mut self, loan_offer_id: String, withdraw_amount: u64) -> Result<()> {
    let lend_amount_to_usd = convert_to_usd_price(
      &self.lend_price_feed_account.to_account_info(), 
      self.setting_account.amount as f64 / 10f64.powf(self.lend_mint_asset.decimals as f64)
    ).unwrap();

    let remaining_collateral = self.loan_offer.collateral_amount - withdraw_amount;

    let remaining_collateral_in_usd = convert_to_usd_price(
      &self.collateral_price_feed_account.to_account_info(), 
      remaining_collateral as f64 / 10f64.powf(self.collateral_mint_asset.decimals as f64)
    ).unwrap();

    let health_ratio = remaining_collateral_in_usd / lend_amount_to_usd;

    if health_ratio < MIN_BORROW_HEALTH_RATIO {
      return Err(LoanOfferError::HealthRatioLimit)?;
    }

    let current_timestamp = Clock::get().unwrap().unix_timestamp;
    let end_borrowed_loan_offer = self.loan_offer.started_at + self.loan_offer.duration as i64;

    if current_timestamp > end_borrowed_loan_offer {
      return Err(LoanOfferError::LoanOfferExpired)?;
    }

    self.loan_offer.request_withdraw_amount = Some(withdraw_amount);

    self.emit_event_withdraw_collateral(
      String::from("withdraw_collateral"),
      loan_offer_id,
      withdraw_amount,
    )?;

    Ok(())
  }

  fn emit_event_withdraw_collateral(&mut self, label: String, loan_offer_id: String, withdraw_amount: u64) -> Result<()> {
    emit!(WithdrawCollateralEvent {
      borrower: self.borrower.key(),
      loan_offer_id,
      collateral_amount: self.loan_offer.collateral_amount,
      withdraw_amount,
    });

    msg!(&label.clone());

    Ok(())
  }
}