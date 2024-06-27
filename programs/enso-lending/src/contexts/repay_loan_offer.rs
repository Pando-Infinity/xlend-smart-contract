use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::{
  LoanOfferAccount, 
  LoanOfferError, 
  LoanOfferStatus, 
  RepayOfferError, 
  SettingAccount, 
  SystemRepayLoadOfferNativeEvent, 
  VaultAuthority,
  ENSO_SEED, 
  LOAN_OFFER_ACCOUNT_SEED, 
  SETTING_ACCOUNT_SEED, 
  VAULT_AUTHORITY_LOAN_OFFER_ACCOUNT_SEED
};

#[derive(Accounts)]
#[instruction(loan_offer_id: String)]
pub struct RepayLoanOffer<'info> {
  #[account(mut)]
  pub borrower: Signer<'info>,
  #[account(
    constraint = lend_mint_asset.key() == setting_account.lend_mint_asset @ RepayOfferError::InvalidMintAsset
  )]
  pub lend_mint_asset: Account<'info, Mint>,
  #[account(
    constraint = collateral_mint_asset.key() == setting_account.collateral_mint_asset @ RepayOfferError::InvalidMintAsset
  )]
  pub collateral_mint_asset: Account<'info, Mint>,
  #[account(
    mut,
    associated_token::mint = lend_mint_asset,
    associated_token::authority = borrower
  )]
  pub borrower_loan_ata_asset: Account<'info, TokenAccount>,
  #[account(
    mut,
    associated_token::mint = collateral_mint_asset,
    associated_token::authority = borrower
  )]
  pub borrower_collateral_ata_asset: Account<'info, TokenAccount>,
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
    bump = loan_offer.bump
  )]
  pub loan_offer: Account<'info, LoanOfferAccount>,
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
    constraint = vault_authority.initializer.key() == borrower.key() @ LoanOfferError::InvalidInitializerVaultAuthority,
    seeds = [
      ENSO_SEED.as_ref(), 
      borrower.key().as_ref(),
      VAULT_AUTHORITY_LOAN_OFFER_ACCOUNT_SEED.as_ref(), 
      crate::ID.key().as_ref(), 
    ],
    bump = vault_authority.bump
  )]
  pub vault_authority: Box<Account<'info, VaultAuthority>>,
  #[account(
    mut,
    associated_token::mint = collateral_mint_asset,
    associated_token::authority = vault_authority
  )]
  pub vault: Box<Account<'info, TokenAccount>>,
  #[account(
    mut,
    associated_token::mint = lend_mint_asset,
    associated_token::authority = setting_account.receiver
  )]
  pub hot_wallet_ata: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>,
  pub system_program: Program<'info, System>,
} 

impl<'info> RepayLoanOffer<'info> {
    pub fn repay_loan_offer(&mut self) -> Result<()> {
      self.validate_loan_offer()?;

      let total_amount = self.calculated_repay_amount();

      if total_amount > self.borrower_loan_ata_asset.amount {
        return err!(RepayOfferError::NotEnoughAmount);
      }

      self.repay_loan_asset(total_amount)?;

      if self.vault.amount < self.loan_offer.collateral_amount {
        return err!(RepayOfferError::NotEnoughCollateralToRepay);
      }

      self.transfer_collateral_to_borrower()?;

      self.loan_offer.status = LoanOfferStatus::BorrowerPaid;

      self.emit_event_repay_loan_offer()
    }

    fn validate_loan_offer(&self) -> Result<()> {
      let current_timestamp = Clock::get().unwrap().unix_timestamp;
      let end_borrowed_loan_offer = self.loan_offer.started_at + self.loan_offer.duration as i64;

      if current_timestamp > end_borrowed_loan_offer {
        return err!(LoanOfferError::LoanOfferExpired);
      }

      Ok(())
    }

    fn repay_loan_asset(& self, repay_amount: u64) -> Result<()> {
      let cpi_ctx = CpiContext::new(
        self.token_program.to_account_info(), 
        TransferChecked {
          from: self.borrower_loan_ata_asset.to_account_info(),
          mint: self.lend_mint_asset.to_account_info(),
          to: self.hot_wallet_ata.to_account_info(),
          authority: self.borrower.to_account_info(),
        }
      );

      transfer_checked(
        cpi_ctx,
        repay_amount,
        self.lend_mint_asset.decimals,
      )
    }

    fn transfer_collateral_to_borrower(& self) -> Result<()> {
      let borrower_pub_key = self.borrower.key();
      let program_id = crate::ID.key();

      let signer: &[&[&[u8]]] = &[&[ 
        ENSO_SEED.as_ref(), 
        borrower_pub_key.as_ref(), 
        VAULT_AUTHORITY_LOAN_OFFER_ACCOUNT_SEED.as_ref(), 
        program_id.as_ref(), 
        &[self.vault_authority.bump] 
      ]];

      let cpi_ctx = CpiContext::new_with_signer(
        self.token_program.to_account_info(), 
        TransferChecked {
          from: self.vault.to_account_info(),
          mint: self.collateral_mint_asset.to_account_info(),
          to: self.borrower_collateral_ata_asset.to_account_info(),
          authority: self.vault_authority.to_account_info(),
        },
        signer
      );

      transfer_checked(
        cpi_ctx,
        self.loan_offer.collateral_amount,
        self.lend_mint_asset.decimals,
      )
    }

    fn calculated_repay_amount(&self) -> u64 {
      let borrower_fee_percent = self.setting_account.borrower_fee_percent / 100.0;
      let fee_amount = (self.loan_offer.borrow_amount as f64) * borrower_fee_percent;

      let loan_interest_percent = self.loan_offer.interest / 100.0;

      let time_borrowed = (self.loan_offer.duration as f64) / ((24 * 60 * 60 * 365) as f64);

      let interest_amount = (self.loan_offer.borrow_amount as f64) * loan_interest_percent * time_borrowed;

      let total_amount = (self.setting_account.amount as f64 + fee_amount + interest_amount) as u64;

      total_amount
    }

    fn emit_event_repay_loan_offer(&mut self) -> Result<()> {
      emit!(SystemRepayLoadOfferNativeEvent {
        lender: self.loan_offer.lender.key(),
        borrower: self.borrower.key(),
        interest: self.loan_offer.interest,
        loan_amount: self.loan_offer.collateral_amount,
        loan_offer_id: self.loan_offer.offer_id.clone(),
        tier_id: self.loan_offer.tier_id.clone(),
        collateral_amount: self.loan_offer.collateral_amount,
        status: self.loan_offer.status,
      });
      
      
      Ok(())
    }
}
