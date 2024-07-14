use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::{
  common::{
    ENSO_SEED, LOAN_OFFER_ACCOUNT_SEED, SETTING_ACCOUNT_SEED
  }, duration_to_year, states::{
    loan_offer::LoanOfferAccount,
    setting_account::SettingAccount
  }, LoanOfferError, LoanOfferStatus, RepayOfferError, SystemRepayLoanOfferNativeEvent
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
      constraint = loan_offer.status == LoanOfferStatus::FundTransferred @ RepayOfferError::LoanOfferIsNotAvailable,
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
      self.validate_loan_offer()?;
      
      let total_amount = self.get_total_repay();

      if total_amount > self.loan_ata_asset.amount {
        return err!(RepayOfferError::NotEnoughAmount);
      }

      self.deposit(total_amount)?;

      self.loan_offer.sub_lamports(self.loan_offer.collateral_amount)?;
      self.borrower.add_lamports(self.loan_offer.collateral_amount)?;
      self.loan_offer.status = LoanOfferStatus::BorrowerPaid;

      self.emit_event_repay_loan_offer( "repay_loan_offer".to_string(), self.loan_offer.collateral_amount)?;
      
      Ok(())
    }

    pub fn deposit(&mut self, repay_amount: u64) -> Result<()> {
      let cpi_accounts = TransferChecked {
        from: self.loan_ata_asset.to_account_info(),
        mint: self.mint_asset.to_account_info(),
        to: self.hot_wallet_ata.to_account_info(),
        authority: self.borrower.to_account_info(),
      };
      
      let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);

      transfer_checked(
        cpi_ctx,
        repay_amount,
        self.mint_asset.decimals,
      )
    }

    fn validate_loan_offer(&self) -> Result<()> {
      let current_timestamp = Clock::get().unwrap().unix_timestamp;
      let end_borrowed_loan_offer = self.loan_offer.started_at + self.loan_offer.duration as i64;

      if current_timestamp > end_borrowed_loan_offer {
        return err!(LoanOfferError::LoanOfferExpired);
      }

      Ok(())
    }

    pub fn emit_event_repay_loan_offer(&mut self, label: String, collateral_amount: u64) -> Result<()> {
      emit!(SystemRepayLoanOfferNativeEvent {
        lender: self.loan_offer.lender.key(),
        borrower: self.borrower.key(),
        interest: self.loan_offer.interest,
        loan_amount: self.loan_offer.collateral_amount,
        loan_offer_id: self.loan_offer.offer_id.clone(),
        tier_id: self.loan_offer.tier_id.clone(),
        collateral_amount,
        status: self.loan_offer.status,
      });
      
      msg!(&label.clone());
      
      Ok(())
    }

    fn get_total_repay(&self) -> u64 {
      let borrower_fee_percent = self.setting_account.borrower_fee_percent / 100.0;

      let loan_interest_percent = self.loan_offer.interest / 100.0;

      let time_borrowed = duration_to_year(self.loan_offer.duration);

      let interest_amount = (self.loan_offer.borrow_amount as f64) * loan_interest_percent * time_borrowed;

      let borrower_fee_amount = borrower_fee_percent * interest_amount;
      
      return (self.setting_account.amount as f64 + interest_amount + borrower_fee_amount) as u64;
    }
}
