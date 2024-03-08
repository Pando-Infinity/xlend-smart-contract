use anchor_lang::prelude::{borsh, AnchorDeserialize, AnchorSerialize, InitSpace};

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum LendOfferStatus {
    Created,
    Loaned,
    Canceled
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum LoanOfferStatus {
    Created,
    Loaned,
    Canceled
}

pub const OPERATE_STSTEM_PUBKEY: &str = "CanzRrJ6U81DLSRx2vJx3w9s9ssy23vizkTqZyGyevFf";

pub const STALENESS_THRESHOLD : u64 = 60; // staleness threshold in seconds