use std::str::FromStr;

use anchor_lang::prelude::*;
use wormhole_anchor_sdk::wormhole::{self, PostedVaa};

use crate::{
    common::WORMHOLE_SYSTEM_PUBKEY, instruction, WormholeError, WormholeMessage, WormholeReceiveEvent
};

#[derive(Accounts)]
#[instruction(vaa_hash: [u8; 32])]
pub struct WormholeReceive<'info> {
  #[account(
    mut,
    constraint = system_wormhole.key() == Pubkey::from_str(WORMHOLE_SYSTEM_PUBKEY).unwrap() @ WormholeError::InvalidSystem
  )]
  pub system_wormhole: Signer<'info>,
	#[account(
		seeds = [
				wormhole::SEED_PREFIX_POSTED_VAA,
				&vaa_hash
		],
		bump,
		seeds::program = wormhole_program
	)]
	pub posted: Account<'info, wormhole::PostedVaa<WormholeMessage>>,
	pub wormhole_program: Program<'info, wormhole::program::Wormhole>,
	pub system_program: Program<'info, System>,
}

impl<'info> WormholeReceive<'info> {
	pub fn receive_message(
		&self,
		vaa_hash: [u8; 32]
	) -> Result<()> {
		let posted_vaa = self.posted.clone().into_inner();
		if let WormholeMessage::Message { payload } = posted_vaa.data() {
		let payload_data = self.get_data_from_vaa(payload).unwrap();

		emit!(WormholeReceiveEvent {
			data: payload_data
		});
	
		Ok(())
		} else {
		Err(WormholeError::InvalidMessage.into())
		}
	}

	fn get_data_from_vaa(
		&self,
		payload: &Vec<u8>
	) -> Result<Vec<String>> {
		let message = String::from_utf8(payload.clone()).unwrap();
		let splited_data: Vec<&str> = message.split(',').collect();

		Ok(splited_data.into_iter().map(String::from).collect())
	}
}