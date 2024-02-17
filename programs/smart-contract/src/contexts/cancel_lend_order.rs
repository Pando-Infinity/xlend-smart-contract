use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, CloseAccount};
use crate::utils::verify_account_empty;


#[derive(Accounts)]
#[instruction(order_id: String, amount: u64, interest: f64, lender_fee: u64)]
pub struct CancelLendOrder<'info> {
    #[account(mut)]
    pub lender: Signer<'info>,

    #[account(mut)]
    pub lend_order: AccountInfo<'info>,

    #[account(mut)]
    pub cw_vault: Account<'info, TokenAccount>,

    #[account(address = anchor_spl::token::ID)]
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> CancelLendOrder<'info> {
    fn close_context(&self) -> CpiContext<'_, '_, '_, 'info, CloseAccount<'info>> {
        CpiContext::new(
            self.token_program.clone(),
            CloseAccount {
                account: self.lend_order.to_account_info(),
                destination: self.cw_vault.to_account_info(),
                authority: self.lender.clone(),
            },
        )
    }
}


pub fn handler(ctx: Context<CancelLendOrder>, _bump: u8) -> ProgramResult {
    let lender = ctx.accounts.lender.load()?;
    let account = ctx.accounts.lend_order.key();

    // Verify if collateral is empty, then proceed
    verify_account_empty(&ctx.accounts.lend_order)?;

    token::close_account(
        ctx.accounts
            .close_context()
            .with_signer(&[&lender.authority_seeds()]),
    )?;

    msg!("closed collateral account");
    Ok(())
}

