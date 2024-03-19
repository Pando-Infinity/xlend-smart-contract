use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};
use crate::{
  common::{
    LendOfferError, LendOfferStatus
  }, states::lend_offer::LendOfferAccount, SettingAccount, LendOfferCanceledEvent, ENSO_SEED, LEND_OFFER_ACCOUNT_SEED, SETTING_ACCOUNT_SEED
};

#[derive(Accounts)]
#[instruction(
  offer_id: String, 
  tier_id: String
)]
pub struct SystemCancelLendOffer<'info> {
  /// CHECK: This account is used to check the validate of wallet receive back lend amount
  #[account(
    constraint = lender.key() == lend_offer.lender @ LendOfferError::InvalidLender
  )]
  pub lender: UncheckedAccount<'info>,
  #[account(
    mut,
    associated_token::mint = mint_asset,
    associated_token::authority = lender
  )]
  pub lender_ata_asset: Account<'info, TokenAccount>,
  #[account(
    constraint = mint_asset.key() == setting_account.lend_mint_asset @ LendOfferError::InvalidMintAsset,
  )]
  pub mint_asset: Account<'info, Mint>,
  #[account(
    mut,
    constraint = lend_offer.status == LendOfferStatus::Canceling @ LendOfferError::InvalidOfferStatus,
    seeds = [
      ENSO_SEED.as_ref(),
      LEND_OFFER_ACCOUNT_SEED.as_ref(),  
      lender.key().as_ref(), 
      offer_id.as_bytes(),
      crate::ID.key().as_ref(), 
    ],
    bump = lend_offer.bump
  )]
  pub lend_offer: Account<'info, LendOfferAccount>,
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
  #[account(mut)]
  pub hot_wallet: Signer<'info>,
  #[account(
    mut,
    associated_token::mint = mint_asset,
    associated_token::authority = hot_wallet
  )]
  pub hot_wallet_ata: Account<'info, TokenAccount>,
  pub token_program: Program<'info, Token>
}

impl<'info> SystemCancelLendOffer<'info> {
  pub fn system_cancel_lend_offer(&mut self, lend_amount: u64, waiting_interest: u64) -> Result<()>  {
    if lend_amount != self.lend_offer.amount {
      return Err(LendOfferError::InvalidLendAmount)?;
    }

    let total_repay = lend_amount + waiting_interest;

    if total_repay > self.hot_wallet_ata.amount {
      return Err(LendOfferError::NotEnoughAmount)?;
    }

    self.transfer_back_lend_asset(total_repay)?;

    self.lend_offer.status = LendOfferStatus::Canceled;

    self.emit_event_cancel_lend_offer(String::from("system_cancel_lend_offer"), total_repay)?;

    Ok(())
  }

  fn transfer_back_lend_asset(&mut self, total_repay: u64) -> Result<()> {
    transfer_checked(
        self.into_transfer_back_lend_asset_context(),
        total_repay,
        self.mint_asset.decimals,
    )
  }

  fn into_transfer_back_lend_asset_context(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
    let cpi_accounts = TransferChecked {
        from: self.hot_wallet_ata.to_account_info(),
        mint: self.mint_asset.to_account_info(),
        to: self.lender_ata_asset.to_account_info(),
        authority: self.hot_wallet.to_account_info(),
    };
    CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
  }

  fn emit_event_cancel_lend_offer(&mut self, label: String, total_repay: u64) -> Result<()> {
    emit!(LendOfferCanceledEvent {
      lender: self.lender.key(),
      amount: total_repay,
      duration: self.lend_offer.duration,
      interest: self.lend_offer.interest,
      lender_fee_percent: self.lend_offer.lender_fee_percent,
      offer_id: self.lend_offer.offer_id.clone()
    });

    msg!(&label.clone());

    Ok(())
  }
}