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
