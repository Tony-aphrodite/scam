use primitives::{transaction::Transaction, types::U256};
use tokio::time::Duration;


use e2e_test::{
    common::{create_key_pairs, create_signed},
    rpc_client::send_tx_to_rpc,
};


// This is for testing
#[tokio::main]
async fn main() {
    let boot_node_url = "http://127.0.0.1:8888";
    let (key_pint, _addr_pint) = create_key_pairs("pint".as_bytes());
    let (_key_apple, addr_apple) = create_key_pairs("apple".as_bytes());
    let (_key_banana, addr_banana) = create_key_pairs("banana".as_bytes());


    tokio::time::sleep(Duration::from_secs(3)).await;


    let tx = Transaction {
        chain_id: 0,
        nonce: 0,
        to: addr_apple,
        fee: 5,
        value: U256::from(1000),
    };


    let signed = create_signed(&key_pint, tx);
    let _ = send_tx_to_rpc(signed.clone(), boot_node_url).await.unwrap();


    // Wait for the block to be mined...
    tokio::time::sleep(Duration::from_secs(15)).await;


    for i in 1..5 {
        let tx = Transaction {
            chain_id: 0,
            nonce: 5 - i,
            to: addr_apple,
            fee: 5 * i as u128,
            value: U256::from(1000 * i),
        };


        let signed = create_signed(&key_pint, tx);
        let _ = send_tx_to_rpc(signed.clone(), boot_node_url).await.unwrap();


        let tx = Transaction {
            chain_id: 0,
            nonce: 4 - i,
            to: addr_banana,
            fee: 5 * i as u128,
            value: U256::from(1000 * i),
        };


        let signed = create_signed(&_key_banana, tx);
        let _ = send_tx_to_rpc(signed.clone(), boot_node_url).await.unwrap();
    }
}
