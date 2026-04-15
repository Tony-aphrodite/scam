use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

use e2e_test::{
    common::{create_key_pairs, create_signed},
    process::{NodeConfig, launch_test_node},
    rpc_client::{
        get_account_from_rpc, get_chain_height_from_rpc, get_tx_from_rpc, send_tx_to_rpc,
    },
};
use primitives::{
    transaction::Transaction,
    types::{Address, U256},
};
use tracing_subscriber::EnvFilter;

#[tokio::test]
async fn e2e_single_node_basic() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .with_level(true)
        .init();

    let boot_node_url = "http://127.0.0.1:8888";
    let miner_address = String::from("28dcb1338b900419cd613a8fb273ae36e7ec2b1c");
    let boot_node = NodeConfig {
        name: String::from("Boot_node"),
        address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        port: 33333,
        rpc_port: 8888,
        miner_address: miner_address.clone(),
        boot_node: true,
    };
    let _ = launch_test_node(boot_node).await;

    let (key_pint, addr_pint) = create_key_pairs("pint".as_bytes());
    let (_key_apple, addr_apple) = create_key_pairs("banana".as_bytes());

    tokio::time::sleep(Duration::from_secs(3)).await;

    let (nonce_before, balance_before) = get_account_from_rpc(addr_pint, boot_node_url)
        .await
        .expect("Account must exists");

    let tx = Transaction {
        chain_id: 0,
        nonce: 0,
        to: addr_apple,
        fee: 5,
        value: U256::from(1000),
    };

    let signed = create_signed(&key_pint, tx);

    tokio::time::sleep(Duration::from_secs(2)).await;
    let _ = send_tx_to_rpc(signed.clone(), boot_node_url).await.unwrap();

    // Wait enough time for mining block..
    tokio::time::sleep(Duration::from_secs(15)).await;

    // Transaction Request
    let encoded_hash = hex::encode(signed.hash.0.as_slice());
    let tx = get_tx_from_rpc(encoded_hash, boot_node_url)
        .await
        .expect("tx must exist");
    let tx_hash = tx.encode_for_signing();
    assert_eq!(tx_hash, signed.hash);
    // Account Request
    let (nonce, balance) = get_account_from_rpc(addr_pint, boot_node_url)
        .await
        .expect("Account must exists");
    assert_eq!(nonce - nonce_before, 1);
    assert_eq!(balance_before - balance, 1005);
    // Blcok Height Request
    let block_height = get_chain_height_from_rpc(boot_node_url)
        .await
        .expect("Can't get block height from node!");
    assert_eq!(block_height, 1);
    // Miner account
    let (_nonce, balance) =
        get_account_from_rpc(Address::from_hex(miner_address).unwrap(), boot_node_url)
            .await
            .expect("Account must exists");
    assert_eq!(balance, 5);

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
    }
    // Wait enough time for mining block..
    tokio::time::sleep(Duration::from_secs(15)).await;
}

#[tokio::test]
async fn e2e_multi_node_basic() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(true)
        .with_level(true)
        .init();
    let boot_node_url = "http://127.0.0.1:8888";
    let boot_node = NodeConfig {
        name: String::from("Boot_node"),
        address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        port: 33333,
        rpc_port: 8888,
        miner_address: String::from("28dcb1338b900419cd613a8fb273ae36e7ec2b1c"),
        boot_node: true,
    };
    let _ = launch_test_node(boot_node).await;
    tokio::time::sleep(Duration::from_secs(3)).await;

    let node_a = NodeConfig {
        name: String::from("Node_A"),
        address: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        port: 33334,
        rpc_port: 8889,
        miner_address: String::from("28dcb1338b900419cd613a8fb273ae36e7ec2b20"),
        boot_node: false,
    };
    let _ = launch_test_node(node_a).await;
    tokio::time::sleep(Duration::from_secs(3)).await;

    let (key_pint, addr_pint) = create_key_pairs("pint".as_bytes());
    let (_key_apple, addr_apple) = create_key_pairs("apple".as_bytes());
    let (_key_banana, addr_banana) = create_key_pairs("banana".as_bytes());

    tokio::time::sleep(Duration::from_secs(3)).await;

    let (nonce_before, balance_before) = get_account_from_rpc(addr_pint, boot_node_url)
        .await
        .expect("Account must exists");

    let tx = Transaction {
        chain_id: 0,
        nonce: 0,
        to: addr_apple,
        fee: 5,
        value: U256::from(1000),
    };

    let signed = create_signed(&key_pint, tx);
    let _ = send_tx_to_rpc(signed.clone(), boot_node_url).await.unwrap();

    // Wait enough time for mining block..
    tokio::time::sleep(Duration::from_secs(15)).await;

    // Boot node!
    // Transaction Request
    let encoded_hash = hex::encode(signed.hash.0.as_slice());
    let tx = get_tx_from_rpc(encoded_hash, boot_node_url)
        .await
        .expect("tx must exist");
    let tx_hash = tx.encode_for_signing();
    assert_eq!(tx_hash, signed.hash);
    // Account Request
    let (nonce, balance) = get_account_from_rpc(addr_pint, boot_node_url)
        .await
        .expect("Account must exists");

    dbg!(balance, balance_before, nonce_before, nonce);
    assert_eq!(nonce - nonce_before, 1);
    assert_eq!(balance_before - balance, 1005);
    // Block Height Request
    let block_height = get_chain_height_from_rpc(boot_node_url)
        .await
        .expect("Can't get block height from node!");
    //
    assert_eq!(block_height, 1);

    // Node A
    // Transaction Request
    let node_a_url = "http://127.0.0.1:8889";
    let encoded_hash = hex::encode(signed.hash.0.as_slice());
    let tx = get_tx_from_rpc(encoded_hash, node_a_url)
        .await
        .expect("tx must exist");
    let tx_hash = tx.encode_for_signing();
    assert_eq!(tx_hash, signed.hash);
    // Account Request
    let (nonce, balance) = get_account_from_rpc(addr_pint, node_a_url)
        .await
        .expect("Account must exists");

    dbg!(balance, balance_before, nonce_before, nonce);
    assert_eq!(nonce - nonce_before, 1);
    assert_eq!(balance_before - balance, 1005);
    // Block Height Request
    let block_height = get_chain_height_from_rpc(node_a_url)
        .await
        .expect("Can't get block height from node!");
    //
    assert_eq!(block_height, 1);

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

        let signed = create_signed(&_key_apple, tx);
        let _ = send_tx_to_rpc(signed.clone(), boot_node_url).await.unwrap();
    }
    // Wait enough time for mining block..
    tokio::time::sleep(Duration::from_secs(30)).await;
}
