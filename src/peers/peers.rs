use std::net::{self, SocketAddr, Ipv4Addr, IpAddr};

#[derive(Debug)]
pub struct Peer {
    ip: Ipv4Addr,
    port: u16,
}

impl Peer {
    pub fn general_address(&self) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(self.ip), self.port)
    }
}

pub fn un_marshal(peers_bin: &[u8]) -> Vec<Peer> {
    let peer_size = 6;
    let num_peers = peers_bin.len() / peer_size;
    let mut peers = vec![];
    if peers_bin.len() % peer_size != 0 {
        return peers;
    }
    for i in 0..num_peers {
        let offset = i * peer_size;
        let left = u16::from(peers_bin[offset+4]);
        let right = u16::from(peers_bin[offset+5]);
        let port  = (left << 8) + right;

        peers.push(Peer {
            ip: net::Ipv4Addr::new(peers_bin[offset], peers_bin[offset+1], peers_bin[offset+2], peers_bin[offset+3]),
            port,
        })
    }
    return peers;
}
