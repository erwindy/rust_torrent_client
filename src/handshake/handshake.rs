use std::{net::TcpStream, cell::RefCell, io::Read, vec};

#[derive(Debug, Clone)]
pub struct Handshake<'a> {
    pub pstr: &'a str,
    pub info_hash: &'a [u8; 20],
    pub peer_id: &'a [u8; 20],
}

impl <'a>Handshake<'a> {
    pub fn new(info_hash: &'a [u8; 20], peer_id: &'a [u8; 20]) ->Self {
        Self {
            pstr: "BitTorrent protocol",
            info_hash,
            peer_id,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let pstr_len = self.pstr.as_bytes().len();
        let mut buf = vec![0u8; pstr_len + 49];
        buf[0] = pstr_len as u8;
        let mut curr = 1;
        buf[curr..curr+pstr_len].copy_from_slice(self.pstr.as_bytes());
        curr += pstr_len;
        buf[curr..curr+8].copy_from_slice(&[0u8; 8]);
        curr += 8;
        buf[curr..curr+self.info_hash.len()].copy_from_slice(self.info_hash);
        curr += self.info_hash.len();
        buf[curr..].copy_from_slice(self.peer_id);
        buf
    }
}

pub fn read(conn: &TcpStream) -> Option<[u8; 20]> {
    let stream = RefCell::new(conn);
    let mut length_buf = [0u8; 1];
    stream.borrow_mut().read_exact(&mut length_buf);
    let pstr_len = length_buf[0] as usize;
    if pstr_len == 0 {
        return None;
    }
    let mut handshake_buf = vec![0; 48 + pstr_len];
    stream.borrow_mut().read_exact(&mut handshake_buf);

    let mut info_hash = [0u8; 20];
    let mut peer_id = [0u8; 20];

    {
        info_hash.copy_from_slice(&handshake_buf[pstr_len + 8..pstr_len + 8 + 20]);
        peer_id.copy_from_slice(&handshake_buf[pstr_len+ 8 + 20..]);
    }

    // Handshake {
    //     pstr: &String::from_utf8(handshake_buf[0..pstr_len].to_vec()).unwrap(),
    //     info_hash: &[0u8; 20],
    //     peer_id: &[0u8; 20],
    // }

    Some(info_hash)
}
