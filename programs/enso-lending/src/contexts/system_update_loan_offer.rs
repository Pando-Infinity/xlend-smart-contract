use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};
use crate::{
  common::LoanOfferError, LoanOfferAccount, LoanOfferStatus, LoanOfferUpdateEvent, ENSO_SEED, LOAN_OFFER_ACCOUNT_SEED
};

#[derive(Accounts)]
#[instruction(
  offer_id: String, 
  tier_id: String,
  borrow_amount: u64
)]
pub struct SystemUpdateLoanOffer<'info> {
  /// CHECK: This account is used to check the validate of wallet receive back lend amount
  #[account(
    constraint = borrower.key() == loan_offer.borrower @ LoanOfferError::InvalidBorrower
  )]
  pub borrower: AccountInfo<'info>,
  #[account(
    mut,
    associated_token::mint = mint_asset,
    associated_token::authority = borrower
  )]
  pub borrower_ata_asset: Account<'info, TokenAccount>,
  #[account(
    constraint = mint_asset.key() == loan_offer.lend_mint_token @ LoanOfferError::InvalidLendMintAsset,
  )]
  pub mint_asset: Account<'info, Mint>,
  #[account(
    mut,
    constraint = loan_offer.status == LoanOfferStatus::Matched @ LoanOfferError::InvalidLoanOffer,
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
  #[account(mut)]
  pub hot_wallet: Signer<'info>,
  #[account(
    mut,
    constraint = hot_wallet_ata.amount >= borrow_amount @ LoanOfferError::NotEnoughAmount,
    associated_token::mint = mint_asset,
    associated_token::authority = hot_wallet
  )]
  pub hot_wallet_ata: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>
}

impl<'info> SystemUpdateLoanOffer<'info> {
  pub fn system_update_loan_offer(&mut self, borrow_amount: u64) -> Result<()>  {
    if borrow_amount != self.loan_offer.borrow_amount {
      return err!(LoanOfferError::InvalidBorrowAmount)?;
    }

    self.transfer_lend_asset_to_borrower(borrow_amount)?;

    self.loan_offer.status = LoanOfferStatus::FundTransferred;

    self.emit_event_system_update_loan_offer(String::from("system_update_loan_offer"))?;

    Ok(())
  }

  fn transfer_lend_asset_to_borrower(&mut self, borrow_amount: u64) -> Result<()> {
    transfer_checked(
        self.into_transfer_lend_asset_to_borrower_context(),
        borrow_amount,
        self.mint_asset.decimals,
    )
  }

  fn into_transfer_lend_asset_to_borrower_context(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
    let cpi_accounts = TransferChecked {
        from: self.hot_wallet_ata.to_account_info(),
        mint: self.mint_asset.to_account_info(),
        to: self.borrower_ata_asset.to_account_info(),
        authority: self.hot_wallet.to_account_info(),
    };
    CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
  }

  fn emit_event_system_update_loan_offer(&mut self, label: String) -> Result<()> {
    emit!(LoanOfferUpdateEvent {
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
}