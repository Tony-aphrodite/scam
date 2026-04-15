use primitives::{
    transaction::{SignedTransaction, Transaction},
    types::{Address, U256},
};
use serde_json::json;

pub async fn send_tx_to_rpc(signed: SignedTransaction, url: &str) -> anyhow::Result<String> {
    // Test code! Initial transactions
    // From: pint, To: apple, Fee: 10, Value: 1000, Nonce: 0

    let encoded = hex::encode(signed.encode());

    let payload = json!({
        "jsonrpc": "2.0",
        "method": "local_transaction",
        "params": [encoded],
        "id": 0
    });

    let _res = reqwest::Client::new().post(url).json(&payload).send().await;

    Ok(encoded)
}

pub async fn get_tx_from_rpc(tx_hash: String, url: &str) -> anyhow::Result<Transaction> {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "transaction",
        "params": [tx_hash],
        "id": 0
    });

    let res = reqwest::Client::new()
        .post(url)
        .json(&payload)
        .send()
        .await?;

    let body = res.text().await?;
    let resp: serde_json::Value = serde_json::from_str(&body)?;

    let tx_hex = resp["result"]["tx"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("missing result.tx"))?;
    let bytes = hex::decode(tx_hex)?;
    let (signed, _size) = SignedTransaction::decode(&bytes).unwrap();
    Ok(signed.tx)
}

pub async fn get_account_from_rpc(address: Address, url: &str) -> anyhow::Result<(u64, U256)> {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "account",
        "params": [address.get_addr_hex()],
        "id": 0
    });

    let res = reqwest::Client::new()
        .post(url)
        .json(&payload)
        .send()
        .await?;

    let body = res.text().await?;
    let resp: serde_json::Value = serde_json::from_str(&body)?;

    let nonce = resp["result"]["nonce"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Can't find account info!"))?
        .parse()?;

    let balance = resp["result"]["balance"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Can't find account info!"))?
        .parse()?;

    Ok((nonce, balance))
}

pub async fn get_chain_height_from_rpc(url: &str) -> anyhow::Result<u64> {
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "blockchain_height",
        "params": [],
        "id": 0
    });

    let res = reqwest::Client::new()
        .post(url)
        .json(&payload)
        .send()
        .await?;

    let body = res.text().await?;
    let resp: serde_json::Value = serde_json::from_str(&body)?;

    let block_height = resp["result"]
        .as_u64()
        .ok_or(anyhow::anyhow!("result is not u64"))?;

    Ok(block_height)
}
