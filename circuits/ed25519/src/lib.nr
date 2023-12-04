use dep::std::ec::tecurve::affine::{Curve, Point as AffinePoint};
use dep::std::ec::tecurve::curvegroup::Point;
use dep::std::field::bytes32_to_field;
use dep::bigint::BigUint56;
use dep::std::assert_constant;
use dep::std::println;
use dep::std::option::Option;


// FIXME: noir OOMS on anything more than 0.9 for exponentiation

// https://neuromancer.sk/std/other/Ed25519
global field_p = 57896044618658097711785492504343953926634992332820282019728792003956564819949;
global a = 57896044618658097711785492504343953926634992332820282019728792003956564819948;
global d = 37095705934669439343138083508754565189542113879843219016388785533085940283555;
global g1 = 15112221349535400772501151409588531511454012693041857206046113283949847762202;
global g2 = 46316835694926478169428394003475163141307993866256225615783033603165251855960;
global n = 7237005577332262213973186563042994240857116359379907606001950938285454250989;
global h = 8;

fn compute_challenge(r: [u8; 32], a: [u8; 32], m: [u8; 32]) -> [u8; 64] {
    let mut parts = [0; 96];
    let mut count = 0;
    for i in r {
        parts[count] = i;
        count += 1;
    }
    for i in a {
        parts[count] = i;
        count += 1;
    }
    for i in m {
        parts[count] = i;
        count += 1;
    }

    dep::std::sha512::digest(parts) 
}

fn recompute_r(signature_r: [u8; 32], signature_s: [u8; 32], a: [u8; 32], message: [u8; 32]) -> AffinePoint {
    let k = compute_challenge(signature_r, a, message);
    let k = BigUint56::from_bytes2(k);

    let signature_scalar = check_scalar(signature_s).unwrap();

    let (curve, b) = ed25519();
    let minus_A = b.negate();

    
    // Recall the (non-batched) verification equation: -[k]A + [s]B = R
    let lhs = curve.bit_mul(k.to_bits(), minus_A);
    let rhs = curve.mul(signature_scalar as Field, b);

    let computed_r = curve.add(lhs, rhs);

    computed_r
}

fn compress_affine(point: AffinePoint) -> [u8; 32] {
    let point = point.into_group();
    let recip = point.z.pow_32(field_p - 2);
    let x = point.x * recip;
    let y = point.y * recip;

    let mut s = [0; 32];
    s = y.to_le_bytes(32);

    let is_negative: u8 = (x.to_le_bytes(32)[0] & 1);
    s[31] ^= is_negative << 7;

    let mut compressed_affine_array = [0; 32];
    let mut counter = 0;
    for i in 0..32 {
        compressed_affine_array[counter] = s[i];
        counter += 1;
    }
    compressed_affine_array
}

// FIXME: Could be bug here
// Check if highest 3 bits are set
fn check_scalar(mut signature_s: [u8; 32]) -> Option<u32> {
    let set = signature_s[31] & 224;
    if set != 0 {
      Option::none()
    } else {
      signature_s[31] &= 127;
      let s = bytes32_to_field(signature_s);
      Option::some(s as u32)
    }
}

pub fn verify(pubkey: [u8; 32], signature_r: [u8; 32], signature_s: [u8; 32], message: [u8; 32]) -> bool {
    let r = recompute_r(signature_r, signature_s, pubkey, message);
    assert(compress_affine(r) == signature_r);
    true
}

fn ed25519() -> (Curve, AffinePoint) {
    let generator = AffinePoint::new(g1, g2);
    let curve = Curve::new(a, d, generator);
    
    (curve, generator)
}

#[test]
fn test_ed25519() {
    let _sec_bytes = 0x833fe62409237b9d62ec77587520911e9a759cec1d19755b7da901b96dca3d42;

    let publ = [236,23,43,147,173,94,86,59,244,147,44,112,225,36,80,52,195,84,103,239,46,253,77,100,235,248,25,104,52,103,226,191];
    let mut msg = [0; 32];
    msg[0] = 97;
    msg[1] = 98;
    msg[2] = 99;
    let sig_r = [152,167,2,34,240,184,18,26,169,211,15,129,61,104,63,128,158,70,43,70,156,127,248,118,57,73,155,185,78,109,174,65];
    let sig_s = [49,248,80,66,70,60,42,53,90,32,3,208,98,173,245,170,161,11,140,97,230,54,6,42,170,209,28,42,38,8,52,6];

    assert(verify(publ, sig_r, sig_s, msg) == true);
}

