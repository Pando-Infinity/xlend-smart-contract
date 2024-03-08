use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::{LendOfferAccount, LendOfferStatus, LoanOfferAccount, LoanOfferError, SettingAccount};

#[derive(Accounts)]
#[instruction(
  offer_id: String, 
  lend_offer_id: String, 
  tier_id: String, 
  collateral_amount: u64
)]
pub struct CreateLoanOffer<'info> {
  #[account(mut)]
  pub borrower: Signer<'info>,
  #[account(
    constraint = mint_asset.key() == setting_account.collateral_mint_asset @ LoanOfferError::InvalidMintAsset
  )]
  pub mint_asset: Account<'info, Mint>,
  #[account(
    mut,
    constraint = borrower_ata_asset.amount >= collateral_amount,
    associated_token::mint = mint_asset,
    associated_token::authority = borrower
  )]
  pub borrower_ata_asset: Account<'info, TokenAccount>,
  #[account(
    init_if_needed,
    payer = borrower,
    space = LoanOfferAccount::INIT_SPACE,
    seeds = [
      b"enso".as_ref(),
      b"loan_offer".as_ref(),
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
      b"enso".as_ref(), 
      b"lend_offer".as_ref(), 
      lender.key().as_ref(), 
      lend_offer_id.as_bytes(),
      crate::ID.key().as_ref(), 
    ],
    bump = lend_offer.bump
  )]
  pub lend_offer: Account<'info, LendOfferAccount>,
  #[account(
    seeds = [
        b"enso".as_ref(), 
        b"setting_account".as_ref(),
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

impl<'info> CreateLoanOffer<'info> {
  pub fn initialize_loan_offer(
    &mut self,
    bumps: &CreateLoanOfferBumps,
    offer_id: String, 
    lend_offer_id: String, 
    tier_id: String, 
  ) -> Result<()> {
    // validate amount collateral >= minimum collateral
    // let minimum_collateral = ;

    Ok(())
  }
}