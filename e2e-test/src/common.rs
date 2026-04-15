use k256::{
    EncodedPoint,
    ecdsa::{RecoveryId, Signature as ECDSASig, SigningKey},
};
use primitives::{
    signature::Signature,
    transaction::{SignedTransaction, Transaction},
    types::Address,
};
use sha2::{Digest, Sha256};

pub fn create_key_pairs(seed: &[u8]) -> (SigningKey, Address) {
    let private_key_random = Sha256::digest(&seed);
    let signing_key = SigningKey::from_bytes(&private_key_random).unwrap();

    let verifying_key = signing_key.clone().verifying_key().clone();
    let pubkey_uncompressed: EncodedPoint = verifying_key.to_encoded_point(false);
    let pubkey_bytes = pubkey_uncompressed.as_bytes();
    let address = pubkey_bytes[pubkey_bytes.len() - 20..].to_vec();
    let address = Address::from_byte(address.try_into().unwrap());
    (signing_key, address)
}

pub fn create_signed(signing_key: &SigningKey, tx: Transaction) -> SignedTransaction {
    let tx_hash = tx.encode_for_signing();
    let digest = Sha256::new_with_prefix(tx_hash.hash());
    let (sig, recid): (ECDSASig, RecoveryId) = signing_key.sign_digest_recoverable(digest).unwrap();
    let signature = Signature::from_sig(sig, recid);
    SignedTransaction::new(tx, signature, tx_hash)
}
