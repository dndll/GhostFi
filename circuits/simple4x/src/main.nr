use dep::std::ec::montcurve::affine::{Curve, Point as AffinePoint};
use dep::std::ec::montcurve::curvegroup::Point;
use dep::std::field::bytes32_to_field;

// https://neuromancer.sk/std/other/Curve25519
// 0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffed
global p = 57896044618658097711785492504343953926634992332820282019728792003956564819949;

// 0x76d06
global a = 486662;
// b	0x01
global b = 1;
// G	(0x09, 0x20ae19a1b8a086b4e01edd2c7748d14c923d4d7e6d7c61b229e9c5a27eced3d9)
global g1 = 9;
global g2 = 14781619447589544791020593568409986887264606134616475288964881837755586237401;

// n	0x1000000000000000000000000000000014def9dea2f79cd65812631a5cf5d3ed
global order = 7237005577332262213973186563042994240857116359379907606001950938285454250989;
// h	0x08
global cofactor = 8;

// Characteristics
global j_invariant = 39240375672115510010799456308813573486606784421612167109713554819120306934551;
global discriminant = 3789438435840;
// TODO: fix sign to -
global trace_of_frobenius = 221938542218978828286815502327069187962;


// FIXME: all of these fields use grumpkin, we need to use raw bytes here

fn main(pubkey: Field, balance: pub Field) {
    assert(balance as u128 >= balance as u128 * 4);
}

fn compute_prehashed_challenge(r: [u8; 32], a: [u8; 32], m: [u8; 32]) -> [u8; 64] {
    let mut parts = [u8];
    parts.append(r.as_slice());
    parts.append(a.as_slice());
    parts.append(m.as_slice());

    dep::std::sha512::digest(parts) 
}

fn recompute_r(signature_r: [u8; 32], signature_s: [u8; 32], a: [u8; 32], prehashed_message: [u8; 32]) -> Field {
    let h = compute_prehashed_challenge(signature_r, a, prehashed_message);

    // TODO: decompress point and convert from TE to Mont 
    let minus_A = h.negate().into_tecurve();
    let computed_r = minus_A.x.mul(h);
    computed_r


}

fn verify_prehashed(pubkey: [u8; 32], signature_r: [u8; 32], signature_s: [u8; 32], prehashed_message: [u8; 32]) -> bool {
    let curve = curve25519();

    // TODO: decode point from pubkey
    let a = AffinePoint::new(pubkey, g2);
    let h = compute_prehashed_challenge(signature_r, pubkey, prehashed_message);

    // TODO: convert to edwards
    let b_s = b.mul(signature_s);
    let r_s = a.mul(h);
    assert(b_s == r_s);


    // TODO: decompress signature R
    let r = recompute_r(signature_r, signature_s, a, prehashed_message);

    // TODO: verify order of r and point
    assert(r == signature_r);

    true
}

fn curve25519() -> Curve {
    let generator = AffinePoint::new(g1, g2);
    let curve = Curve::new(a, b, generator);
    curve
}

#[test]
fn test_main() {
    main(1, 2);

    // Uncomment to make test fail
    // main(1, 1);
}
