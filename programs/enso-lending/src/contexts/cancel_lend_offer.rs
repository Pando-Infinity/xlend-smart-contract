use anchor_lang::{prelude::*};
use crate::common::{CancelLendOfferEvent, LendOfferStatus, LendOfferError};
use crate::states::lend_offer::LendOfferAccount;

#[derive(Accounts)]
#[instruction(offer_id: String)]
pub struct CancelLendOffer<'info> {
#[account(mut)]
  pub lender: Signer<'info>,
  #[account(
    mut,
    constraint = lend_offer.status == LendOfferStatus::Created @ LendOfferError::InvalidOfferStatus,
    seeds = [
      b"enso".as_ref(), 
      b"lend_offer".as_ref(), 
      lender.key().as_ref(), 
      offer_id.as_bytes(),
      crate::ID.key().as_ref(), 
    ],
    bump = lend_offer.bump
  )]
  pub lend_offer: Account<'info, LendOfferAccount>,
}

impl<'info> CancelLendOffer<'info> {
  pub fn cancel_lend_offer(&mut self) -> Result<()>  {
    let lend_offer = &mut self.lend_offer;
    lend_offer.status = LendOfferStatus::Canceled;

    Ok(())
  }

  pub fn emit_event_cancel_lend_offer(&mut self, label: String, offer_id: String) -> Result<()> {
    emit!(CancelLendOfferEvent {
      lender: self.lender.key(),
      offer_id
    });

    msg!(&label.clone());

    Ok(())
  }
}