use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::{CreateLendOfferEvent, LendOfferAccount, LendOfferError, LendOfferStatus, SettingAccount};

#[derive(Accounts)]
#[instruction(offer_id: String, tier_id: String, interest: f64)]
pub struct CreateLendOffer<'info> {
    #[account(mut)]
    pub lender: Signer<'info>,
    #[account(
        constraint = mint_asset.key() == setting_account.lend_mint_asset @ LendOfferError::InvalidMintAsset,
    )]
    pub mint_asset: Account<'info, Mint>,
    #[account(
        mut,
        constraint = lender_ata_asset.amount >= setting_account.amount @ LendOfferError::NotEnoughAmount,
        associated_token::mint = mint_asset,
        associated_token::authority = lender
    )]
    pub lender_ata_asset: Account<'info, TokenAccount>,
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
        init_if_needed,
        payer = lender,
        space = LendOfferAccount::INIT_SPACE,
        seeds = [
            b"enso".as_ref(), 
            b"lend_offer".as_ref(), 
            lender.key().as_ref(), 
            offer_id.as_bytes(),
            crate::ID.key().as_ref(), 
        ],
        bump
    )]
    pub lend_offer: Account<'info, LendOfferAccount>,
    #[account(
        mut,
        associated_token::mint = mint_asset,
        associated_token::authority = setting_account.receiver
    )]
    pub hot_wallet_ata: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateLendOffer<'info> {
    pub fn initialize_lend_offer(
        &mut self,
        bumps: &CreateLendOfferBumps,
        offer_id: String,
        interest: f64,
    ) -> Result<()> {
            if interest <= (0 as f64) {
                return err!(LendOfferError::InterestGreaterThanZero);
            }

            let SettingAccount { amount, lender_fee_percent, duration, .. } = self.setting_account.clone().into_inner();

            self.lend_offer.set_inner(LendOfferAccount {
                amount,
                duration,
                bump: bumps.lend_offer,
                interest,
                lender_fee_percent,
                lender: self.lender.key(),
                lend_mint_token: self.mint_asset.key(),
                offer_id: offer_id.clone(),
                status: LendOfferStatus::Created,
            });

        Ok(())
    }

    pub fn deposit(&mut self) -> Result<()> {
        transfer_checked(
            self.into_deposit_context(),
            self.setting_account.amount,
            self.mint_asset.decimals,
        )
    }

    fn into_deposit_context(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        let cpi_accounts = TransferChecked {
            from: self.lender_ata_asset.to_account_info(),
            mint: self.mint_asset.to_account_info(),
            to: self.hot_wallet_ata.to_account_info(),
            authority: self.lender.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    pub fn emit_event_create_lend_offer(&mut self, label: String) -> Result<()> {
        emit!(CreateLendOfferEvent {
            lender: self.lender.key(),
            interest: self.lend_offer.interest,
            lender_fee_percent: self.lend_offer.lender_fee_percent,
            amount: self.lend_offer.amount,
            duration: self.lend_offer.duration,
            offer_id: self.lend_offer.offer_id.clone(),
            tier_id: self.setting_account.tier_id.clone(),
        });
        
        msg!(&label.clone());
        
        Ok(())
    }
}
