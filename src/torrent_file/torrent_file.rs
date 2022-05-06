use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::str;

use bendy::{
    decoding::{FromBencode},
};

use lava_torrent::torrent::v1::Torrent;
use rand::RngCore;
use url::Url;
extern crate url;
use url::form_urlencoded::{byte_serialize};

use crate::{torrent_file::tracker::BencodeTrackerResp, peers::peers::Peer, p2p::p2p::P2pTorrent};

#[derive(Debug)]
pub struct CustomTorrent {
    pub torrent: Torrent,
    pub announce: String,
    pub info_hash: [u8; 20],
    pub piece_hashes: Vec<[u8; 20]>,
    pub piece_length: usize,
    pub length: usize,
    pub name: String,
}

impl CustomTorrent {
    pub fn general_custom_torrent(torrent: Torrent) -> Result<Self, hex::FromHexError> {
        let mut info_hash = [0u8; 20];
        hex::decode_to_slice(torrent.info_hash(), &mut info_hash)?;

        // let hash_len = 20;
        let piece_hashes = torrent.pieces.iter().map(|p| {
            let mut piece = [0u8; 20];
            piece.copy_from_slice(&p[..]);
            piece
        }).collect::<Vec<_>>();

        Ok(CustomTorrent {
            torrent: torrent.clone(),
            announce: torrent.announce.unwrap_or_else(|| "".to_string()),
            info_hash,
            piece_hashes,
            piece_length: torrent.piece_length as usize,
            length: torrent.length as usize,
            name: torrent.name,
        })
    }


    pub fn down_load_to_file(&self, out_path: &str) {
        let mut peer_id = [0u8; 20];
        rand::thread_rng().fill_bytes(&mut peer_id);
        let peers = self.request_peers(&peer_id, 6881);
        let p2p_torrent = P2pTorrent::general_p2p_torrent(self, peers, peer_id);
        let buf = p2p_torrent.download();
        println!("下载完成 buf_len {}", buf.len());

        let path = Path::new(out_path);
        let display = path.display();
        let mut file = match File::create(&path) {
            Err(why) => panic!("couldn't create {}: {}", display, why.to_string()),
            Ok(file) => file,
        };
        match file.write_all(&buf) {
            Err(why) => {
                panic!("couldn't write to {}: {}", display, why.to_string());
            },
            Ok(_) => println!("successfully wrote to {}", display),
        }
    }

    fn request_peers(&self, peer_id: &[u8], port: u16) -> Vec<Peer> {
        let url = self.build_tracker_url(peer_id, port);
        let resp = reqwest::blocking::get(url.as_str()).unwrap();
        let tracker = BencodeTrackerResp::from_bencode(&resp.bytes().unwrap()).unwrap();
        tracker.peers
    }

    fn build_tracker_url(&self, peer_id: &[u8], port: u16) -> url::Url {
        let info_hash: String = byte_serialize(&self.info_hash).collect();
        let peer = byte_serialize(&peer_id).collect::<String>();// String::from_utf8_lossy(peer_id).to_string();
        let port = port.to_string();
        let uploaded = "0";
        let downloaded = "0";
        let compact = "1";
        let left = &self.length.to_string();

        let mut parsed = Url::parse(&self.announce).unwrap();
        let mut hash = "info_hash=".to_string() + &info_hash;
        hash += "&peer_id=";
        hash += &peer;
        parsed.set_query(Some(&hash));

        parsed.query_pairs_mut().append_pair("port", &port);
        parsed.query_pairs_mut().append_pair("uploaded", uploaded);
        parsed.query_pairs_mut().append_pair("downloaded", downloaded);
        parsed.query_pairs_mut().append_pair("compact", compact);
        parsed.query_pairs_mut().append_pair("left", left);

        return parsed;
    }

}

pub fn open(path: &str) -> Result<CustomTorrent, lava_torrent::LavaTorrentError> {
    let torrent = Torrent::read_from_file(path)?;

    let torrent = CustomTorrent::general_custom_torrent(torrent);
    if let Err(_) = torrent {
        return Err(lava_torrent::LavaTorrentError::from(std::io::Error::new(std::io::ErrorKind::Other, "")));
    }
    Ok(torrent.unwrap())
}

