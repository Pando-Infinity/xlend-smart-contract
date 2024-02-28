use anchor_lang::{prelude::*};
use crate::common::constant::EditLendOfferEvent;
use crate::{LendOrderAccount, LendOrderStatus, LendOrderError};

#[derive(Accounts)]
#[instruction(order_id: String, interest: f64)]
pub struct EditLendOffer<'info> {
  #[account(mut)]
  pub lender: Signer<'info>,
  #[account(
    mut,
    has_one = lender,
    constraint = lend_order.status == LendOrderStatus::Created @ LendOrderError::InvalidOrderStatus,
    constraint = lend_order.order_id == order_id @ LendOrderError::InvalidOrderId,
    seeds = [b"enso".as_ref(), lender.key().as_ref(), order_id.as_bytes()],
    bump
  )]
  pub lend_order: Account<'info, LendOrderAccount>,
  /// CHECK: This is not dangerous because we don't read or write from this account
  pub token_program: AccountInfo<'info>,
  pub system_program: Program<'info, System>,
}

impl<'info> EditLendOffer<'info> {
  pub fn edit_lend_offer(&mut self, interest: f64) -> Result<()>  {
    let lend_order = &mut self.lend_order;
    require!(interest > 0.0, LendOrderError::InvalidInterestRate);
    lend_order.interest = interest;
    Ok(())
  }

  pub fn emit_event_edit_lend_offer(&mut self, label: String, order_id: String) -> Result<()> {
    emit!(EditLendOfferEvent {
        lender: self.lender.key(),
        order_id,
        interest: self.lend_order.interest
    });
    
    msg!(&label.clone());
    
    Ok(())
  }
}