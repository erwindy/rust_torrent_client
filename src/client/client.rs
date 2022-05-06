use std::{net::{TcpStream}, io::{Write, Error, ErrorKind}, cell::RefCell, time::Duration};

use crate::{peers::peers::Peer, message::message, bitfield::bitfield::Bitfield, handshake::handshake};

pub struct CustomClient<'a> {
    conn: RefCell<TcpStream>,
    pub choked: RefCell<bool>,
    pub bit_field: Bitfield,
    peer: &'a Peer,
    info_hash: &'a [u8; 20],
    peer_id: [u8; 20],
}

fn complete_handshake<'a>(conn: &'a TcpStream, info_hash: &'a [u8; 20], peer_id: &'a [u8; 20]) -> Option<[u8; 20]> {
    let stream = RefCell::new(conn);
    let req = handshake::Handshake::new(info_hash, peer_id);
    let ser = req.serialize();
    let res = stream.borrow_mut().write(&ser);

    if let Err(err) = res {
        println!("handshake error {}", err);
    }

    let res = handshake::read(conn);
    if let Ok(res_info_hash) = res {
        if &res_info_hash != info_hash {
            return None;
        } else {
            return Some(res_info_hash);
        }
    } else {
        return None;
    }
}

fn recv_bitfield(conn: &TcpStream) -> Option<Vec<u8>> {
    let msg = message::read(conn);

    if let None = msg {
        return None;
    }
    let mes = msg.unwrap();
    if mes.id != message::MessageId::MsgBitfield {
        return None;
    }
    return Some(mes.payload);
}

impl <'a>CustomClient<'a> {
    pub fn new(peer: &'a Peer, peer_id: [u8; 20], info_hash: &'a [u8; 20]) -> Result<Self, std::io::Error> {

        println!("创造tcpstream");
        let addr = peer.general_address();
        let stream = TcpStream::connect_timeout(&addr, Duration::new(3, 0))?;
        stream.set_read_timeout(Some(Duration::new(3, 0))).expect("set_write_timeout call failed");

        // println!("开始握手");
        let handshake_res = complete_handshake(&stream, info_hash, &peer_id);
        if let None = handshake_res {
            return Err(Error::new(ErrorKind::Other, "握手失败"));
        }
        // println!("握手结束");

        let bf = recv_bitfield(&stream);

        // println!("bitfield 数据");

        if let None = bf {
            return Err(Error::new(ErrorKind::Other, "oh no!"));
        }
        println!("client 创建成功");

        Ok(Self {
            conn: RefCell::new(stream),
            choked: RefCell::new(false),
            peer,
            info_hash,
            peer_id,
            bit_field: bf.unwrap(),
        })
    }

    // Read reads and consumes a message from the connection
    pub fn read(&self) -> Option<message::Message> {
        let msg = message::read(&mut self.conn.borrow_mut());
        return msg;
    }

    // SendRequest sends a Request message to the peer
    pub fn send_request(&self, index: usize, begin: usize, length: usize) -> Result<(), std::io::Error> {
        let req = message::format_request(index, begin, length);
        let mut conn = self.conn.borrow_mut();
        conn.write_all(&req.serialize())?;
        conn.flush()?;
        Ok(())
    }

    // SendInterested sends an Interested message to the peer
    pub fn send_interested(&self) -> Result<(), std::io::Error> {
        let msg = message::Message::new(message::MessageId::MsgInterested, vec![]);
        let mut conn = self.conn.borrow_mut();
        conn.write_all(&msg.serialize())?;
        conn.flush()?;
        Ok(())
    }

    // SendNotInterested sends a NotInterested message to the peer
    // pub fn send_not_interested(&self) {
    //     let msg = message::Message::new(message::MessageId::MsgNotInterested, vec![]);
    //     let mut conn = self.conn.borrow_mut();
    //     conn.write_all(&msg.serialize());
    //     conn.flush();
    // }

    // SendUnchoke sends an Unchoke message to the peer
    pub fn send_unchoke(&self) -> Result<(), std::io::Error> {
        let msg = message::Message::new(message::MessageId::MsgUnchoke, vec![]);
        let mut conn = self.conn.borrow_mut();
        conn.write_all(&msg.serialize())?;
        conn.flush()?;
        Ok(())
    }

    // SendHave sends a Have message to the peer
    pub fn send_have(&self, index: usize) -> Result<(), std::io::Error>{
        let msg = message::format_have(index);
        let mut conn = self.conn.borrow_mut();
        conn.write_all(&msg.serialize())?;
        conn.flush()?;
        Ok(())
    }

    pub fn set_choked(&self, choked: bool) {
        *self.choked.borrow_mut() = choked;
    }
}
