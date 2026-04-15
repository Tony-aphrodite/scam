use std::sync::Arc;

use axum::{Json, extract::State};
use primitives::{
    handle::{ConsensusHandleMessage, NetworkHandleMessage},
    transaction::SignedTransaction,
    types::{Address, B256, TxHash},
};
use provider::DatabaseTrait;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tracing::error;
use transaction_pool::{error::PoolErrorKind, identifier::TransactionOrigin};

use crate::Node;

#[derive(Debug, Deserialize, Clone)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Vec<Value>,
    pub id: u64,
}

impl RpcRequest {
    pub fn noob() -> Self {
        Self {
            jsonrpc: "abc".to_string(),
            method: "test".to_string(),
            params: Vec::new(),
            id: 0,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    pub success: bool,
    pub result: Value,
    pub id: u64,
}

pub async fn rpc_handle<DB: DatabaseTrait>(
    State(node): State<Arc<Node<DB>>>,
    Json(req): Json<RpcRequest>,
) -> Json<RpcResponse> {
    let mut success = false;
    match req.method.as_str() {
        "chain_name" => {
            let result = json!("Pint");
            Json(RpcResponse {
                jsonrpc: "2.0".to_string(),
                success,
                result,
                id: req.id,
            })
        }
        "local_transaction" => {
            let result;
            if let Some(raw) = req.params[0].as_str() {
                let data = match hex::decode(raw) {
                    Ok(data) => data,
                    Err(_e) => {
                        return Json(RpcResponse {
                            jsonrpc: "2.0".to_string(),
                            success,
                            result: json!("Transaction Hex Decode Error"),
                            id: req.id,
                        });
                    }
                };
                let signed = match SignedTransaction::decode(&data) {
                    Ok((signed, _)) => signed,
                    Err(_e) => {
                        return Json(RpcResponse {
                            jsonrpc: "2.0".to_string(),
                            success,
                            result: json!("Transaction Decode Error"),
                            id: req.id,
                        });
                    }
                };
                let signed_tx = signed.clone();
                let origin = TransactionOrigin::Local;
                let recovered = match signed.into_recovered() {
                    Ok(recovered) => recovered,
                    Err(_e) => {
                        return Json(RpcResponse {
                            jsonrpc: "2.0".to_string(),
                            success,
                            result: json!("Transaction Recovery Error"),
                            id: req.id,
                        });
                    }
                };
                let recovered_cloned = recovered.clone();
                let tx_hash = match node.pool.add_transaction(origin, recovered) {
                    Ok(tx_hash) => tx_hash,
                    Err(e) => match e.kind {
                        PoolErrorKind::AlreadyImported => {
                            return Json(RpcResponse {
                                jsonrpc: "2.0".to_string(),
                                success,
                                result: json!("Transaction Pool Error: AlreadyImported"),
                                id: req.id,
                            });
                        }
                        PoolErrorKind::InvalidTransaction(_tx) => {
                            return Json(RpcResponse {
                                jsonrpc: "2.0".to_string(),
                                success,
                                result: json!("Transaction Pool Error: InvalidTransaction"),
                                id: req.id,
                            });
                        }
                        PoolErrorKind::RelpacementUnderpriced(_tx) => {
                            return Json(RpcResponse {
                                jsonrpc: "2.0".to_string(),
                                success,
                                result: json!("Transaction Pool Error: ReloacedUnderpriced"),
                                id: req.id,
                            });
                        }
                        PoolErrorKind::InvalidPoolTransactionError(_tx) => {
                            return Json(RpcResponse {
                                jsonrpc: "2.0".to_string(),
                                success,
                                result: json!(
                                    "Transaction Pool Error: InvalidPoolTransactionError"
                                ),
                                id: req.id,
                            });
                        }
                        PoolErrorKind::ImportError => {
                            return Json(RpcResponse {
                                jsonrpc: "2.0".to_string(),
                                success,
                                result: json!("Transaction Pool Error: ImportError"),
                                id: req.id,
                            });
                        }
                    },
                };

                // broadcast to peer!
                success = true;
                result = json!(tx_hash.hash().to_string());
                node.network
                    .send(NetworkHandleMessage::BroadcastTransaction(signed_tx));

                if node.pool.check_pending_pool_len() >= 1 {
                    node.consensus
                        .send(ConsensusHandleMessage::NewTransaction(recovered_cloned));
                }
            } else {
                result = json!("There is no new transaction");
            }
            Json(RpcResponse {
                jsonrpc: "2.0".to_string(),
                success,
                result,
                id: req.id,
            })
        }
        "account" => {
            let mut result = json!("Failed");
            if let Some(raw) = req.params[0].as_str() {
                let address = match Address::from_hex(raw.to_string()) {
                    Ok(addr) => addr,
                    Err(e) => {
                        error!(error = ?e, "Failed to get account info.");
                        result = json!("Wrong address");
                        return Json(RpcResponse {
                            jsonrpc: "2.0".to_string(),
                            success,
                            result,
                            id: req.id,
                        });
                    }
                };

                match node.provider.db().basic(&address) {
                    Ok(account) => {
                        result = match account {
                            Some(account) => {
                                let nonce = format!("{}", account.nonce());
                                let balance = format!("{}", account.balance());
                                json!({
                                    "nonce": nonce,
                                    "balance": balance
                                })
                            }
                            None => json!("No account info"),
                        };
                    }
                    Err(_e) => {}
                };
            }
            Json(RpcResponse {
                jsonrpc: "2.0".to_string(),
                success,
                result,
                id: req.id,
            })
        }
        "blockchain_height" => {
            let result = json!(node.provider.block_number());
            Json(RpcResponse {
                jsonrpc: "2.0".to_string(),
                success,
                result,
                id: req.id,
            })
        }
        "transaction" => {
            let mut result: Value = json!("There is no transaction you want to find.");
            if let Some(raw) = req.params[0].as_str() {
                let data = match hex::decode(raw) {
                    Ok(data) => data,
                    Err(_e) => {
                        return Json(RpcResponse {
                            jsonrpc: "2.0".to_string(),
                            success,
                            result: json!("Transaction Hex Decode Error"),
                            id: req.id,
                        });
                    }
                };

                let tx_hash: TxHash = TxHash::from(B256::from_slice(&data));
                match node.provider.db().get_transaction_by_hash(tx_hash) {
                    Ok(Some((tx, bno))) => {
                        let data = tx.encode();
                        result = json!({
                            "tx": hex::encode(data),
                            "block_number": bno
                        });
                    }
                    Ok(None) => {
                        result = json!("There is no transaction you want to find.");
                    }
                    Err(_e) => {
                        result = json!("Database Error. Try again");
                    }
                }
            }
            Json(RpcResponse {
                jsonrpc: "2.0".to_string(),
                success,
                result,
                id: req.id,
            })
        }
        "block_by_number" => {
            let mut result: Value = json!("Initial Error");
            if let Some(raw) = req.params[0].as_str() {
                let bno = match raw.parse::<u64>() {
                    Ok(n) => n,
                    Err(_e) => {
                        result = json!("U64 parse Failed");
                        return Json(RpcResponse {
                            jsonrpc: "2.0".to_string(),
                            success,
                            result,
                            id: req.id,
                        });
                    }
                };

                match node.provider.db().get_block(bno) {
                    Ok(Some(block)) => {
                        result = json!({
                            "block": hex::encode(block.encode_ref()),
                        });
                    }
                    Ok(None) => {
                        result = json!("There is no block you want to find");
                    }
                    Err(_e) => {
                        result = json!("DB Error");
                    }
                }
            }
            Json(RpcResponse {
                jsonrpc: "2.0".to_string(),
                success,
                result,
                id: req.id,
            })
        }
        "peers" => {
            let result: Value = json!("There isn't peer you want to find");

            if let Some(raw) = req.params[0].as_str() {
                dbg!(raw);
                // TODO: Fill below
                // let addr = SocketAddr::from(raw);
                // match node.handle_network(NetworkHandleMessage::PeerConnectionTest { peer: () });
            }

            Json(RpcResponse {
                jsonrpc: "2.0".to_string(),
                success,
                result,
                id: req.id,
            })
        }
        _ => {
            let result = json!("Wrong Requirement");
            Json(RpcResponse {
                jsonrpc: "2.0".to_string(),
                success,
                result,
                id: req.id,
            })
        }
    }
}
