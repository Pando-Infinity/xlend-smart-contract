use anchor_lang::{error_code};

#[error_code]
pub enum SettingAccountError {
    #[msg("Invalid tier id")]
    InvalidTierId,
}