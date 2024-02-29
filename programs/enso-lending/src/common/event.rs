use anchor_lang::{
    event,
    prelude::{borsh, AnchorDeserialize, AnchorSerialize, Pubkey},
};

#[event]
pub struct InitSettingAccountEvent {
    pub amount: f64,
    pub duration: u64,
    pub owner: Pubkey,
    pub receiver: Pubkey,
    pub lend_mint_asset: Pubkey,
    pub collateral_mint_asset: Pubkey,
    pub tier_id: String,
}

#[event]
pub struct EditSettingAccountEvent {
    pub receiver: Pubkey,
    pub lend_mint_asset: Pubkey,
    pub collateral_mint_asset: Pubkey,
    pub tier_id: String,
    pub amount: Option<f64>,
    pub duration: Option<u64>,
    pub lender_fee_percent: Option<f64>
}

#[event]
pub struct CloseSettingAccountEvent {
    pub tier_id: String,
}
