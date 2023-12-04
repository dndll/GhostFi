use dep::ed25519;

fn main(pubkey: [u8; 32], signature_r: [u8; 32], signature_s: [u8; 32], hashed_msg: [u8; 32], balance: pub Field) {
    //assert(balance as u128 >= balance as u128 * 4);
    //assert(ed25519::verify(pubkey, signature_r, signature_s, hashed_msg));
}


#[test]
fn test_main() {
}

#[test]
fn test_sha512() {
    let s = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];
    dep::std::sha512::digest(s);
}

#[test]
fn test_sha512_broken_slices() {
    let s = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32];
    dep::std::sha512::digest(s.as_slice());
}


