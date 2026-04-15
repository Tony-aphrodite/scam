# Pint-chain
## Project Overview
 **pint-chain** is a Rust-based experimental **Layer 1 blockchain node project** designed for learning purposes.  
The primary goal of this project is to understand how real-world blockchain clients operate internally by implementing a simplified Layer 1 blockchain from scratch.

 This project is heavily inspired by the architecture of the Ethereum Execution Client **reth**, and focuses on breaking down and re-implementing its core ideas in a more compact form.  
 
 pint-chain is an **account-based blockchain**, and the node is designed around a set of clearly separated core components, each with a well-defined responsibility:
- `Network`  
  Handles TCP-based P2P communication, peer management, and block/transaction propagation.
- `Consensus (BlockImporter + Miner)`  
  Responsible for chain selection, block validation, and Proof-of-Work–based mining.
- `PayloadBuilder`  
  Packages validated transactions into new block payloads.
- `Pool (TxPool + Validator)`  
  Manages transaction collection, validation, and state-based filtering.
- `Provider (StateProvider + DB)`  
  Provides an abstraction layer over block data and account state stored in the database.
- `RPC Server`  
  Exposes an HTTP-based interface for external clients to interact with the node.

The consensus algorithm is **Proof of Work (PoW)**, with a simplified difficulty adjustment rule where the difficulty is doubled or halved on every block. 

Chain selection follows the **Longest Chain Rule**, and peer discovery starts from a boot node, followed by random peer selection.

Transaction signing and verification are implemented using **ECDSA with signer recovery**, following the same model used by Ethereum.

Beyond basic functionality, this project is Validating node correctness and stability through **unit tests and end-to-end (E2E) tests**

Overall, pint-chain serves as a compact but realistic playground for experimenting with blockchain node architecture, consensus mechanics, networking, and system-level design trade-offs.

## How to use
### Run a node
```bash
cargo run -- --boot-node
cargo run -- --name A --port 30304 --rpc-port 8546 --miner-address 0534501c34f5a0f3fa43dc5d78e619be7edfa21a
```
### E2E-test
```bash
cd e2e-test
RUST_LOG=DEBUG cargo test multi -- --no-capture
```

### Rpc connectioin
Use [clients/rpc](clients/rpc) crate. 
Currently only responds with binary data (example included).

## Execution basic examples
This example uses pint-utils crates that sends test transactions.

### Boot Node
1. Node start
<img width="1918" height="268" alt="2026-01-12 232556" src="https://github.com/user-attachments/assets/77d33e0a-db56-4fc6-a631-8dcf2dcbd088" />  <br>

2. New transaction + payload building
<img width="1918" height="360" alt="2026-01-12 232646" src="https://github.com/user-attachments/assets/fdbd47e3-3f4e-43b1-8b89-e6898a496c12" />  <br>

3. Mining + block import + broadcasting block + pool reorg + db import
<img width="1919" height="451" alt="2026-01-12 232728" src="https://github.com/user-attachments/assets/ded39ac6-b140-4850-92f8-41a58812f33e" />  <br>

4. New peer connection (with Node A) + send block sync data
<img width="1915" height="202" alt="2026-01-12 232809" src="https://github.com/user-attachments/assets/456a8b66-c6c7-43e6-be53-136bf7a44415" />  <br>

5. Peer connection test: Ping (Boot Node: Ping -> Node A Pong)
<img width="1917" height="94" alt="2026-01-12 232908" src="https://github.com/user-attachments/assets/b3c9243b-a77b-4c93-b110-0d8d12f5fc11" />  <br>

6. Ctrl+C: Gracefully shutdown (Node A: peer disconnection)
<img width="1919" height="25" alt="2026-01-12 232933" src="https://github.com/user-attachments/assets/903f0cce-559e-4504-85ba-2d97be138d85" />  <br>

### Node A
1. Node start
<img width="1919" height="181" alt="2026-01-12 233438" src="https://github.com/user-attachments/assets/45ee399b-8835-49c2-929d-da2d54588213" />  <br>

2. Connect with boot node + Block Sync (Boot Node: New peer connection + send block sync data)
<img width="1918" height="596" alt="2026-01-12 233528" src="https://github.com/user-attachments/assets/061568b2-1b1a-4bc7-a87c-7c3695135626" />  <br>

3. Receive block from boot node + halt mining + block import + pool reorg + db import
<img width="1917" height="437" alt="2026-01-12 233746" src="https://github.com/user-attachments/assets/988ecb58-2314-453c-8f8f-946e0b8fe2ee" />  <br>

4. Peer connection test: Pong (Boot Node: Ping -> Node A Pong)
<img width="1917" height="41" alt="2026-01-12 233829" src="https://github.com/user-attachments/assets/b5809cbb-37f6-4a94-9db8-7284ee528722" />  <br>

5. Remove Peer (Ctrl+C from Boot Node)
<img width="1919" height="57" alt="2026-01-12 233914" src="https://github.com/user-attachments/assets/f35658b4-a6b5-4717-9837-e03424dacca2" />  <br>

