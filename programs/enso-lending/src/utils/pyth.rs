use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{get_feed_id_from_hex, PriceUpdateV2};

use crate::MAXIMUM_AGE_PRICE_UPDATE;

pub fn convert_to_usd_price(
    price_feed_account: &PriceUpdateV2,
    price_fee_id: &str,
    amount: f64,
) -> Result<f64> {
    let feed_id: [u8; 32] =
        get_feed_id_from_hex(price_fee_id)?;
    let current_price =
        price_feed_account.get_price_no_older_than(&Clock::get()?, MAXIMUM_AGE_PRICE_UPDATE, &feed_id)?;

    let display_price = current_price.price as f64 / 10f64.powf(-current_price.exponent as f64);

    Ok(display_price * amount)
}
