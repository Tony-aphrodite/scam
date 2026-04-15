use alloy_primitives::B256;
use libmdbx::orm::{Decodable, Encodable};

#[derive(Debug, Clone)]
pub struct World {}

impl World {
    pub fn new() -> Self {
        Self {  }
    }

    pub fn calculate_hash(&self) -> B256 {
        B256::default()
    }

    pub fn encode(&self) -> Vec<u8> {
        Vec::with_capacity(32)
    }
}

impl Encodable for World {
    type Encoded = Vec<u8>;

    fn encode(self) -> Self::Encoded {
        let res = Vec::new();
        res
    }
}

impl Decodable for World {
    fn decode(_b: &[u8]) -> anyhow::Result<Self> {
        let res = World::new();
        Ok(res)
    }
}