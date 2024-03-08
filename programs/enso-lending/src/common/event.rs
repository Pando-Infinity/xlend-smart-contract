use anchor_lang::{
    event,
    prelude::{borsh, AnchorDeserialize, AnchorSerialize, Pubkey},
};

#[event]
pub struct InitSettingAccountEvent {
    pub amount: u64,
    pub duration: u64,
    pub owner: Pubkey,
    pub receiver: Pubkey,
    pub lend_mint_asset: Pubkey,
    pub collateral_mint_asset: Pubkey,
    pub tier_id: String,
    pub lender_fee_percent: f64,
    pub lend_price_feed: Pubkey,
    pub collateral_price_feed: Pubkey,
}

#[event]
pub struct EditSettingAccountEvent {
    pub receiver: Pubkey,
    pub lend_mint_asset: Pubkey,
    pub collateral_mint_asset: Pubkey,
    pub tier_id: String,
    pub amount: u64,
    pub duration: u64,
    pub lender_fee_percent: f64,
    pub lend_price_feed: Pubkey,
    pub collateral_price_feed: Pubkey,
}

#[event]
pub struct CloseSettingAccountEvent {
    pub tier_id: String,
}

#[event]
pub struct CreateLendOfferEvent {
    pub lender: Pubkey,
    pub interest: f64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub amount: u64,
    pub offer_id: String,
    pub tier_id: String,
}

#[event]
pub struct EditLendOfferEvent {
    pub lender: Pubkey,
    pub interest: f64,
    pub lender_fee_percent: f64,
    pub duration: u64,
    pub amount: u64,
    pub offer_id: String,
}