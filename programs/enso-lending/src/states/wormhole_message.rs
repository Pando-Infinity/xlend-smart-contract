use anchor_lang::{prelude::Pubkey, solana_program::msg, AnchorDeserialize, AnchorSerialize};
use std::{fs::read, io::{self, *}, str};

pub const MESSAGE_PAYLOAD_MAX_LENGTH: usize = 255;

#[derive(Clone)]
pub enum WormholeMessage {
    Message {
        payload: Vec<u8>
    },
}

impl AnchorSerialize for WormholeMessage {
  fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
    match self {  
      WormholeMessage::Message { payload } => {
        if payload.len() > MESSAGE_PAYLOAD_MAX_LENGTH {
            Err(Error::new(
              ErrorKind::InvalidInput,
              format!("message payload exceeds {MESSAGE_PAYLOAD_MAX_LENGTH}")
            ))
        } else {
            (payload.len() as u16).to_be_bytes().serialize(writer)?;
            for element in payload {
              element.serialize(writer)?;
            }
            Ok(())
        }
      }
    }
  }
}

impl AnchorDeserialize for WormholeMessage {
  fn deserialize(buf: &mut &[u8]) -> io::Result<Self> {
    msg!("deserialize");
    Self::deserialize_reader(&mut *buf)
  }
  
  fn deserialize_reader<R: Read>(reader: &mut R) -> Result<Self> {
    let mut buffer: [u8; 255] = [0; 255];
    let length = reader.read(&mut buffer[..])?;
    if length > MESSAGE_PAYLOAD_MAX_LENGTH {
      Err(Error::new(
        ErrorKind::InvalidInput,
        format!("message payload exceeds {MESSAGE_PAYLOAD_MAX_LENGTH}")
      ))
    } else {
      Ok(WormholeMessage::Message { payload: buffer[..length].to_vec() })
    }
  }
}