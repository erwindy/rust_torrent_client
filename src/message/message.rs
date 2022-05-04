use std::{io::{Read, BufReader, BufRead}, net::TcpStream, cell::RefCell};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MessageId {
    // MsgChoke chokes the receiver
	MsgChoke = 0,
	// MsgUnchoke unchokes the receiver
	MsgUnchoke = 1,
	// MsgInterested expresses interest in receiving data
	MsgInterested = 2,
	// MsgNotInterested expresses disinterest in receiving data
	MsgNotInterested = 3,
	// MsgHave alerts the receiver that the sender has downloaded a piece
	MsgHave = 4,
	// MsgBitfield encodes which pieces that the sender has downloaded
	MsgBitfield = 5,
	// MsgRequest requests a block of data from the receiver
	MsgRequest = 6,
	// MsgPiece delivers a block of data to fulfill a request
	MsgPiece = 7,
	// MsgCancel cancels a request
	MsgCancel = 8,
}

fn uint2messageId(num: u8) -> MessageId {
    match num {
        0 => MessageId::MsgChoke,
        1 => MessageId::MsgUnchoke,
        2 => MessageId::MsgInterested,
        3 => MessageId::MsgNotInterested,
        4 => MessageId::MsgHave,
        5 => MessageId::MsgBitfield,
        6 => MessageId::MsgRequest,
        7 => MessageId::MsgPiece,
        8 => MessageId::MsgCancel,
        _ => MessageId::MsgCancel,
    }
}

#[derive(Debug)]
pub struct Message {
    pub id: MessageId,
    pub payload: Vec<u8>,
}

pub fn format_request(index: usize, begin: usize, length: usize) -> Message {
    let mut payload = vec![];
    payload.extend_from_slice(&(index as u32).to_be_bytes());
    payload.extend_from_slice(&(begin as u32).to_be_bytes());
    payload.extend_from_slice(&(length as u32).to_be_bytes());
    Message {
        id: MessageId::MsgRequest,
        payload
    }
}

pub fn parse_piece(index: usize, buf: &mut [u8], msg: &Message) -> u32 {
	if msg.id != MessageId::MsgPiece {
		return 0;
	}
	if msg.payload.len() < 8 {
		return 0;
	}
    let mut buffer = [0u8; 4];
    buffer.copy_from_slice(&msg.payload[0..4]);
    let parsed_index = u32::from_be_bytes(buffer) as usize;
	if parsed_index != index {
		return 0;
	}
    buffer.copy_from_slice(&msg.payload[4..8]);
    let begin = u32::from_be_bytes(buffer) as usize;
	if begin >= buffer.len() {
		return 0;
	}
	let data = &msg.payload[8..];
	if begin + data.len() > buf.len() {
		return 0;
	}
    buf[begin..].copy_from_slice(data);
	return data.len() as u32;
}

pub fn format_have(i: usize) -> Message {
    let mut payload = vec![];
    let index = i as u32;
    payload.extend_from_slice(&index.to_be_bytes());
    Message {
        id: MessageId::MsgHave,
        payload,
    }
}

pub fn parse_have(msg: &Message) -> u32 {
    if msg.id == MessageId::MsgHave {
        return 0;
    }
    if msg.payload.len() != 4 {
        return 0;
    }
    let mut buf = [0u8; 4];
    buf.copy_from_slice(&msg.payload);
    let index = u32::from_be_bytes(buf);
    return index;
}

pub fn read(conn: &TcpStream) -> Option<Message> {
    // let stream = RefCell::new(conn);

    let mut reader = BufReader::new(conn);

    // let mut all_data = vec![];
    // stream.borrow_mut().read_to_end(&mut all_data);
    // println!("all data {:?}", all_data);

    // if buffer.len() < 4> {
    //     return None;
    // }
    let mut length_buf = [0u8; 4];
    // length_buf.copy_from_slice(&buffer[0..4]);
    // stream.borrow_mut().read_exact(&mut length_buf);
    reader.read_exact(&mut length_buf);

    let length = u32::from_be_bytes(length_buf); //((length_buf[0] as u32) << 24) + ((length_buf[1] as u32) << 16) + ((length_buf[2] as u32) << 8) + (length_buf[3] as u32);
    println!("buffer length {:?} {}", length_buf, length);
    if length == 0 {
        return None;
    }

    let mut message_buf = vec![0u8; length as usize];
    let buf_res = reader.read_exact(&mut message_buf); // stream.borrow_mut().read_exact(&mut message_buf);
    // message_buf.copy_from_slice(&buffer[4..4+(length as usize)]);
    if let Err(err) = buf_res {
    //     // println!("stream read message error {:?}", err);
        return None;
    }
    // println!("stream read message success {:?}", message_buf.len());

    // let mut buffer: Vec<u8> = Vec::new();
    // reader
    //     .read_until(b'\n', &mut buffer)
    //     .expect("Could not read into buffer");
    // println!("all buffer {:?} {}", buffer, reader.buffer().is_empty());

    Some(Message {
        id: uint2messageId(message_buf[0]),
        payload: message_buf[1..].to_vec(),
    })
}

impl Message {
    pub fn new(id: MessageId, payload: Vec<u8>) -> Self {
        Self {
            id,
            payload: payload,
        }
    }

    pub fn serialize(&self) -> Vec<u8> {
        let length = self.payload.len() + 1;
        let mut buf = vec![0u8; 4 + length];
        let len_buf: [u8; 4] = (length as u32).to_be_bytes();
        buf[0..4].copy_from_slice(&len_buf);
        // println!("serial buf {:?}", buf);
        let id = self.id as u8;
        buf[4] = id;
        buf[5..].copy_from_slice(&self.payload[..]);
        buf
    }
}
