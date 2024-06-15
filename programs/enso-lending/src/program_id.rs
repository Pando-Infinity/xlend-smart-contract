use anchor_lang::prelude::Pubkey;
use solana_program::pubkey;

cfg_if::cfg_if!{
  if #[cfg(all(feature = "devnet", feature = "beta-test"))] {
    pub const PROGRAM_ID: Pubkey = pubkey!("BderhzujHHQNjhCZGRjLWnN2XQ66q4EZnZx2p5WLJnBV");
  } else {
    // Default use for localnet
    pub const PROGRAM_ID: Pubkey = pubkey!("G3LQL3DpD8Bd5q3ERGZwqAgkgyFKJTbt9ViCpu1hVd6o");
  }
}