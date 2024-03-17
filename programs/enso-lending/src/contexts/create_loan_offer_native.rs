use std::str::FromStr;

use anchor_lang::{prelude::*, solana_program::{program::invoke_signed, system_instruction}};

use crate::{
  common::{ENSO_SEED, LEND_OFFER_ACCOUNT_SEED, LOAN_OFFER_ACCOUNT_SEED, SETTING_ACCOUNT_SEED}, convert_to_usd_price, CreateLoanOfferEvent, LendOfferAccount, LendOfferStatus, LoanOfferAccount, LoanOfferError, LoanOfferStatus, SettingAccount, MIN_BORROW_HEALTH_RATIO, NATIVE_MINT
};

#[derive(Accounts)]
#[instruction(
  offer_id: String, 
  lend_offer_id: String, 
  tier_id: String, 
  collateral_amount: u64
)]
pub struct CreateLoanOfferNative<'info> {
  #[account(mut)]
  pub borrower: Signer<'info>,
  #[account(
    init_if_needed,
    payer = borrower,
    space = LoanOfferAccount::INIT_SPACE,
    seeds = [
      ENSO_SEED.as_ref(),
      LOAN_OFFER_ACCOUNT_SEED.as_ref(),
      borrower.key().as_ref(),
      offer_id.as_bytes(),
      crate::ID.key().as_ref()
    ],
    bump
  )]
  pub loan_offer: Account<'info, LoanOfferAccount>,
  /// CHECK: This account is used to check the validate of lend offer account
  pub lender: AccountInfo<'info>,
  #[account(
    mut,
    constraint = lend_offer.status == LendOfferStatus::Created @ LoanOfferError::LendOfferIsNotAvailable,
    seeds = [
      ENSO_SEED.as_ref(), 
      LEND_OFFER_ACCOUNT_SEED.as_ref(), 
      lender.key().as_ref(), 
      lend_offer_id.as_bytes(),
      crate::ID.key().as_ref(), 
    ],
    bump = lend_offer.bump
  )]
  pub lend_offer: Account<'info, LendOfferAccount>,
  /// CHECK: This is the account used to convert lend asset price to USD price
  pub lend_price_feed_account: AccountInfo<'info>,
  /// CHECK: This is the account used to convert collateral asset price to USD price
  pub collateral_price_feed_account: AccountInfo<'info>,
  #[account(
    seeds = [
        ENSO_SEED.as_ref(), 
        SETTING_ACCOUNT_SEED.as_ref(),
        tier_id.as_bytes(), 
        crate::ID.key().as_ref(), 
    ],
    bump = setting_account.bump
  )]
  pub setting_account: Account<'info, SettingAccount>,
  /// CHECK: This is the account used to receive the collateral amount
  pub receiver: AccountInfo<'info>,
  pub system_program: Program<'info, System>,
}

impl<'info> CreateLoanOfferNative<'info> {
  pub fn initialize_loan_offer(
    &mut self,
    bumps: &CreateLoanOfferNativeBumps,
    offer_id: String, 
    lend_offer_id: String, 
    tier_id: String, 
    collateral_amount: u64
  ) -> Result<()> {
    self.validate_initialize_loan_offer(collateral_amount)?;
    if self.receiver.key() != self.setting_account.receiver.key() {
      return Err(LoanOfferError::InvalidReceiver)?;
    }

    self.deposit_collateral(collateral_amount)?;

    self.lend_offer.status = LendOfferStatus::Loaned;
    self.loan_offer.set_inner(LoanOfferAccount {
      tier_id,
      borrow_amount: self.lend_offer.amount,
      borrower: self.borrower.key(),
      borrower_fee_percent: self.setting_account.borrower_fee_percent,
      bump: bumps.loan_offer,
      collateral_amount,
      collateral_mint_token: Pubkey::from_str(NATIVE_MINT).unwrap(),
      duration: self.lend_offer.duration,
      interest: self.lend_offer.interest,
      lend_mint_token: self.lend_offer.lend_mint_token.key(),
      lend_offer_id,
      lender: self.lend_offer.lender,
      lender_fee_percent: self.lend_offer.lender_fee_percent,
      offer_id,
      started_at: Clock::get()?.unix_timestamp,
      status: LoanOfferStatus::Matched,
      liquidating_at: None,
      liquidating_price: None,
      liquidated_tx: None,
      liquidated_price: None,
    });

    Ok(())
  }

  pub fn emit_event_create_loan_offer(&self, label: String) -> Result<()> {
    emit!(CreateLoanOfferEvent {
      tier_id: self.loan_offer.tier_id.clone(),
      lend_offer_id: self.loan_offer.lend_offer_id.clone(),
      interest: self.loan_offer.interest,
      borrow_amount: self.loan_offer.borrow_amount,
      lender_fee_percent: self.loan_offer.lender_fee_percent,
      duration: self.loan_offer.duration,
      lend_mint_token: self.loan_offer.lend_mint_token,
      lender: self.loan_offer.lender,
      offer_id: self.loan_offer.offer_id.clone(),
      borrower: self.loan_offer.borrower,
      collateral_mint_token: self.loan_offer.collateral_mint_token,
      collateral_amount: self.loan_offer.collateral_amount,
      status: self.loan_offer.status,
      borrower_fee_percent: self.loan_offer.borrower_fee_percent,
      started_at: self.loan_offer.started_at,
    });

    msg!(&label.clone());
    
    Ok(())
  }

  fn validate_initialize_loan_offer(&self, collateral_amount: u64) -> Result<()> {
    self.validate_price_feed_account()?;

    let convert_collateral_amount_to_usd = convert_to_usd_price(&self.collateral_price_feed_account.to_account_info(), collateral_amount).unwrap();
    let convert_lend_amount_to_usd = convert_to_usd_price(&self.lend_price_feed_account.to_account_info(), self.setting_account.amount).unwrap();
    let health_ratio = convert_collateral_amount_to_usd.checked_div(convert_lend_amount_to_usd).unwrap() as f64;

    if health_ratio < MIN_BORROW_HEALTH_RATIO {
        return Err(LoanOfferError::CanNotTakeALoanBecauseHealthRatioIsNotValid)?;
    }

    Ok(())
  }

  fn deposit_collateral(&self, collateral_amount: u64) -> Result<()> {
    let transfer_instruction = system_instruction::transfer(
      &self.borrower.key(), 
      &self.receiver.key(), 
      collateral_amount
    );
    
    invoke_signed(
      &transfer_instruction,
      &[
        self.borrower.to_account_info(),
        self.receiver.to_account_info(),          
        self.system_program.to_account_info()
      ],
      &[],  
    )?;

    Ok(())
  }

  fn validate_price_feed_account(&self) -> Result<()> {
    if self.setting_account.lend_price_feed.key() != self.lend_price_feed_account.key() {
      return Err(LoanOfferError::InvalidPriceFeedAccountForLendAsset)?;
    }

    if self.setting_account.collateral_price_feed.key() != self.collateral_price_feed_account.key() {
      return Err(LoanOfferError::InvalidPriceFeedAccountForCollateralAsset)?;
    }

    Ok(())
  }
}