use anchor_lang::prelude::*;
use anchor_spl::token::{transfer_checked, Mint, Token, TokenAccount, TransferChecked};

use crate::{LendOrderAccount, LendOrderError, LendOrderStatus, NumberLendOrderCreate};

#[derive(Accounts)]
#[instruction(order_ids: Vec<String>, number_orders: NumberLendOrderCreate, amount: u64, interest: u64, lender_fee: u64)]
pub struct CreateLendOrder<'info> {
    #[account(mut)]
    pub lender: Signer<'info>,
    pub mint_asset: Account<'info, Mint>,
    #[account(
        mut,
        constraint = lender_ata_asset.amount >= amount,
        associated_token::mint = mint_asset,
        associated_token::authority = lender
    )]
    pub lender_ata_asset: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = lender,
        space = LendOrderAccount::INIT_SPACE,
        constraint = order_ids.len() >= 1,
        seeds = [b"enso".as_ref(), order_ids[0].as_bytes()],
        bump
    )]
    pub first_lend_order: Account<'info, LendOrderAccount>,
    #[account(
        init_if_needed,
        payer = lender,
        space = LendOrderAccount::INIT_SPACE,
        constraint = order_ids.len() == 2 && number_orders == NumberLendOrderCreate::Two || order_ids.len() > 2 && ( number_orders == NumberLendOrderCreate::Five || number_orders == NumberLendOrderCreate::Ten ),
        seeds = [b"enso".as_ref(), order_ids[1].as_bytes()],
        bump
    )]
    pub second_lend_order: Option<Account<'info, LendOrderAccount>>,
    pub cw_vault: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

impl<'info> CreateLendOrder<'info> {
    pub fn initialize_lend_order(
        &mut self,
        bumps: &CreateLendOrderBumps,
        order_ids: Vec<String>,
        number_orders: NumberLendOrderCreate,
        amount: u64,
        interest: u64,
        lender_fee: u64,
    ) -> Result<()> {
        match number_orders {
            NumberLendOrderCreate::One => self.first_lend_order.set_inner(LendOrderAccount {
                amount,
                bump: bumps.first_lend_order,
                interest,
                lender_fee,
                lender_pubkey: self.lender.key(),
                loan_mint_token: self.mint_asset.key(),
                order_id: order_ids[0].clone(),
                status: LendOrderStatus::Created,
            }),
            NumberLendOrderCreate::Two => {
                let _amount = amount.div_ceil(2);
                let _lender_fee = lender_fee.div_ceil(2);

                self.first_lend_order.set_inner(LendOrderAccount {
                    amount: _amount,
                    bump: bumps.first_lend_order,
                    interest,
                    lender_fee: _lender_fee,
                    lender_pubkey: self.lender.key(),
                    loan_mint_token: self.mint_asset.key(),
                    order_id: order_ids[0].clone(),
                    status: LendOrderStatus::Created,
                });

                match &mut self.second_lend_order {
                    Some( second_order) => second_order.set_inner(LendOrderAccount {
                        amount: _amount,
                        bump: bumps.first_lend_order,
                        interest,
                        lender_fee: _lender_fee,
                        lender_pubkey: self.lender.key(),
                        loan_mint_token: self.mint_asset.key(),
                        order_id: order_ids[0].clone(),
                        status: LendOrderStatus::Created,
                    }),
                    None => {
                        return err!(LendOrderError::SecondLendOrderNotProvide);
                    }
                }
            }
            _ => println!("test"),
        }

        Ok(())
    }

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
      transfer_checked(
        self.into_deposit_context(), 
        amount, 
        self.mint_asset.decimals
      )
    }

    fn into_deposit_context(&self) -> CpiContext<'_, '_, '_, 'info, TransferChecked<'info>> {
      let cpi_accounts = TransferChecked {
          from: self.lender.to_account_info(),
          mint: self.mint_asset.to_account_info(),
          to: self.cw_vault.to_account_info(),
          authority: self.lender.to_account_info(),
      };
      CpiContext::new(self.token_program.to_account_info(), cpi_accounts)
  }
}
