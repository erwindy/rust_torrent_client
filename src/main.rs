mod torrent_file;
mod peers;
mod p2p;
mod client;
mod message;
mod bitfield;
mod handshake;

fn main() {
    let in_path = "src/torrent_file/testdata/debian-11.3.0-amd64-netinst.iso.torrent";
    let out_path = "src/torrent_file/testdata/debian.iso";

    let res = torrent_file::torrent_file::open(&in_path);

    // if let Err(_) = res {
    //     return;
    // }
    // res.unwrap().down_load_to_file(out_path);
}
