use dep::ed25519;
use dep::std::println;
use dep::schnorr;

global HEURISTICS = 2;
global PARAMS = 16;

global lender_signatory_x = pubkey (SUDO)
global lender_signatory_y = pubkey (SUDO)

//global bank_signatory = publickeyhere

// Introduce more fns
struct HeuristicRegistry {
    // Require that the balance is 4x the requested amount
    fourx: fn(Field, Field) -> bool,
    verify_lender_signatory: fn(Field, Field, [u8; 64], [u8]) -> bool,
}

impl HeuristicRegistry {
    fn new() -> Self {
        Self {
            fourx: |requested: Field, balance: Field| (balance * 4) as u127 >= requested as u127, 
            verify_lender_signatory: |public_key_x: Field, public_key_y: Field, signature: [u8; 64], message: [u8]| schnorr::verify_signature(public_key_x, public_key_y, signature, message),
            // Schnorr signatures, for signatories([Signatures]),
        }
    }
    fn apply(self, id: u8, entry: [Field; PARAMS], requested: Field) -> bool {
        if id == 0 {
            let f = self.fourx;
            f(requested, entry[0])
        } else if id == 1 {
            let f = self.verify_lender_signatory;
            f(lender_signatory_x, lender_signatory_y, entry[0], entry[1])
        } else {
            false
        }
    }
}

/// Opaque parameters for heuristics, 
struct HeuristicParams {
    params: [Field; PARAMS],
}

impl HeuristicParams {
    fn new(params: [Field]) -> Self {
        let mut inner = [0; PARAMS];
        for i in 0..params.len() {
            inner[i] = params[i];
        }
        Self {
            params: inner 
        }
    }
}

fn main(pubkey: pub [u8; 32], requested: Field, params: [HeuristicParams; HEURISTICS]) {
    assert(pubkey.len() == 32);

    let reg = HeuristicRegistry::new();

    for i in 0..HEURISTICS {
        let applied = reg.apply(i, params[i].params, requested);
        println(applied);
        assert(applied);
    }

}


#[test]
fn test_simple() {
    let pubkey = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
    let requested_amount = 100;
    let params = HeuristicParams::new([25]);
    main(pubkey, requested_amount, [params]);
}

#[test]
fn test_simple_fail() {
    let pubkey = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31];
    let requested_amount = 101;
    let params = HeuristicParams::new([25]);
    main(pubkey, requested_amount, [params]);
}

