use std::{cell::RefCell, io};

use crate::{peers::peers::Peer, torrent_file::torrent_file::CustomTorrent, client::client::CustomClient, bitfield::bitfield::{has_piece, set_piece}, message};

const MAX_BACK_LOG: usize = 5;
const MAX_BLOCK_SIZE: usize = 16384;

#[derive(Debug)]
pub struct P2pTorrent {
    peers: Vec<Peer>,
    peerId: [u8; 20],
    infoHash: [u8; 20],
    piece_hashes: Vec<[u8; 20]>,
    piece_length: usize,
    length: usize,
    name: String,
}

#[derive(Debug)]
struct PieceWork {
    index: usize,
    hash: [u8; 20],
    length: usize,
}

#[derive(Debug)]
struct PieceResult {
    index: usize,
    buffer: Vec<u8>,
}

struct PieceProgress<'a> {
    index: usize,
    client: RefCell<&'a CustomClient<'a>>,
    buf: Vec<u8>,
    downloaded: usize,
    requested: usize,
    backlog: usize,
}

impl <'a>PieceProgress<'a> {
    pub fn read_message(&mut self) {
        let client = self.client.borrow_mut();
        let msg = client.read();
        if let None = msg {
            return;
        }
        let msg = msg.unwrap();
        match msg.id {
            message::message::MessageId::MsgUnchoke => client.set_choked(false),
            message::message::MessageId::MsgChoke => client.set_choked(true),
            message::message::MessageId::MsgInterested => todo!(),
            message::message::MessageId::MsgNotInterested => todo!(),
            message::message::MessageId::MsgHave => {
                let index = message::message::parse_have(&msg);
                // println!("设置bit field");
                set_piece(& RefCell::new(&client.bit_field), index as usize);
            },
            message::message::MessageId::MsgBitfield => todo!(),
            message::message::MessageId::MsgRequest => todo!(),
            message::message::MessageId::MsgPiece => {
                let n = message::message::parse_piece(self.index, &mut self.buf, &msg);
                self.downloaded += n as usize;
                self.backlog -= 1;
                // println!("接收到piece数据 {:?} {:?}", self.buf, msg);
            },
            message::message::MessageId::MsgCancel => todo!(),
        }
    }
}

fn attempt_download_piece(c: &CustomClient, pw: &PieceWork) -> Result<Vec<u8>, io::Error> {
    let mut state = PieceProgress {
		index:  pw.index,
		client: RefCell::new(c),
		buf: vec![],
        downloaded: 0,
        requested: 0,
        backlog: 0,
	};

    println!("开始下载piece");

    if state.downloaded < pw.length {
        if !*c.choked.borrow() {
            if state.backlog < MAX_BACK_LOG && state.requested < pw.length {
                let mut block_size = MAX_BLOCK_SIZE;

                if pw.length - state.requested < block_size {
                    block_size = pw.length - state.requested;
                }

                // println!("下载piece 发送请求 {} {} {}", pw.index, state.requested, block_size);

                c.send_request(pw.index, state.requested, block_size)?;

                state.backlog += 1;
                state.requested += block_size;
            }
        }
        println!("接收数据");
        state.read_message();
    }

    return Ok(state.buf);
}

impl P2pTorrent {
    pub fn general_p2p_torrent(custom_torrent: &CustomTorrent, peers: Vec<Peer>, peer_id: [u8; 20]) -> Self {
        Self {
            peers: peers,
            peerId: peer_id,
            infoHash: custom_torrent.info_hash,
            piece_hashes: custom_torrent.piece_hashes.clone(),
            piece_length: custom_torrent.piece_length,
            length: custom_torrent.length,
            name: custom_torrent.name.clone(),
        }
    }

    fn start_download_worker(&self, peer: &Peer, work_queue: &Vec<PieceWork>, results: &RefCell<&mut Vec<PieceResult>>) {
        let client = CustomClient::new(peer, self.peerId, &self.infoHash);
        if let Err(_) = client {
            // println!("init client error: {} {:?}", e, peer);
            return;
        }
        // println!("init client success");
        let c = client.unwrap();
        c.send_unchoke().err();
        c.send_interested().err();

        for pw in work_queue.iter() {
            if !has_piece(&c.bit_field, pw.index) {
                continue;
            }

            let buf = attempt_download_piece(&c, pw);
            if let Err(_) = buf {
                return;
            }
            let buf = buf.unwrap();

            c.send_have(pw.index).err();
            results.borrow_mut().push(PieceResult {
                index: pw.index,
                buffer: buf,
            });
        }
    }

    fn calculate_bounds_for_piece(&self, index: usize) -> (usize, usize) {
        let begin = index * self.piece_length;
        let mut end = begin + self.piece_length;
        if end > self.length {
            end = self.length;
        }
        (begin, end)
    }

    fn calculate_piece_size(&self, index: usize) -> usize {
        let (begin, end) = self.calculate_bounds_for_piece(index);
        end - begin
    }

    pub fn download(&self) -> Vec<u8> {
        let mut work_queue = vec![];
        let mut results = vec![];
        let results_ref = RefCell::new(&mut results);
        for index in 0..self.piece_hashes.len() {
            let length = self.calculate_piece_size(index);
            work_queue.push(PieceWork {
                index,
                hash: self.piece_hashes[index],
                length,
            })
        }

        println!("Downloading peers {:?}", &self.peers.len());

        for peer in &self.peers {
            self.start_download_worker(peer, &work_queue, &results_ref)
        }

        let mut buf = vec![0u8; self.length];
        let mut done_pieces = 0;
        while done_pieces < self.piece_hashes.len() {
            let res = results_ref.borrow_mut().pop();
            if let None = res {
                break;
            }
            let res = res.unwrap();
            let (begin, end) = self.calculate_bounds_for_piece(res.index);
            buf[begin..end].copy_from_slice(&res.buffer);
            done_pieces += 1;
            // let percent = ((done_pieces as f64) / (self.piece_hashes.len() as f64)) * 100.0;
        }
        return buf;
    }
}
