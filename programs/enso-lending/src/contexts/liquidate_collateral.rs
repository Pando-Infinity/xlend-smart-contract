use std::str::FromStr;

use anchor_lang::prelude::*;

use crate::{
  common::constant::{ENSO_SEED, LOAN_OFFER_ACCOUNT_SEED, OPERATE_SYSTEM_PUBKEY}, 
  LiquidatingCollateralEvent, LoanOfferAccount, LoanOfferError, LoanOfferStatus, HOT_WALLET_PUBKEY
};

#[derive(Accounts)]
#[instruction(offer_id: String)]
pub struct LiquidateCollateral<'info> {
  #[account(mut)]
  pub system: Signer<'info>,
  /// CHECK: This is the account used to make a seeds
  pub borrower: UncheckedAccount<'info>,
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
  #[account(mut)]
  /// CHECK: This is the account used to received the collateral for liquidate 
  pub hot_wallet: UncheckedAccount<'info>
}

impl<'info> LiquidateCollateral<'info> {
  pub fn start_liquidate_contract(&mut self, liquidating_price: u64, liquidating_at: u64) -> Result<()> {
    if self.system.key() != Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() {
      return err!(LoanOfferError::InvalidSystem);
    }

    if self.hot_wallet.key() != Pubkey::from_str(HOT_WALLET_PUBKEY).unwrap() {
      return err!(LoanOfferError::InvalidHotWallet);
    }

    let loan_offer = &mut self.loan_offer;
    if loan_offer.status != LoanOfferStatus::FundTransferred {
      return err!(LoanOfferError::InvalidOfferStatus);
    }

    loan_offer.liquidating_price = Some(liquidating_price);
    loan_offer.liquidating_at = Some(liquidating_at);
    loan_offer.status = LoanOfferStatus::Liquidating;

    self.loan_offer.sub_lamports(self.loan_offer.collateral_amount)?;
    self.hot_wallet.add_lamports(self.loan_offer.collateral_amount)?;

    Ok(())
  }

  pub fn emit_event_start_liquidate_contract(&self, label: String) -> Result<()> {
    emit!(LiquidatingCollateralEvent {
      offer_id: self.loan_offer.offer_id.clone(),
      liquidating_at: self.loan_offer.liquidating_at.unwrap(),
      liquidating_price: self.loan_offer.liquidating_price.unwrap(),
    });

    msg!(&label.clone());
    Ok(())
  }
}