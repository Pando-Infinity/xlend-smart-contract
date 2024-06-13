use anchor_lang::{prelude::Pubkey, AnchorDeserialize, AnchorSerialize};
use std::io::{self, *};

const PAYLOAD_ID_MESSAGE: u8 = 0;

pub const MESSAGE_PAYLOAD_MAX_LENGTH: usize = 512;

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
            PAYLOAD_ID_MESSAGE.serialize(writer)?;
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
    let length = buf.len();
    if length > MESSAGE_PAYLOAD_MAX_LENGTH {
      Err(Error::new(
        ErrorKind::InvalidInput,
        format!("message payload exceeds {MESSAGE_PAYLOAD_MAX_LENGTH}")
      ))
    } else {
      Ok(WormholeMessage::Message { payload: buf[length].to_vec() })
    }
  }
  
  fn deserialize_reader<R: Read>(reader: &mut R) -> Result<Self> {
    Ok(WormholeMessage::Message { payload: b"".to_vec() })
  }
}