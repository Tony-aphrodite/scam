use k256::ecdsa::RecoveryId;
use crate::{error::{DecodeError, SignatureError}, types::U256};

/// ESDCA Signature
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct Signature {
    pub y_parity: bool,
    pub r: U256,
    pub s: U256,
}

impl Signature {

    pub const fn raw_len() -> usize {
        1 + 32 + 32
    }

    pub fn dummy() -> Self {
        Self::default()
    }

    pub fn y_parity(&self) -> bool {
        self.y_parity
    }
    pub fn from_sig(signature: k256::ecdsa::Signature, recid: RecoveryId) -> Self {
        let r: [u8; 32] = signature.r().to_bytes().into();
        let s: [u8; 32] = signature.s().to_bytes().into();

        let r = U256::from_be_bytes(r);
        let s = U256::from_be_bytes(s);
        let y_parity = if recid.to_byte() != 0 { true } else { false };

        Self { y_parity, r, s }
    }

    pub fn as_bytes(&self) -> [u8; 65] {
        let mut sig = [0u8; 65];
        sig[..32].copy_from_slice(&self.r.to_be_bytes::<32>());
        sig[32..64].copy_from_slice(&self.s.to_be_bytes::<32>());
        sig[64] = self.y_parity as u8;
        sig
    }

    pub fn raw_decode(bytes: &[u8; 65]) -> Result<Self, DecodeError> {
        // Binding front array except the last one in byets
        let [bytes @ .., v] = bytes;
        let v = *v as u64;
        let parity = match v {
            0 => false,
            1 => true,
            _ => return Err(DecodeError::InvalidSignature(SignatureError::InvalidParity(v))),
        };
        Ok(Self::from_bytes_and_parity(bytes, parity))
    }

    pub fn from_bytes_and_parity(bytes: &[u8], parity: bool) -> Self {
        let mut r_arr = [0u8; 32];
        let mut s_arr = [0u8; 32];

        let (r_bytes, s_bytes) = bytes[..64].split_at(32);
        r_arr.copy_from_slice(r_bytes);
        s_arr.copy_from_slice(s_bytes);

        let r = U256::from_be_bytes(r_arr);
        let s = U256::from_be_bytes(s_arr);
        Self {
            y_parity: parity,
            r,
            s,
        }
    }
}

impl Into<k256::ecdsa::Signature> for Signature {
    fn into(self) -> k256::ecdsa::Signature {
        let r_bytes: [u8; 32] = self.r.to_be_bytes();
        let s_bytes: [u8; 32] = self.s.to_be_bytes();

        let mut sig_bytes: [u8; 64] = [0u8; 64];
        sig_bytes[0..32].copy_from_slice(&r_bytes);
        sig_bytes[32..64].copy_from_slice(&s_bytes);

        let sig = k256::ecdsa::Signature::from_slice(&sig_bytes).unwrap();
        sig
    }
}