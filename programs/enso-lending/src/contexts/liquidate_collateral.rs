use std::str::FromStr;

use anchor_lang::prelude::*;

use crate::{common::constant::{ENSO_SEED, LOAN_OFFER_ACCOUNT_SEED, OPERATE_SYSTEM_PUBKEY}, LiquidatedCollateralEvent, LiquidatingCollateralEvent, LoanOfferAccount, LoanOfferError, LoanOfferStatus};

#[derive(Accounts)]
#[instruction(offer_id: String)]
pub struct LiquidateCollateral<'info> {
  #[account(mut)]
  pub system: Signer<'info>,
  /// CHECK: This is the account used to make a seeds
  pub borrower: AccountInfo<'info>,
  #[account(
    mut,
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
}

impl<'info> LiquidateCollateral<'info> {
  pub fn liquidating_collateral(&mut self, liquidating_price: u64, liquidating_at: u64) -> Result<()> {
    if self.system.key() != Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() {
      return Err(LoanOfferError::InvalidSystem)?;
    }

    let loan_offer = &mut self.loan_offer;
    if loan_offer.status != LoanOfferStatus::Matched {
      return Err(LoanOfferError::InvalidOfferStatus)?;
    }

    loan_offer.liquidating_price = Some(liquidating_price);
    loan_offer.liquidating_at = Some(liquidating_at);
    loan_offer.status = LoanOfferStatus::Liquidating;

    Ok(())
  }

  pub fn emit_event_liquidating_collateral(&self, label: String) -> Result<()> {
    emit!(LiquidatingCollateralEvent {
      offer_id: self.loan_offer.offer_id.clone(),
      liquidating_at: self.loan_offer.liquidating_at.unwrap(),
      liquidating_price: self.loan_offer.liquidating_price.unwrap(),
    });

    msg!(&label.clone());
    Ok(())
  }

  pub fn liquidated_collateral(&mut self, liquidated_price: u64, liquidated_tx: String) -> Result<()> {
    if self.system.key() != Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() {
      return Err(LoanOfferError::InvalidSystem)?;
    }

    let loan_offer = &mut self.loan_offer;
    if loan_offer.status != LoanOfferStatus::Liquidating {
      return Err(LoanOfferError::InvalidOfferStatus)?;
    }

    loan_offer.liquidated_price = Some(liquidated_price);
    loan_offer.liquidated_tx = Some(liquidated_tx);
    loan_offer.status = LoanOfferStatus::Liquidated;

    Ok(())
  }
  
  pub fn emit_event_liquidated_collateral(&self, label: String) -> Result<()> {
    emit!(LiquidatedCollateralEvent {
      offer_id: self.loan_offer.offer_id.clone(),
      liquidated_price: self.loan_offer.liquidated_price.unwrap(),
      liquidated_tx: self.loan_offer.liquidated_tx.as_ref().unwrap().clone(),
    });

    msg!(&label.clone());
    Ok(())
  }
}