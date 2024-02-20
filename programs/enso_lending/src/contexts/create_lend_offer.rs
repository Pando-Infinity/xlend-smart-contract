use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::{LendOfferAccount, LendOfferStatus, LendOfferError, CreateLendOfferEvent};

#[derive(Accounts)]
#[instruction(offer_id: String, amount: u64, interest: f64, lender_fee: u64, duration: u64)]
pub struct CreateLendOffer<'info> {
    #[account(mut)]
    pub lender: Signer<'info>,
    pub mint_asset: Account<'info, Mint>,
    #[account(
        mut,
        constraint = lender_ata_asset.amount >= amount @ LendOfferError::NotEnoughAmount,
        associated_token::mint = mint_asset,
        associated_token::authority = lender
    )]
    pub lender_ata_asset: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = lender,
        space = LendOfferAccount::INIT_SPACE,
        seeds = [b"enso".as_ref(), lender.key().as_ref(), offer_id.as_bytes()],
        bump
    )]
    pub lend_order: Account<'info, LendOfferAccount>,
    #[account(mut)]
    pub cw_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateLendOffer<'info> {
    pub fn initialize_lend_order(
        &mut self,
        bumps: &CreateLendOfferBumps,
        offer_id: String,
        amount: u64,
        interest: f64,
        lender_fee: u64,
        duration: u64
    ) -> Result<()> {
            self.lend_order.set_inner(LendOfferAccount {
                amount,
                duration,
                bump: bumps.lend_order,
                interest,
                lender_fee,
                lender_pubkey: self.lender.key(),
                loan_mint_token: self.mint_asset.key(),
                offer_id: offer_id.clone(),
                status: LendOfferStatus::Created,
            });

        Ok(())
    }

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        transfer_checked(
            self.into_deposit_context(),
            amount,
            self.mint_asset.decimals,
        )
    }

    fn into_deposit_context(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
        let cpi_accounts = TransferChecked {
            from: self.lender_ata_asset.to_account_info(),
            mint: self.mint_asset.to_account_info(),
            to: self.cw_vault.to_account_info(),
            authority: self.lender.to_account_info(),
        };
        CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
    }

    pub fn emit_event_create_lend_order(&mut self, label: String) -> Result<()> {
        emit!(CreateLendOfferEvent {
            lender: self.lender.key(),
            interest: self.lend_order.interest,
            lender_fee: self.lend_order.lender_fee,
            amount: self.lend_order.amount,
            duration: self.lend_order.duration,
            offer_id: self.lend_order.offer_id.clone(),
        });
        
        msg!(&label.clone());
        
        Ok(())
    }
}
