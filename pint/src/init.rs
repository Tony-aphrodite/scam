use node::Node;
use primitives::transaction::SignedTransaction;
use provider::DatabaseTrait;
use tracing::error;
use transaction_pool::identifier::TransactionOrigin;

// Addr: 28dcb1338b900419cd613a8fb273ae36e7ec2b1d, Seed: pint
// Addr: 0534501c34f5a0f3fa43dc5d78e619be7edfa21a, Seed: chain
// Addr: 08041f667c366ee714d6cbefe2a8477ad7488f10, Seed: apple
// Addr: b2aaaf07a29937c3b833dca1c9659d98a9569070, Seed: banana
pub fn init_txs<DB: DatabaseTrait>(node: &Node<DB>) {
    // Test code! Initial transactions
    // From: pint, To: apple, Fee: 10, Value: 1000, Nonce: 0
    let tx = "0000000000000000000000000000000008041f667c366ee714d6cbefe2a8477ad7488f100000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000003e8e124cac1252a8595c4da5e4d810d231a68571e8b590da337c17a67980e9452ef4e4dbd0a4b7312bd778b5a28dde2e73d152c07a56c5cb246d84f2d6f6d5631aa00";
    let data = hex::decode(tx).unwrap();
    let (signed, _) = SignedTransaction::decode(&data).unwrap();

    if let Err(_e) = node.pool.add_transaction(
        TransactionOrigin::External,
        signed.into_recovered().unwrap(),
    ) {
        error!("Tx1 add failed");
    }
    // From: pint, To: banana, Fee: 10, Value: 1000, Nonce: 1
    let tx = "00000000000000000000000000000001b2aaaf07a29937c3b833dca1c9659d98a95690700000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000003e8c1f3d993c37465ba08cf75eecddb01b214f84d77be915543c47374ae22d4cc6b78354616140743272fd536194a866ad0bd3c6d2d3f4531ee52d3c6bad99b5d1a01";
    let data = hex::decode(tx).unwrap();
    let (signed, _) = SignedTransaction::decode(&data).unwrap();

    if let Err(_e) = node.pool.add_transaction(
        TransactionOrigin::External,
        signed.into_recovered().unwrap(),
    ) {
        error!("Tx2 add failed");
    }
    // From: chain, To: banana, Fee: 5, Value: 1000, Nonce: 0
    let tx = "00000000000000000000000000000000b2aaaf07a29937c3b833dca1c9659d98a95690700000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000003e806cc9be9a58dbba4fa5459512c6d5c3d100bbfcb71cfffb669037243babb0c8678077cba676a8eb659f35a148b551dfadaef085cccbac97729c5a743cab9eec901";
    let data = hex::decode(tx).unwrap();
    let (signed, _) = SignedTransaction::decode(&data).unwrap();

    if let Err(_e) = node.pool.add_transaction(
        TransactionOrigin::External,
        signed.clone().into_recovered().unwrap(),
    ) {
        error!("Tx3 add failed");
    }

    node.pool.print_pool();
    node.handle_network(primitives::handle::NetworkHandleMessage::NewTransaction(
        signed,
    ));
}
