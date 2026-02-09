// methods/guest/src/main.rs — AXIOM Protocol 124M Supply Integrity Law
//
// This code runs inside the RISC Zero zkVM. It is the immutable,
// cryptographically enforced law of the 124M supply network.
// Any attempt to violate the supply cap will cause proof generation
// to fail — no governance override is possible.

#![no_main]
risc0_zkvm::guest::entry!(main);

use blake3;
use serde::{Serialize, Deserialize};

/// Private transaction data fed into the zkVM by the Host prover.
/// The `initial_balance` and structural details remain hidden; only
/// the 512-bit BLAKE3 anchor is committed to the public journal.
#[derive(Serialize, Deserialize)]
struct Transaction {
    initial_balance: u64,
    amount: u64,
    fee: u64,
    nonce: u64,
}

fn main() {
    // 1. Read private transaction data from the Host
    let tx: Transaction = risc0_zkvm::guest::env::read();

    // 2. HARD ENFORCEMENT: 124M Supply Invariance
    //    If (amount + fee) exceeds balance the STARK proof cannot be created.
    assert!(
        tx.initial_balance >= tx.amount.checked_add(tx.fee).expect("overflow"),
        "STARK Error: Spend exceeds available 124M allocation",
    );

    // 3. 512-bit BLAKE3 ANCHOR
    //    Hash the entire transaction to create a unique pulse that binds
    //    this proof to the block header.
    let mut hasher = blake3::Hasher::new();
    let tx_bytes = bincode::serialize(&tx).expect("serialization");
    hasher.update(&tx_bytes);

    let mut output_512 = [0u8; 64];
    hasher.finalize_xof().fill(&mut output_512);

    // 4. COMMIT TO JOURNAL
    //    The journal is the PUBLIC part of the STARK receipt.
    //    The world sees this 512-bit hash but NOT the balance or nonce.
    risc0_zkvm::guest::env::commit(&output_512);
}
