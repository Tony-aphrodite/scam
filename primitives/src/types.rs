// This project use alloy_primitives in only this file.
pub use alloy_primitives::{B256, U256};
use libmdbx::orm::{Decodable, Encodable};
use rand::Rng;
use serde::Serialize;

use crate::error::AddressError;

pub type ChainId = u64;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TxHash(pub B256);

impl TxHash {
    pub fn hash(&self) -> B256 {
        self.0
    }
}

impl From<B256> for TxHash {
    fn from(value: B256) -> Self {
        Self(value)
    }
}

impl Encodable for TxHash {
    type Encoded = Vec<u8>;

    fn encode(self) -> Self::Encoded {
        self.hash().to_vec()
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct BlockHash(pub B256);
impl BlockHash {
    pub fn hash(&self) -> B256 {
        self.0
    }
}

impl From<B256> for BlockHash {
    fn from(value: B256) -> Self {
        Self(value)
    }
}

impl Encodable for BlockHash {
    type Encoded = Vec<u8>;

    fn encode(self) -> Self::Encoded {
        self.hash().to_vec()
    }
}

pub type PayloadId = u64;

const ADDR_LEN: usize = 20;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Address([u8; ADDR_LEN]);
pub const COINBASE_ADDR: Address = Address([0u8; 20]);

impl Address {
    pub fn min() -> Self {
        let addr = [0u8; 20];
        Self::from_byte(addr)
    }

    pub fn max() -> Self {
        let addr = [0xff; 20];
        Self::from_byte(addr)
    }

    pub fn from_byte(address: [u8; 20]) -> Self {
        Self(address)
    }

    pub fn from_hex(address: String) -> Result<Self, AddressError> {
        let bytes = hex::decode(address)?;
        if bytes.len() != ADDR_LEN {
            return Err(AddressError::InvalidLength(bytes.len()));
        }

        let arr: [u8; ADDR_LEN] = bytes.try_into().unwrap();
        Ok(Address(arr))
    }

    // This is for dev/test code
    pub fn random() -> Self {
        let mut arr = [0u8; 20];
        let mut rng = rand::rng();
        rng.fill(&mut arr);
        Self(arr)
    }

    pub fn get_addr_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn get_addr(&self) -> &[u8] {
        &self.0
    }
}

impl Default for Address {
    fn default() -> Self {
        COINBASE_ADDR
    }
}

impl Encodable for Address {
    type Encoded = Vec<u8>;

    fn encode(self) -> Self::Encoded {
        self.0.to_vec()
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Account {
    pub nonce: u64,
    pub balance: U256,
}

impl Account {
    pub fn new(nonce: u64, balance: U256) -> Self {
        Self { nonce, balance }
    }
    pub fn update(&mut self, nonce: u64, balance: U256) {
        self.nonce = nonce;
        self.balance = balance;
    }

    pub fn balance(&self) -> U256 {
        self.balance
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    pub fn sub_balance(&mut self, value: U256) {
        if value > self.balance {
            self.balance = U256::ZERO;
        } else {
            self.balance -= value;
        }
    }

    pub fn add_balance(&mut self, value: U256) {
        if self.balance > U256::MAX - value {
            self.balance = U256::MAX;
        } else {
            self.balance += value;
        }
    }

    pub fn increase_nonce(&mut self) {
        self.nonce += 1;
    }
}

impl Decodable for Account {
    fn decode(b: &[u8]) -> anyhow::Result<Self> {
        let mut raw = [0u8; 8];
        raw.copy_from_slice(&b[0..8]);
        let nonce = u64::from_be_bytes(raw);
        let balance = U256::from_be_slice(&b[8..40]);
        Ok(Account { nonce, balance })
    }
}

impl Encodable for Account {
    type Encoded = Vec<u8>;

    fn encode(self) -> Vec<u8> {
        let mut res = Vec::new();
        res.extend_from_slice(&self.nonce().to_be_bytes());
        res.extend_from_slice(&self.balance().to_be_bytes::<32>());
        res
    }
}
