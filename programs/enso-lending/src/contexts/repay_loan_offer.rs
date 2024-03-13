use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::{
  states::{
    loan_offer::LoanOfferAccount,
    setting_account::SettingAccount
  }, 
  RepayOfferError,
  LoanOfferStatus,
  common::{
    ENSO_SEED, SETTING_ACCOUNT_SEED,
    LOAN_OFFER_ACCOUNT_SEED, RepayLoanOfferEvent,
  }
};


#[derive(Accounts)]
#[instruction(loan_offer_id: String)]
pub struct RepayLoanOffer<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,
    #[account(
        constraint = mint_asset.key() == setting_account.lend_mint_asset @ RepayOfferError::InvalidMintAsset,
    )]
    pub mint_asset: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = mint_asset,
        associated_token::authority = borrower
    )]
    pub loan_ata_asset: Account<'info, TokenAccount>,
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
      constraint = loan_offer.status == LoanOfferStatus::Matched @ RepayOfferError::LoanOfferIsNotAvailable,
      seeds = [
        ENSO_SEED.as_ref(),
        LOAN_OFFER_ACCOUNT_SEED.as_ref(),
        borrower.key().as_ref(),
        loan_offer_id.as_bytes(),
        crate::ID.key().as_ref()
      ],
      bump
    )]
    pub loan_offer: Account<'info, LoanOfferAccount>,
    #[account(
        mut,
        associated_token::mint = mint_asset,
        associated_token::authority = setting_account.receiver
    )]
    pub hot_wallet_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> RepayLoanOffer<'info> {
    pub fn repay_loan_offer(&mut self) -> Result<()> {
      let fee_amount = (self.loan_offer.borrow_amount as f64) * self.setting_account.borrower_fee_percent;
      let interest_amount = (self.loan_offer.borrow_amount as f64) * self.loan_offer.interest;

      let total_amount = (self.setting_account.amount as f64) + fee_amount + interest_amount;

      if total_amount > self.loan_ata_asset.amount as f64 {
        return Err(RepayOfferError::NotEnoughAmount.into());
      }

      self.deposit(total_amount as u64)?;
      self.loan_offer.status = LoanOfferStatus::Finished;

      self.emit_event_repay_loan_offer( "repay_loan_offer".to_string(), self.loan_offer.offer_id.clone(), total_amount as u64)?;
      
      Ok(())
    }

    pub fn deposit(&mut self, repay_amount: u64) -> Result<()> {
      transfer_checked(
        self.into_deposit_context(),
        repay_amount,
        self.mint_asset.decimals,
      )
    }

    fn into_deposit_context(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
      let cpi_accounts = TransferChecked {
        from: self.loan_ata_asset.to_account_info(),
        mint: self.mint_asset.to_account_info(),
        to: self.hot_wallet_ata.to_account_info(),
        authority: self.borrower.to_account_info(),
      };
      CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    pub fn emit_event_repay_loan_offer(&mut self, label: String, loan_offer_id: String, repay_amount: u64) -> Result<()> {
      emit!(RepayLoanOfferEvent {
        borrower: self.borrower.key(),
        loan_offer_id,
        repay_amount,
        borrower_fee_percent: self.setting_account.borrower_fee_percent,
      });
      
      msg!(&label.clone());
      
      Ok(())
    }
}
