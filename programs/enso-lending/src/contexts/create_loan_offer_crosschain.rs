use std::str::FromStr;

use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use wormhole_anchor_sdk::wormhole;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::{
    common::{ENSO_SEED, LEND_OFFER_ACCOUNT_SEED, LOAN_OFFER_CROSSCHAIN_ACCOUNT_SEED, SETTING_ACCOUNT_SEED, SUI_USD_PRICE_FEED_ID, SUI_USD_SYMBOL, USDC_USD_PRICE_FEED_ID, WORMHOLE_SYSTEM_PUBKEY, MIN_BORROW_HEALTH_RATIO}, convert_to_usd_price, LendOfferAccount, LendOfferStatus, LoanOfferCrosschainAccount, LoanOfferCrosschainCreateRequestEvent, LoanOfferCrosschainError, SettingAccount, LoanOfferCrosschainStatus
};

#[derive(Accounts)]
#[instruction(
    tier_id: String,
    loan_offer_id: String, 
    lend_offer_id: String,
    vaa_hash: [u8; 32], 
)]
pub struct CreateLoanOfferCrosschain<'info> {
    #[account(
      mut,
      constraint = system_wormhole.key() == Pubkey::from_str(WORMHOLE_SYSTEM_PUBKEY).unwrap() @ LoanOfferCrosschainError::InvalidSystem
    )]
    pub system_wormhole: Signer<'info>,
    #[account( 
        constraint = lend_mint_asset.key() == setting_account.lend_mint_asset @ LoanOfferCrosschainError::InvalidLendMintAsset,
      )]
    pub lend_mint_asset: Account<'info, Mint>,
    #[account(
        init,
        payer = system_wormhole,
        space = 8 + LoanOfferCrosschainAccount::INIT_SPACE,
        seeds = [
          ENSO_SEED.as_ref(),
          LOAN_OFFER_CROSSCHAIN_ACCOUNT_SEED.as_ref(),
          borrower.key().as_ref(),
          loan_offer_id.as_bytes(),
          crate::ID.key().as_ref()
        ],
        bump
      )] 
    pub loan_offer: Account<'info, LoanOfferCrosschainAccount>,
    /// CHECK: This account is used to check the validate of lend offer account
    pub lender: UncheckedAccount<'info>,
    /// CHECK: This account is used to check the validate of loan offer account
    pub borrower: UncheckedAccount<'info>,
    #[account(
        mut,
        constraint = lend_offer.status == LendOfferStatus::Created @ LoanOfferCrosschainError::LendOfferIsNotAvailable,
        seeds = [
          ENSO_SEED.as_ref(), 
          LEND_OFFER_ACCOUNT_SEED.as_ref(), 
          lender.key().as_ref(), 
          lend_offer_id.as_bytes(),
          crate::ID.key().as_ref(), 
        ],
        bump = lend_offer.bump
      )]
      pub lend_offer: Account<'info, LendOfferAccount>,
      pub lend_price_feed_account: Account<'info, PriceUpdateV2>,
      pub collateral_price_feed_account: Account<'info, PriceUpdateV2>,
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
        seeds = [
            wormhole::SEED_PREFIX_POSTED_VAA,
            &vaa_hash
        ],
        bump,
        seeds::program = wormhole_program
    )]
      pub posted: Account<'info, wormhole::PostedVaaData>,
      pub wormhole_program: Program<'info, wormhole::program::Wormhole>,
      pub system_program: Program<'info, System>,
}

impl<'info> CreateLoanOfferCrosschain<'info> {
    pub fn create_loan_offer_crosschain(
        &mut self,
        bumps: &CreateLoanOfferCrosschainBumps,
        tier_id: String,
        loan_offer_id: String, 
        lend_offer_id: String, 
    ) -> Result<()> {
      let posted_vaa = &self.posted.clone().into_inner();
      let ( collateral_amount, collateral_token_symbol, collateral_token_decimal ) = self.get_data_from_vaa(posted_vaa.payload.clone()).unwrap();

      self.validate_initialize_loan_offer_crosshchain(
        collateral_amount,
        collateral_token_symbol.clone(),
        collateral_token_decimal,
      )?;

      self.lend_offer.status = LendOfferStatus::Loaned;
      self.loan_offer.set_inner(LoanOfferCrosschainAccount {
        tier_id,
        lend_offer_id,
        interest: self.lend_offer.interest,
        borrow_amount: self.lend_offer.amount,
        lender_fee_percent: self.lend_offer.lender_fee_percent,
        duration: self.lend_offer.duration,
        lender: self.lend_offer.lender,
        borrower: self.borrower.key(),
        loan_offer_id,
        collateral_amount,
        collateral_token_symbol,
        collateral_token_decimal,
        request_withdraw_amount: None,
        status: LoanOfferCrosschainStatus::Matched,
        borrower_fee_percent: self.setting_account.borrower_fee_percent,
        bump: bumps.loan_offer,
        lend_mint_token: self.lend_offer.lend_mint_token.key(),
        started_at: Clock::get()?.unix_timestamp,
        liquidating_at: None,
        liquidating_price: None,
        liquidated_tx: None,
        liquidated_price: None,
      });

      self.emit_event_create_loan_offer_crosschain()?;

      Ok(())
    }

    fn emit_event_create_loan_offer_crosschain(
      &self
    ) -> Result<()> {
      emit!(LoanOfferCrosschainCreateRequestEvent {
        tier_id: self.loan_offer.tier_id.clone(),
        lend_offer_id: self.loan_offer.lend_offer_id.clone(),
        interest: self.loan_offer.interest,
        borrow_amount: self.loan_offer.borrow_amount,
        lender_fee_percent: self.loan_offer.lender_fee_percent,
        duration: self.loan_offer.duration,
        lend_mint_token: self.loan_offer.lend_mint_token,
        lender: self.loan_offer.lender,
        loan_offer_id: self.loan_offer.loan_offer_id.clone(),
        borrower: self.loan_offer.borrower,
        collateral_amount: self.loan_offer.collateral_amount,
        collateral_token_symbol: self.loan_offer.collateral_token_symbol.clone(),
        collateral_token_decimal: self.loan_offer.collateral_token_decimal,
        status: self.loan_offer.status,
        borrower_fee_percent: self.loan_offer.borrower_fee_percent,
        started_at: self.loan_offer.started_at,
      });

      Ok(())
    }

    fn validate_initialize_loan_offer_crosshchain(
      &self,
      collateral_amount: u64,
      collateral_token_symbol: String,
      collateral_token_decimal: u8,
    ) -> Result<()> {
      let collateral_feed_id = self.get_feed_id_from_symbol(collateral_token_symbol).unwrap();
      let convert_collateral_amount_to_usd = convert_to_usd_price(
        &self.collateral_price_feed_account, 
        collateral_feed_id.as_str(),
        collateral_amount as f64 / 10f64.powf(collateral_token_decimal as f64)
      ).unwrap();
      let convert_lend_amount_to_usd = convert_to_usd_price(
        &self.lend_price_feed_account, 
        USDC_USD_PRICE_FEED_ID,
        self.setting_account.amount as f64 / 10f64.powf(self.lend_mint_asset.decimals as f64)
      ).unwrap();
      let health_ratio = convert_collateral_amount_to_usd / convert_lend_amount_to_usd;
  
      if health_ratio < MIN_BORROW_HEALTH_RATIO {
          return err!(LoanOfferCrosschainError::CanNotTakeALoanBecauseHealthRatioIsNotValid);
      }
  
      Ok(())
    }

    fn get_data_from_vaa(
      &self,
      posted_vaa: Vec<u8>,
    ) -> Result<(u64, String, u8)> {
      let message = String::from_utf8(posted_vaa).unwrap();
      let data: Vec<&str> = message.split(',').collect();
      let collateral_amount = data[3].parse::<u64>().unwrap();
      let collateral_token_symbol = data[4].to_string();
      let collateral_token_decimal = data[5].parse::<u8>().unwrap();

      Ok((collateral_amount, collateral_token_symbol, collateral_token_decimal))
    }

    fn get_feed_id_from_symbol(
      &self,
      collateral_token_symbol: String
    ) -> Result<String> {
      if collateral_token_symbol == SUI_USD_SYMBOL {
        return Ok(SUI_USD_PRICE_FEED_ID.to_string());
      } else {
        return err!(LoanOfferCrosschainError::InvalidCollateralTokenSymbol);
      }
    }

}