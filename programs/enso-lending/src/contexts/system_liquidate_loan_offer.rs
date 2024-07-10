use std::str::FromStr;

use crate::{
    common::{
        constant::{LoanOfferStatus, OPERATE_SYSTEM_PUBKEY},
        LiquidateOfferError,
    },
    states::loan_offer::LoanOfferAccount,
    LiquidatedCollateralEvent, ENSO_SEED, LOAN_OFFER_ACCOUNT_SEED,
};
use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

#[derive(Accounts)]
#[instruction(loan_offer_id: String)]
pub struct SystemLiquidateLoanOffer<'info> {
  #[account(
    mut,
    constraint = system.key() == Pubkey::from_str(OPERATE_SYSTEM_PUBKEY).unwrap() @ LiquidateOfferError::InvalidSystem
  )]
  pub system: Signer<'info>,
  #[account(
    mut,
    associated_token::mint = mint_asset,
    associated_token::authority = system
  )]
  pub system_ata_asset: Account<'info, TokenAccount>,
  #[account(
    constraint = mint_asset.key() == loan_offer.lend_mint_token @ LiquidateOfferError::InvalidMintAsset,
  )]
  pub mint_asset: Account<'info, Mint>,
  /// CHECK: This account is used to transfer back collateral for borrower
  #[account(
    constraint = borrower.key() == loan_offer.borrower @ LiquidateOfferError::InvalidBorrower
  )]
  pub borrower: AccountInfo<'info>,
  #[account(
    mut,
    associated_token::mint = mint_asset,
    associated_token::authority = borrower
  )]
  pub borrower_ata_asset: Account<'info, TokenAccount>,
  #[account(
    mut,
    constraint = loan_offer.status == LoanOfferStatus::Liquidating @ LiquidateOfferError::InvalidOfferStatus,
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
  pub token_program: Program<'info, Token>,
}

impl<'info> SystemLiquidateLoanOffer<'info> {
  pub fn system_liquidate_loan_offer(
    &mut self,
    collateral_swapped_amount: u64,
    liquidated_price: u64,
    liquidated_tx: String,
  ) -> Result<()> {
    let loan_interest_percent = self.loan_offer.interest / 100.0;
    let borrower_fee_percent = self.loan_offer.borrower_fee_percent / 100.0;
    let time_borrowed = (self.loan_offer.duration as f64) / ((24 * 60 * 60 * 365) as f64);
    let interest_loan_amount =
        loan_interest_percent * self.loan_offer.borrow_amount as f64 * time_borrowed;

    let borrower_fee_amount =
        borrower_fee_percent * interest_loan_amount;

    let remaining_fund_to_borrower = (collateral_swapped_amount as f64
      - self.loan_offer.borrow_amount as f64
      - interest_loan_amount
      - borrower_fee_amount) as u64;

    if remaining_fund_to_borrower > 0 {
      self.transfer_asset_to_borrower(remaining_fund_to_borrower)?;
    }

    let loan_offer = &mut self.loan_offer;
    loan_offer.liquidated_price = Some(liquidated_price);
    loan_offer.liquidated_tx = Some(liquidated_tx);
    loan_offer.status = LoanOfferStatus::Liquidated;
    
    self.emit_event_system_liquidate_loan_offer(
      String::from("system_liquidate_loan_offer"),
      remaining_fund_to_borrower,
      collateral_swapped_amount,
    )?;
    Ok(())
  }
  fn transfer_asset_to_borrower(&mut self, remaining_fund_to_borrower: u64) -> Result<()> {
    self.process_transfer(
      remaining_fund_to_borrower,
      self.borrower_ata_asset.to_account_info(),
    )?;
    Ok(())
  }
  fn process_transfer(&mut self, amount: u64, to: AccountInfo<'info>) -> Result<()> {
    transfer_checked(
      self.into_transfer_context(to),
      amount,
      self.mint_asset.decimals,
    )
  }
  fn into_transfer_context(
    &self,
    to: AccountInfo<'info>,
  ) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
    let cpi_accounts = TransferChecked {
      from: self.system_ata_asset.to_account_info(),
      mint: self.mint_asset.to_account_info(),
      to,
      authority: self.system.to_account_info(),
    };
    CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
  }
  fn emit_event_system_liquidate_loan_offer(
    &mut self,
    label: String,
    remaining_fund_to_borrower: u64,
    collateral_swapped_amount: u64,
  ) -> Result<()> {
    emit!(LiquidatedCollateralEvent {
      system: self.system.key(),
      lender: self.loan_offer.lender.key(),
      borrower: self.borrower.key(),
      loan_offer_id: self.loan_offer.offer_id.clone(),
      collateral_swapped_amount,
      status: self.loan_offer.status,
      liquidated_price: self.loan_offer.liquidated_price.unwrap(),
      liquidated_tx: self.loan_offer.liquidated_tx.as_ref().unwrap().clone(),
      remaining_fund_to_borrower
    });
    msg!(&label.clone());
    Ok(())
  }
}
