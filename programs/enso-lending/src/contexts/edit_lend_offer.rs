use anchor_lang::prelude::*;

use crate::{EditLendOfferEvent, LendOfferAccount, LendOfferError, LendOfferStatus};

#[derive(Accounts)]
#[instruction(offer_id: String)]
pub struct EditLendOffer<'info> {
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

impl<'info> EditLendOffer<'info> {
    pub fn edit_lend_offer(&mut self, interest: f64) -> Result<()> {
      if interest <= (0 as f64) {
        return err!(LendOfferError::InterestGreaterThanZero);
      }

      let lend_offer = &mut self.lend_offer;
      lend_offer.interest = interest;

      Ok(())
    }

    pub fn emit_event_edit_lend_offer(&mut self, label: String) -> Result<()> {
      emit!(EditLendOfferEvent {
        lender: self.lender.key(),
        interest: self.lend_offer.interest,
        lender_fee: self.lend_offer.lender_fee,
        amount: self.lend_offer.amount,
        duration: self.lend_offer.duration,
        offer_id: self.lend_offer.offer_id.clone(),
    });
    
    msg!(&label.clone());
    
    Ok(())
    }
}