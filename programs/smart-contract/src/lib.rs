use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

mod contexts;
use contexts::*;
mod states;

declare_id!("CNkCiHgVyh6u1ifYb6YpK9bZAjD7oviJEsR5G1cMmLob");

#[derive(InitSpace)]
#[account]
pub struct Order {
    lender: Pubkey,
    mint: Pubkey,
    duration: i64,
    lend_amount: u64,
    interest: u64,
    status: Option<u8>,
    deadline: Option<i64>,
}

#[derive(Accounts)]
pub struct PlaceOrder<'info> {
    #[account(
        init_if_needed, 
        seeds=[b"order", signer.key().as_ref(), mint.key().as_ref()],
        bump,
        payer = signer, 
        space = Order::INIT_SPACE + 8,
    )]
    pub order: Account<'info, Order>,

    #[account(
        mut,
        constraint = lender_account.owner.key() == signer.key(),
        // constraint = lender_account.amount == amount, 
        constraint = mint.key() == lender_account.mint,
    )]
    pub lender_account: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = signer, 
        associated_token::mint = mint, 
        associated_token::authority = order,
    )]
    pub order_pda_token_account: Account<'info, TokenAccount>,

    pub mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account(mut)]
    pub signer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[program]
pub mod smart_contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn place_order(ctx: Context<PlaceOrder>, duration: i64, lend_amount: u64, interest: u64) -> Result<()> {
        let cpi_program = ctx.accounts.token_program.to_account_info();
        
        let transfer = Transfer {
            from: ctx.accounts.lender_account.to_account_info(),
            to: ctx.accounts.order_pda_token_account.to_account_info(),
            authority: ctx.accounts.signer.to_account_info(),
        };

        let token_transfer_context = CpiContext::new(
            cpi_program,
            transfer,
        );

        token::transfer(token_transfer_context, lend_amount)?;

        ctx.accounts.order.lender = ctx.accounts.signer.key();
        ctx.accounts.order.mint = ctx.accounts.mint.key();
        ctx.accounts.order.duration = duration;
        ctx.accounts.order.lend_amount = lend_amount;
        ctx.accounts.order.interest = interest;
        ctx.accounts.order.status = None;
        ctx.accounts.order.deadline = None;

        Ok(())
    }
}

