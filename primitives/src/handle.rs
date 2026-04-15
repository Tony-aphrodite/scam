use std::{
    fmt::Debug,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use alloy_primitives::B256;
use colored::Colorize;
use tracing::info;

use crate::{
    block::{Block, Header, Payload, PayloadHeader},
    error::DecodeError,
    transaction::{Recovered, SignedTransaction, Tx},
    types::BlockHash,
};

pub trait Handle: Send + Sync + std::fmt::Debug {
    type Msg: Send + Sync;

    fn send(&self, msg: Self::Msg);
}

#[derive(Debug)]
pub enum NetworkHandleMessage {
    PeerConnectionTest,
    NewTransaction(SignedTransaction),
    NewPayload(Block),
    BroadcastBlock(Block),
    RequestDataResponse(u64, IpAddr, u16),
    RequestData(u64),
    RequestDataResponseFinished,
    HandShake(u64, IpAddr, u16),
    Hello(u64, IpAddr, u16),
    RemovePeer(u64),
    BroadcastTransaction(SignedTransaction),
    ReorgChainData,
    RequestChainData(IpAddr, u16),
    RespondChainDataResult(u64, Vec<BlockHash>),
    Ping(IpAddr, u16),
    Pong(IpAddr, u16),
    RemoveUnresponsivePeer(u64),
}

impl NetworkHandleMessage {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            Self::PeerConnectionTest => {
                let msg_type = 0x01 as u8;
                let protocol_version = 0x00 as u8;
                let payload_length = 0x00 as u64;

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw
            }
            Self::NewTransaction(signed) => {
                let msg_type = 0x02 as u8;
                let protocol_version = 0x00 as u8;
                let mut data = signed.encode();
                let payload_length = data.len() as u64;

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.append(&mut data);
                raw
            }
            Self::NewPayload(block) => {
                let msg_type = 0x03 as u8;
                let protocol_version = 0x00 as u8;
                let mut data = block.encode_ref();
                let payload_length = data.len() as u64;

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.append(&mut data);
                raw
            }
            // Internal Msg
            Self::BroadcastBlock(_block) => {
                let raw = Vec::new();
                raw
            }
            Self::RequestDataResponse(from, ip_addr, port) => {
                let msg_type = 0x04 as u8;
                let protocol_version = 0x00 as u8;
                let from = from.to_be_bytes();
                let mut ip_addr = match ip_addr {
                    IpAddr::V4(v4) => v4.octets().to_vec(),
                    IpAddr::V6(v6) => v6.octets().to_vec(),
                };
                let mut port = port.to_be_bytes().to_vec();
                let payload_length = (from.len() + ip_addr.len() + port.len()) as u64;

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.extend_from_slice(&from);
                raw.append(&mut ip_addr);
                raw.append(&mut port);
                // ??? why should it be here ???
                //dbg!(raw.len());
                raw
            }
            Self::RequestData(from) => {
                let msg_type = 0x05 as u8;
                let protocol_version = 0x00 as u8;
                let data = from.to_be_bytes();
                let payload_length = data.len() as u64;
                let mut raw = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.extend_from_slice(&data);
                raw
            }
            Self::RequestDataResponseFinished => {
                let msg_type = 0x06 as u8;
                let protocol_version = 0x00 as u8;
                let payload_length = 0x00 as u64;
                let mut raw = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw
            }
            Self::HandShake(pid, ip_addr, port) => {
                let msg_type = 0x07 as u8;
                let protocol_version = 0x00 as u8;
                let pid = pid.to_be_bytes();
                let mut ip_addr = match ip_addr {
                    IpAddr::V4(v4) => v4.octets().to_vec(),
                    IpAddr::V6(v6) => v6.octets().to_vec(),
                };
                let port = port.to_be_bytes();
                let payload_length = (pid.len() + ip_addr.len() + port.len()) as u64;

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.extend_from_slice(&pid);
                raw.append(&mut ip_addr);
                raw.extend_from_slice(&port);
                raw
            }
            Self::Hello(pid, ip_addr, port) => {
                let msg_type = 0x08 as u8;
                let protocol_version = 0x00 as u8;
                let pid = pid.to_be_bytes();
                let mut ip_addr = match ip_addr {
                    IpAddr::V4(v4) => v4.octets().to_vec(),
                    IpAddr::V6(v6) => v6.octets().to_vec(),
                };
                let port = port.to_be_bytes();
                let payload_length = (pid.len() + ip_addr.len() + port.len()) as u64;

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.extend_from_slice(&pid);
                raw.append(&mut ip_addr);
                raw.extend_from_slice(&port);
                raw
            }
            // Internal Msg
            Self::RemovePeer(_pid) => {
                let raw = Vec::new();
                raw
            }
            // Internal Msg
            Self::BroadcastTransaction(_signed) => {
                let raw = Vec::new();
                raw
            }
            // Internal Msg
            Self::ReorgChainData => {
                let raw = Vec::new();
                raw
            }
            Self::RequestChainData(ip_addr, port) => {
                let msg_type = 0x09 as u8;
                let protocol_version = 0x00 as u8;
                let mut ip_addr = match ip_addr {
                    IpAddr::V4(v4) => v4.octets().to_vec(),
                    IpAddr::V6(v6) => v6.octets().to_vec(),
                };
                let port = port.to_be_bytes();
                let payload_length = (ip_addr.len() + port.len()) as u64;

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.append(&mut ip_addr);
                raw.extend_from_slice(&port);
                raw
            }
            Self::RespondChainDataResult(len, vec) => {
                let msg_type = 0x10 as u8;
                let protocol_version = 0x00 as u8;
                let payload_length = (32 * (*len) as u64) + 8;

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.extend_from_slice(&len.to_be_bytes());

                for hash in vec.iter() {
                    raw.extend_from_slice(&hash.0.to_vec());
                }

                raw
            }
            Self::Ping(ip_addr, port) => {
                let msg_type = 0x11 as u8;
                let protocol_version = 0x00 as u8;
                let mut ip_addr = match ip_addr {
                    IpAddr::V4(v4) => v4.octets().to_vec(),
                    IpAddr::V6(v6) => v6.octets().to_vec(),
                };
                let port = port.to_be_bytes();
                let payload_length = (ip_addr.len() + port.len()) as u64;

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.append(&mut ip_addr);
                raw.extend_from_slice(&port);
                raw
            }
            Self::Pong(ip_addr, port) => {
                let msg_type = 0x12 as u8;
                let protocol_version = 0x00 as u8;
                let mut ip_addr = match ip_addr {
                    IpAddr::V4(v4) => v4.octets().to_vec(),
                    IpAddr::V6(v6) => v6.octets().to_vec(),
                };
                let port = port.to_be_bytes();
                let payload_length = (ip_addr.len() + port.len()) as u64;

                let mut raw: Vec<u8> = vec![msg_type, protocol_version];
                raw.extend_from_slice(&payload_length.to_be_bytes());
                raw.append(&mut ip_addr);
                raw.extend_from_slice(&port);
                raw
            }
            // Internal Msg
            Self::RemoveUnresponsivePeer(_pid) => {
                let raw = Vec::new();
                raw
            }
        }
    }

    // First Byte: Message Type
    // Second Byte: Payload Length
    // Third Byte: Protocol Version
    // remains: Data
    pub fn decode(
        buf: &[u8],
        _addr: SocketAddr,
    ) -> Result<(Option<NetworkHandleMessage>, usize), DecodeError> {
        if buf.len() < 3 {
            return Ok((None, buf.len()));
        }

        let msg_type = buf[0];
        let protocol_version = buf[1];
        let mut payload_len_raw = [0u8; 8];
        payload_len_raw.copy_from_slice(&buf[2..10]);
        let payload_length = usize::from_be_bytes(payload_len_raw) as u64;

        if (buf.len() as u64) < (10 + payload_length) {
            return Ok((None, buf.len()));
        }

        if protocol_version > 0 {
            info!("Not proper protocol version.");
            return Ok((None, buf.len()));
        }

        let data = &buf[10..];
        let mut buf_used = 10;
        match msg_type {
            // PeerConnectionTest
            0x01 => Ok((Some(NetworkHandleMessage::PeerConnectionTest), buf_used)),
            // NewTransaction
            0x02 => {
                let (signed, _) = SignedTransaction::decode(&data.to_vec())?;
                Ok((Some(NetworkHandleMessage::NewTransaction(signed)), buf_used))
            }
            // NewPayload
            0x03 => {
                let (block, used) = Block::decode(&data.to_vec())?;
                buf_used += used;
                Ok((Some(NetworkHandleMessage::NewPayload(block)), buf_used))
            }
            // RequestDataResponse
            0x04 => {
                if data.len() < 14 {
                    return Err(DecodeError::TooShortRawData(buf.to_vec()));
                }
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&data[0..8]);
                let from = u64::from_be_bytes(arr);
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&data[8..12]);
                let ip_addr =
                    IpAddr::V4(Ipv4Addr::from(u32::from_be_bytes(arr.try_into().unwrap())));
                let mut arr2 = [0u8; 2];
                arr2.copy_from_slice(&data[12..14]);
                let port = u16::from_be_bytes(arr2.try_into().unwrap());
                buf_used += 14;
                Ok((
                    Some(NetworkHandleMessage::RequestDataResponse(
                        from, ip_addr, port,
                    )),
                    buf_used,
                ))
            }

            // Handshake
            0x07 => {
                if data.len() < 14 {
                    return Err(DecodeError::TooShortRawData(buf.to_vec()));
                }
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&data[0..8]);
                let pid = usize::from_be_bytes(arr);
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&data[8..12]);
                let ip_addr =
                    IpAddr::V4(Ipv4Addr::from(u32::from_be_bytes(arr.try_into().unwrap())));
                let mut arr2 = [0u8; 2];
                arr2.copy_from_slice(&data[12..14]);
                let port = u16::from_be_bytes(arr2.try_into().unwrap());
                buf_used += 14;
                Ok((
                    Some(NetworkHandleMessage::HandShake(pid as u64, ip_addr, port)),
                    buf_used,
                ))
            }
            // Hello
            0x08 => {
                if data.len() < 14 {
                    return Err(DecodeError::TooShortRawData(buf.to_vec()));
                }
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&data[0..8]);
                let pid = usize::from_be_bytes(arr);
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&data[8..12]);
                let ip_addr =
                    IpAddr::V4(Ipv4Addr::from(u32::from_be_bytes(arr.try_into().unwrap())));
                let mut arr2 = [0u8; 2];
                arr2.copy_from_slice(&data[12..14]);
                let port = u16::from_be_bytes(arr2.try_into().unwrap());
                buf_used += 14;
                Ok((
                    Some(NetworkHandleMessage::Hello(pid as u64, ip_addr, port)),
                    buf_used,
                ))
            }
            0x09 => {
                if data.len() < 6 {
                    return Err(DecodeError::TooShortRawData(buf.to_vec()));
                }
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&data[0..4]);
                let ip_addr =
                    IpAddr::V4(Ipv4Addr::from(u32::from_be_bytes(arr.try_into().unwrap())));
                let mut arr2 = [0u8; 2];
                arr2.copy_from_slice(&data[4..6]);
                let port = u16::from_be_bytes(arr2.try_into().unwrap());
                buf_used += 6;
                Ok((
                    Some(NetworkHandleMessage::RequestChainData(ip_addr, port)),
                    buf_used,
                ))
            }
            0x10 => {
                let mut arr = [0u8; 8];
                arr.copy_from_slice(&data[0..8]);
                let len = u64::from_be_bytes(arr);
                buf_used += 8;

                let mut hash_vec: Vec<BlockHash> = Vec::new();
                for i in 0..len {
                    let start: usize = 8 + i as usize * 32;
                    let block_hash = B256::from_slice(&data[start..start + 32]);
                    hash_vec.push(BlockHash::from(block_hash));
                    buf_used += 32;
                }

                Ok((
                    Some(NetworkHandleMessage::RespondChainDataResult(len, hash_vec)),
                    buf_used,
                ))
            }
            0x11 => {
                if data.len() < 6 {
                    return Err(DecodeError::TooShortRawData(buf.to_vec()));
                }
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&data[0..4]);
                let ip_addr =
                    IpAddr::V4(Ipv4Addr::from(u32::from_be_bytes(arr.try_into().unwrap())));
                let mut arr2 = [0u8; 2];
                arr2.copy_from_slice(&data[4..6]);
                let port = u16::from_be_bytes(arr2.try_into().unwrap());
                buf_used += 6;
                Ok((Some(NetworkHandleMessage::Ping(ip_addr, port)), buf_used))
            }
            0x12 => {
                if data.len() < 6 {
                    return Err(DecodeError::TooShortRawData(buf.to_vec()));
                }
                let mut arr = [0u8; 4];
                arr.copy_from_slice(&data[0..4]);
                let ip_addr =
                    IpAddr::V4(Ipv4Addr::from(u32::from_be_bytes(arr.try_into().unwrap())));
                let mut arr2 = [0u8; 2];
                arr2.copy_from_slice(&data[4..6]);
                let port = u16::from_be_bytes(arr2.try_into().unwrap());
                buf_used += 6;
                Ok((Some(NetworkHandleMessage::Pong(ip_addr, port)), buf_used))
            }
            _ => Ok((None, buf.len())),
        }
    }
}

use std::fmt;

impl fmt::Display for NetworkHandleMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkHandleMessage::PeerConnectionTest => {
                write!(f, "{} {}", "[Network]", "PeerConnectionTest",)
            }
            NetworkHandleMessage::NewTransaction(tx) => {
                write!(
                    f,
                    "{} {} hash: {:?}",
                    "[Network]", "NewTransaction", tx.hash
                )
            }
            NetworkHandleMessage::NewPayload(block) => {
                write!(
                    f,
                    "{} {} height: {}, hash: {:?}",
                    "[Network]",
                    "NewPayload",
                    block.header.height.to_string(),
                    block.header().calculate_hash()
                )
            }
            NetworkHandleMessage::BroadcastBlock(block) => {
                write!(
                    f,
                    "{} {} height: {}, hash: {:?}",
                    "[Network]",
                    "BroadcastBlock",
                    block.header.height.to_string(),
                    block.header().calculate_hash()
                )
            }
            NetworkHandleMessage::RequestDataResponse(num, ip, port) => {
                write!(
                    f,
                    "{} {} block_no: {}, addr: {}:{}",
                    "[Network]",
                    "RequestDataResponse",
                    num.to_string(),
                    ip,
                    port
                )
            }
            NetworkHandleMessage::RequestData(num) => {
                write!(
                    f,
                    "{} {} block_no: {}",
                    "[Network]",
                    "RequestData",
                    num.to_string()
                )
            }
            NetworkHandleMessage::RequestDataResponseFinished => {
                write!(f, "{} {}", "[Network]", "RequestDataResponseFinished")
            }
            NetworkHandleMessage::HandShake(id, ip, port) => {
                write!(
                    f,
                    "{} {} id: {}, addr: {}:{}",
                    "[Network]",
                    "HandShake",
                    id.to_string(),
                    ip,
                    port
                )
            }
            NetworkHandleMessage::Hello(id, ip, port) => {
                write!(
                    f,
                    "{} {} id: {}, addr: {}:{}",
                    "[Network]",
                    "Hello",
                    id.to_string(),
                    ip,
                    port
                )
            }
            NetworkHandleMessage::RemovePeer(id) => {
                write!(f, "{} {} id: {}", "[Network]", "RemovePeer", id)
            }
            NetworkHandleMessage::RemoveUnresponsivePeer(id) => {
                write!(f, "{} {} id: {}", "[Network]", "RemoveUnresponsivePeer", id)
            }
            NetworkHandleMessage::BroadcastTransaction(tx) => {
                write!(
                    f,
                    "{} {} hash: {:?}",
                    "[Network]", "BroadcastTransaction", tx.hash
                )
            }
            NetworkHandleMessage::ReorgChainData => {
                write!(f, "{} {}", "[Network]", "ReorgChainData")
            }
            NetworkHandleMessage::RequestChainData(ip, port) => {
                write!(
                    f,
                    "{} {} addr: {}:{}",
                    "[Network]", "RequestChainData", ip, port
                )
            }
            NetworkHandleMessage::RespondChainDataResult(num, hashes) => {
                write!(
                    f,
                    "{} {} start_no: {}, {} hashes",
                    "[Network]",
                    "RespondChainDataResult",
                    num.to_string(),
                    hashes.len().to_string()
                )
            }
            NetworkHandleMessage::Ping(addr, port) => {
                write!(f, "{} {} {}:{}", "[Network]", "Ping from", addr, port)
            }
            NetworkHandleMessage::Pong(addr, port) => {
                write!(f, "{} {} {}:{}", "[Network]", "Pong from", addr, port)
            }
        }
    }
}

#[derive(Debug)]
pub enum ConsensusHandleMessage {
    ImportBlock(Block),
    NewTransaction(Recovered),
}

impl fmt::Display for ConsensusHandleMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConsensusHandleMessage::ImportBlock(block) => {
                write!(
                    f,
                    "{} {} height: {}, hash: {:?}",
                    "[Consensus]",
                    "ImportBlock",
                    block.header.height.to_string(),
                    block.header.calculate_hash()
                )
            }
            ConsensusHandleMessage::NewTransaction(tx) => {
                write!(
                    f,
                    "{} {} hash: {:?}, from: {:?}, to: {:?}, value: {}",
                    "[Consensus]",
                    "NewTransaction",
                    tx.hash(),
                    tx.signer().get_addr_hex(),
                    tx.tx().tx.to().get_addr_hex().bright_blue(),
                    tx.tx().tx.value().to_string()
                )
            }
        }
    }
}

#[derive(Debug)]
pub enum PayloadBuilderHandleMessage {
    BuildPayload,
    Stop,
}

impl fmt::Display for PayloadBuilderHandleMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PayloadBuilderHandleMessage::BuildPayload => {
                write!(f, "{} {}", "[PayloadBuilderHandle]", "BuildPayload")
            }
            PayloadBuilderHandleMessage::Stop => {
                write!(f, "{} {}", "[PayloadBuilderHandle]", "Stop")
            }
        }
    }
}

#[derive(Debug)]
pub enum PayloadBuilderResultMessage {
    Payload(Payload),
    PoolIsEmpty,
}

impl fmt::Display for PayloadBuilderResultMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PayloadBuilderResultMessage::Payload(payload) => {
                writeln!(f, "{} {}", "[PayloadBuilderResult]", "New Payload Built")?;
                write!(f, "{}", payload) // Payload에 Display 이미 구현되어 있다고 가정
            }
            PayloadBuilderResultMessage::PoolIsEmpty => {
                write!(
                    f,
                    "{} {}",
                    "[PayloadBuilderResult]", "Pool is empty, no payload created"
                )
            }
        }
    }
}

#[derive(Debug)]
pub enum MinerHandleMessage {
    NewPayload(PayloadHeader),
    HaltMining,
}

#[derive(Debug)]
pub enum MinerResultMessage {
    MiningSuccess(Header),
    MiningHalted,
}

impl fmt::Display for MinerHandleMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MinerHandleMessage::NewPayload(header) => {
                write!(
                    f,
                    "{} {} height: {}, prev_hash: {:?}, difficulty: {}, timestamp: {}",
                    "[MinerHandle]",
                    "NewPayload",
                    header.height.to_string(),
                    header.previous_hash,
                    header.difficulty.to_string(),
                    header.timestamp.to_string()
                )
            }
            MinerHandleMessage::HaltMining => {
                write!(f, "{} {}", "[MinerHandle]", "Halt Mining")
            }
        }
    }
}

impl fmt::Display for MinerResultMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MinerResultMessage::MiningSuccess(header) => {
                write!(
                    f,
                    "{} {} height: {}, hash: {:?}, difficulty: {}, timestamp: {}",
                    "[MinerResult]",
                    "MiningSuccess",
                    header.height.to_string(),
                    header.calculate_hash(),
                    header.difficulty.to_string(),
                    header.timestamp.to_string()
                )
            }
            MinerResultMessage::MiningHalted => {
                write!(f, "{} {}", "[MinerResult]", "MiningHalted")
            }
        }
    }
}
