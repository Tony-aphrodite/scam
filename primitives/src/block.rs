use std::fmt;

use alloy_primitives::U256;
use anyhow::bail;
use libmdbx::orm::{Decodable, Encodable};
use sha2::{Digest, Sha256};

use crate::error::{BlockValidatioError, DecodeError};
use crate::types::{Address, B256, COINBASE_ADDR};
use crate::{transaction::SignedTransaction, types::BlockHash};

/// Block hash
#[derive(Debug, Default, Clone)]
pub struct Header {
    pub previous_hash: BlockHash, // 32
    pub transaction_root: B256,   // 32
    pub state_root: B256,         // 32
    pub timestamp: u64,           // 8
    pub proposer: Address,        // 20
    pub nonce: u64,               // 8
    pub difficulty: u32,          // 4
    pub height: u64,              // 8
    pub total_fee: U256,          // 32
}

impl Header {
    pub fn genesis_header() -> Self {
        Self {
            previous_hash: Default::default(),
            transaction_root: Default::default(),
            state_root: Default::default(),
            timestamp: 0,
            proposer: COINBASE_ADDR,
            nonce: 0,
            difficulty: 20,
            height: 0,
            total_fee: U256::ZERO,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut raw = [0u8; 176];
        raw[0..32].copy_from_slice(&self.previous_hash.hash().0);
        raw[32..64].copy_from_slice(&self.transaction_root.0);
        raw[64..96].copy_from_slice(&self.state_root.0);
        raw[96..104].copy_from_slice(&self.timestamp.to_be_bytes());
        raw[104..124].copy_from_slice(&self.proposer.get_addr());
        raw[124..132].copy_from_slice(&self.nonce.to_be_bytes());
        raw[132..136].copy_from_slice(&self.difficulty.to_be_bytes());
        raw[136..144].copy_from_slice(&self.height.to_be_bytes());
        raw[144..176].copy_from_slice(&self.total_fee.to_be_bytes::<32>());
        raw.to_vec()
    }

    pub fn decode(buf: [u8; 176]) -> Self {
        Self {
            previous_hash: BlockHash::from(B256::from_slice(&buf[0..32])),
            transaction_root: B256::from_slice(&buf[32..64]),
            state_root: B256::from_slice(&buf[64..96]),
            timestamp: u64::from_be_bytes(buf[96..104].try_into().unwrap()),
            proposer: Address::from_byte(buf[104..124].try_into().unwrap()),
            nonce: u64::from_be_bytes(buf[124..132].try_into().unwrap()),
            difficulty: u32::from_be_bytes(buf[132..136].try_into().unwrap()),
            height: u64::from_be_bytes(buf[136..144].try_into().unwrap()),
            total_fee: U256::from_be_slice(&buf[144..176]),
        }
    }

    pub fn calculate_hash(&self) -> BlockHash {
        let mut hasher = Sha256::new();
        hasher.update(self.previous_hash.hash());
        hasher.update(self.transaction_root);
        hasher.update(self.state_root);
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(self.proposer.get_addr());
        hasher.update(self.difficulty.to_be_bytes());
        hasher.update(self.height.to_be_bytes());
        hasher.update(self.nonce.to_be_bytes());
        hasher.update(self.total_fee.to_be_bytes::<32>());
        BlockHash::from(B256::from_slice(&hasher.finalize()))
    }
}

#[derive(Debug, Clone)]
/// Block Structure
pub struct Block {
    pub header: Header,
    pub body: Vec<SignedTransaction>,
}

impl Block {
    pub fn genesis_block() -> Self {
        let header = Header::genesis_header();
        Self {
            header,
            body: Vec::new(),
        }
    }

    pub fn encode_ref(&self) -> Vec<u8> {
        let mut res: Vec<u8> = Vec::new();
        let header = self.header.encode();
        res = [res, header].concat();

        // Recoverd -> SignedTransaction
        for recovered in self.body.iter() {
            let encoded = recovered.encode();
            res = [res, encoded].concat();
        }
        res
    }

    pub fn decode(buf: &[u8]) -> Result<(Self, usize), DecodeError> {
        let mut used_byte = 0;
        if buf.len() < 176 {
            return Err(DecodeError::TooShortRawData(buf.to_vec()));
        }

        let (header_raw, mut body_raw) = buf.split_at(176);
        used_byte += 176;
        let header = Header::decode(header_raw.try_into().unwrap());
        let mut body = Vec::new();

        while body_raw.len() >= 149 {
            let (tx_raw, remains) = body_raw.split_at(149);
            used_byte += 149;
            let (signed, _) = SignedTransaction::decode(&tx_raw.to_vec()).unwrap();
            body_raw = remains;
            body.push(signed);
        }

        Ok((Self { header, body }, used_byte))
    }

    pub fn header(&self) -> &Header {
        &self.header
    }
}

impl Encodable for Block {
    type Encoded = Vec<u8>;

    fn encode(self) -> Self::Encoded {
        let mut res: Vec<u8> = Vec::new();
        let header = self.header.encode();
        res = [res, header].concat();

        // Recoverd -> SignedTransaction
        for recovered in self.body.iter() {
            let encoded = recovered.encode();
            res = [res, encoded].concat();
        }
        res
    }
}

impl Decodable for Block {
    fn decode(b: &[u8]) -> anyhow::Result<Self> {
        if b.len() < 176 {
            bail!("Too short raw data: {} bytes", b.len());
        }

        let (header_raw, mut body_raw) = b.split_at(176);
        let header = Header::decode(header_raw.try_into().unwrap());
        let mut body = Vec::new();

        while body_raw.len() >= 149 {
            let (tx_raw, remains) = body_raw.split_at(149);
            let (signed, _) = SignedTransaction::decode(&tx_raw.to_vec()).unwrap();
            body_raw = remains;
            body.push(signed);
        }

        Ok(Self { header, body })
    }
}

/// Block hash
#[derive(Debug, Default, Clone)]
pub struct PayloadHeader {
    pub previous_hash: BlockHash,
    pub transaction_root: B256,
    pub state_root: B256,
    pub proposer: Address,
    pub difficulty: u32,
    pub timestamp: u64,
    pub height: u64,
    pub total_fee: U256,
}

impl PayloadHeader {
    pub fn into_header(self, nonce: u64) -> Header {
        Header {
            previous_hash: self.previous_hash,
            transaction_root: self.transaction_root,
            state_root: self.state_root,
            timestamp: self.timestamp,
            proposer: self.proposer,
            nonce,
            difficulty: self.difficulty,
            height: self.height,
            total_fee: self.total_fee,
        }
    }
}

#[derive(Debug, Clone)]
/// Payload Structure (Before Mining)
pub struct Payload {
    pub header: PayloadHeader,
    pub body: Vec<SignedTransaction>,
}

impl fmt::Display for Payload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Payload {{")?;
        writeln!(f, "  Header:")?;
        writeln!(f, "    Previous Hash: {:?}", self.header.previous_hash)?;
        writeln!(f, "    Tx Root      : {:?}", self.header.transaction_root)?;
        writeln!(f, "    State Root   : {:?}", self.header.state_root)?;
        writeln!(f, "    Proposer     : {:?}", self.header.proposer)?;
        writeln!(f, "    Difficulty   : {}", self.header.difficulty)?;
        writeln!(f, "    Timestamp    : {}", self.header.timestamp)?;
        writeln!(f, "    Height       : {}", self.header.height)?;
        writeln!(f, "    Total Fee    : {}", self.header.total_fee)?;
        writeln!(f, "  Body ({} txs):", self.body.len())?;
        for (i, tx) in self.body.iter().enumerate() {
            writeln!(
                f,
                "    {}. hash: {:?}, to: {:?}, value: {}, fee: {}",
                i + 1,
                tx.hash,
                tx.tx.to,
                tx.tx.value,
                tx.tx.fee
            )?;
        }
        write!(f, "}}")
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Block {{")?;
        writeln!(f, "  Header:")?;
        writeln!(f, "    Previous Hash: {:?}", self.header.previous_hash)?;
        writeln!(f, "    Tx Root      : {:?}", self.header.transaction_root)?;
        writeln!(f, "    State Root   : {:?}", self.header.state_root)?;
        writeln!(f, "    Proposer     : {:?}", self.header.proposer)?;
        writeln!(f, "    Difficulty   : {}", self.header.difficulty)?;
        writeln!(f, "    Timestamp    : {}", self.header.timestamp)?;
        writeln!(f, "    Height       : {}", self.header.height)?;
        writeln!(f, "    Total Fee    : {}", self.header.total_fee)?;
        writeln!(f, "  Body ({} txs):", self.body.len())?;

        for (i, tx) in self.body.iter().enumerate() {
            writeln!(
                f,
                "    {}. hash: {:?}, to: {:?}, value: {}, fee: {}",
                i + 1,
                tx.hash,
                tx.tx.to,
                tx.tx.value,
                tx.tx.fee
            )?;
        }

        write!(f, "}}")
    }
}

pub struct BlockValidationResult {
    pub success: bool,
    pub error: Option<BlockValidatioError>,
}

impl BlockValidationResult {
    pub fn success(&mut self) {
        self.success = true;
    }

    pub fn failed(&mut self) {
        self.success = false;
    }

    pub fn add_error(&mut self, e: BlockValidatioError) {
        self.error = Some(e)
    }
}

impl Default for BlockValidationResult {
    fn default() -> Self {
        Self {
            success: false,
            error: Some(BlockValidatioError::DefaultError),
        }
    }
}
