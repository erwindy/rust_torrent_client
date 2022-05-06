use bendy::{
    decoding::{Error as BendyError, FromBencode, Object},
};

use crate::peers::peers::{un_marshal, Peer};

#[derive(Debug)]
pub struct BencodeTrackerResp {
    pub interval: u64,
    pub peers: Vec<Peer>,
}

impl FromBencode for BencodeTrackerResp {
    fn decode_bencode_object(object: Object) -> Result<Self, BendyError>
    where
        Self: Sized,
    {
        let mut interval = 0u64;
        let mut peers = vec![];

        let mut dict_dec = object.try_into_dictionary()?;

        while let Some(pair) = dict_dec.next_pair()? {
            match pair {
                (b"interval", value) => {
                    fn default_v(_: bendy::decoding::Error) -> u64 { 0 }
                    interval = u64::decode_bencode_object(value).unwrap_or_else(default_v);
                },
                (b"peers", value) => {
                    // fn default_v(s: bendy::decoding::Error) -> String { "".to_string() }
                    // peers = String::decode_bencode_object(value).unwrap_or_else(default_v);
                    let byte = value.bytes_or(Err(""));
                    if let Ok(bytes) = byte {
                        peers = un_marshal(bytes);
                    }
                },
                (_, _) => {},
            }
        }

        Ok(BencodeTrackerResp {
            interval,
            peers,
        })
    }
}
