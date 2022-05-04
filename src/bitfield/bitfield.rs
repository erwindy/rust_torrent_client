use std::cell::RefCell;

pub type Bitfield = Vec<u8>;

pub fn has_piece(bitfield: &Bitfield, index: usize) -> bool {
    let byte_index = index / 8;
    let offset = index % 8;
    if byte_index < 0 || byte_index >= bitfield.len() {
        return false;
    }
    bitfield[byte_index]>>(7 - offset)&1 != 0
}

pub fn set_piece(bitfield: & RefCell<&Bitfield>, index: usize) {
    let byte_index = index / 8;
    let offset = index & 8;

    if byte_index < 0 || byte_index >= bitfield.borrow().len() {
        return;
    }
    let mut _bit = bitfield.borrow_mut()[byte_index];
    _bit |= 1 << (7 - offset)
}
