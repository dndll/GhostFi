use dep::ed25519;
use dep::std::println;
use dep::std::ecdsa_secp256k1;

global HEURISTICS = 2;

global PARAMS = 16;

//global bank_signatory = publickeyhere

// Introduce more fns
struct HeuristicRegistry {
    // Require that the balance is 4x the requested amount
    fourx: fn(Field, Field) -> bool,
    lender: fn(Field, Field, Field, Field, Field) -> bool,
}

impl HeuristicRegistry {
    fn new() -> Self {
        // Trusted signatory is a lender: Essentially sudo signatory

        Self {
            fourx: |requested: Field, balance: Field| (balance * 4) as u126 >= requested as u126,
            lender: |x1: Field, x2: Field, y1: Field, y2: Field, _msg: Field| {
                let lender_x: [u8; 32] = [231, 34, 31, 32, 161, 56, 175, 167, 5, 160, 183, 168, 83, 127, 38, 250, 36, 119, 114, 189, 164, 200, 143, 203, 130, 243, 105, 252, 2, 156, 89, 151];
                let lender_y: [u8; 32] = [9, 216, 201, 160, 66, 226, 68, 194, 88, 241, 153, 97, 161, 15, 113, 117, 160, 141, 38, 3, 213, 31, 67, 65, 137, 153, 113, 131, 248, 27, 35, 46]; 

                let real_signature: [u8; 64] = [183, 10, 182, 91, 162, 175, 27, 211, 174, 19, 55, 48, 186, 177, 6, 49, 86, 162, 231, 102, 254, 7, 134, 35, 127, 179, 137, 102, 118, 17, 40, 110, 13, 58, 30, 50, 88, 57, 236, 43, 70, 141, 247, 171, 74, 151, 91, 7, 94, 82, 10, 82, 193, 61, 93, 74, 237, 104, 85, 82, 91, 233, 195, 202];
                let new_signature = concat_signature(x1, x2, y1, y2);
                let mut signature = [0; 64];

                for i in 0..64 {
                    assert(new_signature[i] == real_signature[i]);
                    signature[i] = new_signature[i];
                }
                let msg = [1; 32];

                ecdsa_secp256k1::verify_signature(lender_x, lender_y, signature, msg)
            }
            // Schnorr signatures, for signatories([Signatures]),
        }
    }

    fn apply(self, params: HeuristicParams, requested: Field) -> bool {
        if params.id == 1 {
            let f = self.fourx;
            f(requested, params.params[0])
        } else if params.id == 2 { 
            let f = self.lender;
            f(params.params[0], params.params[1], params.params[2], params.params[3], params.params[4])
        } else if params.id == 0 {
            // Noop identifier
            true 
        } else {
            false
        }
    }
}

unconstrained fn concat_signature(x1: Field, x2: Field, y1: Field, y2: Field) -> [u8; 64] {
    // This is the problematic thing, even though the created arrays are the same
    // as the sample signature, if you use this in signature verification it passes,
    // but causes the barretenberg backend to panic.
    let x1_bytes = x1.to_be_bytes(16);
    let x2_bytes = x2.to_be_bytes(16);
    let y1_bytes = y1.to_be_bytes(16);
    let y2_bytes = y2.to_be_bytes(16);

    let mut signature: [u8; 64] = [0; 64];

    for i in 0..16 {
        signature[i] = x1_bytes[i];
    }
    for i in 0..16 {
        signature[i + 16] = x2_bytes[i];
    }
    for i in 0..16 {
        signature[i + 32] = y1_bytes[i];
    }
    for i in 0..16 {
        signature[i + 48] = y2_bytes[i];
    }
    signature
}

/// Opaque parameters for heuristics, 
struct HeuristicParams {
    id: u8,
    params: [Field; PARAMS],
}

impl HeuristicParams {
    fn new(id: u8, params: [Field]) -> Self {
        let mut inner = [0; PARAMS];
        for i in 0..params.len() {
            inner[i] = params[i];
        }
        Self {
            id: id,
            params: inner 
        }
    }
}

fn main(public_key: pub [u8; 32], requested_amount: pub Field, params: [HeuristicParams; HEURISTICS]) {
    assert(public_key.len() == 32);

    let reg = HeuristicRegistry::new();

    for param in params {
        let applied = reg.apply(param, requested_amount);
        println(param.id);
        println(applied);
        assert(applied);
    }
    println("done");
}


#[test]
fn test_simple() {
    let pubkey = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
    let requested_amount = 100;
    let params = HeuristicParams::new(1, [25]);
    main(pubkey, requested_amount, [params, HeuristicParams::new(0, [])]);
}

#[test]
fn test_simple_fail() {
    let pubkey = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
    let requested_amount = 101;
    let params = HeuristicParams::new(1, [25]);
    main(pubkey, requested_amount, [params, HeuristicParams::new(0, [])]);
}

