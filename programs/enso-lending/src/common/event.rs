use anchor_lang::{
    event,
    prelude::{borsh, AnchorDeserialize, AnchorSerialize},
};

#[event]
pub struct InitSettingAccountEvent {
    pub tier_id: String,
}
