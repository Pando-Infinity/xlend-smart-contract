use std::ops::Add;

use anchor_lang::{prelude::*, solana_program::{program::invoke_signed, system_instruction}};
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::{
  common::{ENSO_SEED, LOAN_OFFER_ACCOUNT_SEED, SETTING_ACCOUNT_SEED}, DepositCollateralLoanOfferEvent, LoanOfferAccount, LoanOfferError, LoanOfferStatus, SettingAccount
};

#[derive(Accounts)]
#[instruction(
  offer_id: String, 
  tier_id: String,
  amount: u64
)]
pub struct DepositCollateralLoanOffer<'info> {
  #[account(mut)]
  pub borrower: Signer<'info>,
  #[account(
    constraint = mint_asset.key() == setting_account.collateral_mint_asset @ LoanOfferError::InvalidMintAsset,
  )]
  pub mint_asset: Account<'info, Mint>,
  #[account(
    mut,
    constraint = borrower_ata_asset.amount >= amount @ LoanOfferError::NotEnoughAmount,
    associated_token::mint = mint_asset,
    associated_token::authority = borrower
  )]
  pub borrower_ata_asset: Account<'info, TokenAccount>,
  #[account(
    mut,
    constraint = loan_offer.status == LoanOfferStatus::Matched @ LoanOfferError::CanNotDepositCollateralToContractThatNotAvailable,
    seeds = [
      ENSO_SEED.as_ref(),
      LOAN_OFFER_ACCOUNT_SEED.as_ref(),
      borrower.key().as_ref(),
      offer_id.as_bytes(),
      crate::ID.key().as_ref()
    ],
    bump = loan_offer.bump
  )]
  pub loan_offer: Account<'info, LoanOfferAccount>,
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
  #[account(
    mut,
    associated_token::mint = mint_asset,
    associated_token::authority = setting_account.receiver
  )]
  pub hot_wallet_ata: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
}

impl<'info> DepositCollateralLoanOffer<'info> {
  pub fn deposit_collateral_loan_offer(&mut self, amount: u64) -> Result<()> {
    self.deposit_collateral(amount)?;

    let before_collateral_amount = self.loan_offer.collateral_amount;
    self.loan_offer.collateral_amount = before_collateral_amount.add(amount);

    Ok(())
  }

  pub fn emit_event_deposit_collateral_loan_offer(&self, label: String) -> Result<()> {
    emit!(DepositCollateralLoanOfferEvent {
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

  fn deposit_collateral(&self, amount: u64) -> Result<()> {
     // transfer lamport
     let transfer_instruction = system_instruction::transfer(
      &self.borrower.key(), 
      &self.hot_wallet_ata.key(), 
      amount
    );
    
     invoke_signed(
       &transfer_instruction,
       &[
         self.borrower.to_account_info(),
         self.hot_wallet_ata.to_account_info(),          
         self.system_program.to_account_info()
       ],
       &[],  
     )?;
 
     Ok(())
  }
}