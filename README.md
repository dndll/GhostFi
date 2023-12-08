# 💸 GhostFi 💸

[GhostFi is on BOS!](https://test.near.org/ghostfi.testnet/widget/GhostFi)

[Contract](https://testnet.nearblocks.io/address/ghostfi.testnet)

GhostFi is an untrusted lending platform where you prove, without interaction with the lender, that you can meet lending criteria set by the lender.

## Why

A significant problem with traditional finance lending is that it requires providing much need-to-know information to many parties.
Each party introduces some risk of stolen identity, losing data and potentially even blackmail.
An additional risk that lenders in traditional finance use is centralised credit databases, which contain decades of financial information about the entire population; in some cases, these stores and their respective algorithms are entirely "magical."
and sit behind extremely closed doors to the general public. 

The goal of this protocol is to reduce the trust in actors while maintaining regulation and protection for all parties; this improves confidence from the borrower's perspective. 

This problem is addressed mainly by making use of on-chain financial data, but that introduces a new issue: how do we get data that lives off-chain into this system in a risk-free way?
The risks here include:
- privacy: if the data is publicly available, we move the problem downstream
- security: the data that is made publicly available needs to be verified by someone. Who is trusted to verify this data?
- trust: is the data even accurate? If not, how can we react to fraud?

What if we had a system where the owner of the documents could prove they owned the data and that they hadn't modified or doctored it? 
If the documents are authentic, what if the system could reveal them to the lender without exposing them?

The documents in question might be:
- identity documents: passport, driver's license and other forms of ID. If you have this information and it is provable, can we prove this in our day-to-day lives
without actually carrying our precious identity documents everywhere?
- financial documents: bank statements, credit card statements, 401k and pension. If these documents can be provable, we can send these around everywhere and immediately
get the best lending criteria and competitive loans.
- proof of assets: gold certificates, stocks, vehicles or houses. 

With this in mind, a lender can determine an algorithm based on arbitrary heuristics, using the same methodologies they do today and immediately release the funds to the borrower in real-time.
This approach cuts costs for the lender and reduces the risk of identity fraud.

## How does it work?

### Registration

First, a user needs to be able to register with the lender. 
This process can use a previously proven identity. It can be generalised to a proof-of-identity protocol, much like lenders do with Equifax today. 
The difference is that this registration information is hidden from the public eye.
The lender can even require additional registration criteria on top of the original proof with recursive proofs.
This information, all that needs to be revealed to the public, is a pseudonymous Account ID and, if the lender is fulfilling the loan on the blockchain, the funds. However, transferring the funds privately on the blockchain is relatively straightforward, and protocols exist today.

### Credit application

The user provides information locally to a proof protocol, which applies the lender's lending criteria for the user to build a Zero Knowledge Proof.
The user can optionally pre-verify this proof to determine if they can meet the lending criteria immediately. 
The user can build many proofs with different heuristics and select the best proof with the most appealing interest rate; this does not affect their credit score.
If the lending criteria have stayed the same since the user requested the proof, the lender will accept the proof and release the funds.

> For example, a college student with a part-time job might only be able to loan minimal funds at an extremely high-interest rate. 
If they then provide countersignatures from family, their bank, the student loan council and even their employer, they could meet lending criteria for more funds at a lower rate.
The lender could even introduce bonuses in real time for using a specific countersignatory they trust, adding more to the dynamic lending criteria.

The lender trusts the lending criteria they create. If the lending criteria change, the lender can issue a new circuit release and update their verifiers.
Automatically, all previously verified proofs will fail. This problem then becomes an existing problem, releasing packages. 

On top of this, a lender can compose lending criteria from many other lenders that specialise in specific verification. 
> Perhaps a lender based in the UAE would like to be able to lend to the citizens of Switzerland but does not understand particular Swiss Law. They can offload the verification to a lending provider specialising in Switzerland
and trust that the authorities and legal system will be able to recover if things go wrong.

How do we prove easily fakable documents?

## What can we do with this

Benefit from additional heuristics like social information in lending criteria; this is extremely powerful with social data aggregation and can introduce extremely bespoke lending criteria.

# State of development 

As part of the NEAR ecosystem hackathon '23, we wanted to prove the concept of revealing minimal information in an untrusted way.

## V0

There are quite a few moving parts in this project. We could've easily focused heavily on Identity Verification in ZKPs, which would've been a product on its own. 
We also could've focused heavily on the document verification and fingerprinting process so you can feed arbitrary documents into a circuit, and we can extract them on the fly.
Additionally, we could've focused on integration with an on-chain Verifier contract, but Verification requires a lot of gas, for ethereum a hyperplonk verification is ~500k, which would be a limitation from NEAR's side.

Rather than this, We focused on proving that we can reveal minimal information in an untrusted way with some on-chain registration criteria and fulfil the release of a loan.

The criteria for registration are based on social information:
    - [near.social](https://test.near.org): do you have an avatar? have you made a post? have you set a username?
    - have you actually "registered" on the protocol?: this is just a call to `register` on the [contract](https://testnet.nearblocks.io/address/ghostfi.testnet) 

The zero-knowledge circuit is opaque over the native finite field of the proving system, `BN254`, and has the following heuristics:
- fourx: this is a simple protocol that verifies that the provided balance is at least 4x the requested amount
- lender: this is a sample countersignatory that gives the lender the ability to bypass the verification with a signature

We also implemented a toy extraction of passport information using OCR in the `registrar` crate. The goal was to be able to feed the MRZ(Machine Readable Zone) from travel documents
into the circuit to add additional lending heuristics.

## Troubles

We used [noir](todo) for the proving system. Noir is a simple rust-like DSL that targets application developers to simplify creating ZK-proving systems. 
Since it's a hackathon, we'd be able to get going faster and focus on implementing as many features as possible without being bogged down too much with more heavyweight frameworks.

Unfortunately, we ran into a lot of non-trivial issues with the proving system itself that are not unique to us:

### Ed25519 signatures

We partially implemented Ed25519 signature verification in the protocol, with the general goal being to pass information to NEAR protocol and have more assurance that the proof could not be stolen and reused by anybody. 
We ran into issues with the underlying VM encountering memory index overflows, as others have had with [Bls12](https://github.com/noir-lang/noir/issues/3380) and [BN254](https://github.com/onurinanc/noir-bn254).
This meant that initially, proving time failed after ~10mins with overflows. There was a patch hotfix, which meant we could fail fast but never got past the VM memory overflows.

### Unconstrained functions

Noir removed `comptime` checks from the prover, which requires that specific fields are compiled into the circuit at proving time.
The ecosystem has yet to quite catch up to this, and some patches must be made to get things to pass. 
For example, with bigint, we had to introduce a change in `from_be_bytes` to make it an unconstrained function.

Another issue we had with comptime checking is the behaviour between VM calls and dynamic types, such as slices.
There are lagging standard library calls that have not yet been adapted to this change.

#### Sha512
https://github.com/noir-lang/noir/issues/3687


### Blackbox calls weirdness

The last issue we met during ECDSA signature verification was that we had to pass the signature parts to be able to verify the signature.
The complexity arises because the signature can only be passed raw as a field to the VM, with additional complexity in prover arguments. After all, the underlying field is only 32 bytes at most.
We could chunk the parts and join them in the circuit to verify the signature. However, there is some semantic difference between passing a constant array over an array built in the circuit.
We tried various magic tricks of verifying the joined array with the constant array and confirmed in the circuit that they are the same; however, the underlying VM behaves differently with this variable.

Even more interesting was that the signature was verified in the output. According to the print statements, when the `main` function terminates, we see the VM panic.V1

## Technical debt

All in all, if we were to start from the beginning of the week, it would've been much smarter to use lambdaworks/arkworks for the proving system, as we would've had slightly longer time setting up, but ultimately more options
to introduce more features, and more complex proofs, easily integrating additional components.

We would remove Prover/Verifier coupling, this decision was made due to time constraints but in practice, these components would be decoupled.

Small hacks for the purpose of demo, such as looking up access keys to check public key parity.

Fix Ed25519, although if the system was rewritten in Lambdaworks, it would've been simpler.

# Next steps

## V1

If there was additional work made on this system, it would be around:
- registration proofs:
    - passport signatures from ePassports, passing this information to the circuit and proving that would immediately prove a real user
    - enables much richer heuristic requirements, such as lending to only specific countries
- circuit calculated APY
- public keys, not account ids
- real countersignatures from many parties

## V2
- self updating system, the prover should be able to update itself from a bytecode release by the lender
- onchain verifier, this is a given, but
- local prover


# Getting started

The BOS widget is here: [GhostFi is on BOS!](https://test.near.org/ghostfi.testnet/widget/GhostFi)

You'll need to setup your backend first. If you're a nix user, you should be able to run `direnv allow .` and be able to compile the prover immediately.

If not, you can take dependency inspiration from `flake.nix`.

Then you'd need to deploy your contract, setup the confic file and then input the URL in the BOS component.
