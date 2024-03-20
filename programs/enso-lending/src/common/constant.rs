use anchor_lang::prelude::{borsh, AnchorDeserialize, AnchorSerialize, InitSpace};

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum LendOfferStatus {
    Created,
    Loaned,
    Canceled
}

#[derive(AnchorSerialize, AnchorDeserialize, Copy, Clone, PartialEq, Eq, InitSpace, Debug)]
pub enum LoanOfferStatus {
    Matched,
    Repay,
    Finished,
    Liquidating,
    Liquidated
}

pub const ENSO_SEED: &[u8] = b"enso";
pub const SETTING_ACCOUNT_SEED: &[u8] = b"setting_account";
pub const LEND_OFFER_ACCOUNT_SEED: &[u8] = b"lend_offer";
pub const LOAN_OFFER_ACCOUNT_SEED: &[u8] = b"loan_offer";

pub const OPERATE_SYSTEM_PUBKEY: &str = "CanzRrJ6U81DLSRx2vJx3w9s9ssy23vizkTqZyGyevFf";

pub const STALENESS_THRESHOLD : u64 = 60; // staleness threshold in seconds

pub const MIN_BORROW_HEALTH_RATIO: f64 = 1.5;

pub const NATIVE_MINT: &str = "So11111111111111111111111111111111111111112";