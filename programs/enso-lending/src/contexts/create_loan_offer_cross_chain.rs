use std::{fmt::format, str::FromStr};

use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use wormhole_anchor_sdk::wormhole;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::{
    common::{ENSO_SEED, LEND_OFFER_ACCOUNT_SEED, LOAN_OFFER_CROSS_CHAIN_ACCOUNT_SEED, MIN_BORROW_HEALTH_RATIO, SETTING_ACCOUNT_SEED, EMITTER_ADDRESSES, WORMHOLE_SUI_CHAIN_ID}, convert_to_usd_price, LendOfferAccount, LendOfferStatus, LoanOfferCrossChainAccount, LoanOfferCrossChainCreateRequestEvent, LoanOfferCrossChainError, LoanOfferCrossChainStatus, SettingAccount, WormholeError, WormholeMessage
};

#[derive(Accounts)]
#[instruction(
    tier_id: String,
    loan_offer_id: String, 
    lend_offer_id: String,
    vaa_hash: [u8; 32],
)]
pub struct CreateLoanOfferCrossChain<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account( 
        constraint = lend_mint_asset.key() == setting_account.lend_mint_asset @ LoanOfferCrossChainError::InvalidLendMintAsset,
      )]
    pub lend_mint_asset: Account<'info, Mint>,
    #[account(
        init,
        payer = signer,
        space = 8 + LoanOfferCrossChainAccount::INIT_SPACE,
        seeds = [
          ENSO_SEED.as_ref(),
          LOAN_OFFER_CROSS_CHAIN_ACCOUNT_SEED.as_ref(),
          borrower.key().as_ref(),
          lend_offer_id.as_bytes(),
          crate::ID.key().as_ref()
        ],
        bump
      )] 
    pub loan_offer: Account<'info, LoanOfferCrossChainAccount>,
    /// CHECK: This account is used to check the validate of lend offer account
    pub lender: UncheckedAccount<'info>,
    /// CHECK: This account is used to check the validate of loan offer account
    pub borrower: UncheckedAccount<'info>,
    #[account(
        mut,
        constraint = lend_offer.status == LendOfferStatus::Created @ LoanOfferCrossChainError::LendOfferIsNotAvailable,
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
    pub posted: Account<'info, wormhole::PostedVaa<WormholeMessage>>,
    pub wormhole_program: Program<'info, wormhole::program::Wormhole>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateLoanOfferCrossChain<'info> {
    pub fn create_loan_offer_cross_chain(
        &mut self,
        bumps: &CreateLoanOfferCrossChainBumps,
        tier_id: String,
        loan_offer_id: String, 
        lend_offer_id: String, 
    ) -> Result<()> {
      self.validate_vaa()?;

      let posted_vaa = &self.posted.clone().into_inner();
      let WormholeMessage::Message { payload } = posted_vaa.data();
      let ( collateral_amount, collateral_token_symbol, collateral_token_decimal ) = self.get_data_from_vaa(&payload).unwrap();

      self.lend_offer.status = LendOfferStatus::Loaned;
      self.loan_offer.set_inner(LoanOfferCrossChainAccount {
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
        status: LoanOfferCrossChainStatus::Matched,
        borrower_fee_percent: self.setting_account.borrower_fee_percent,
        bump: bumps.loan_offer,
        lend_mint_token: self.lend_offer.lend_mint_token.key(),
        started_at: Clock::get()?.unix_timestamp,
        liquidating_at: None,
        liquidating_price: None,
        liquidated_tx: None,
        liquidated_price: None,
      });

      self.emit_event_create_loan_offer_cross_chain()?;

      Ok(())
    }

    fn validate_vaa(
      &self,
    ) -> Result<()> {
      let emitter_chain = self.posted.meta.emitter_chain;
      let emitter_address ;
      if emitter_chain == WORMHOLE_SUI_CHAIN_ID {
        emitter_address = self.posted.meta.emitter_address.iter().map(|&c| {
          //have no idea
          if c < 16 {
            return format!("0{:x}", c);
          } else {
            return format!("{:x}", c);
          }
        }).collect::<String>();
      } else {
        emitter_address = String::new();
      }

      if !EMITTER_ADDRESSES.contains(&emitter_address.as_str()) {
        return err!(LoanOfferCrossChainError::InvalidVaa);
      }

      Ok(())
    }

    fn emit_event_create_loan_offer_cross_chain(
      &self,
    ) -> Result<()> {
      emit!(LoanOfferCrossChainCreateRequestEvent {
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

    fn get_data_from_vaa(
      &self,
      posted_vaa: &Vec<u8>,
    ) -> Result<(u64, String, u8)> {
      let message = String::from_utf8_lossy(posted_vaa).into_owned();
      let data: Vec<&str> = message.split(',').collect();
      msg!("Message received: {:?}", data);
      let collateral_amount = data[5].parse::<u64>().unwrap();
      let collateral_token_symbol = data[6].to_string();
      let collateral_token_decimal = data[7].parse::<u8>().unwrap();

      Ok((collateral_amount, collateral_token_symbol, collateral_token_decimal))
    }
}