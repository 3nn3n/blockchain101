# blockchain101


# Async P2P Blockchain Simulation in Rust

- A fully asynchronous, multi-node blockchain network implementing Proof-of-Work, peer-to-peer gossip, and longest-chain consensus using Rust and Tokio.

## Overview

This project implements a **miniature blockchain protocol** from first principles, designed to mirror **real-world blockchain engineering patterns** such as:

* Peer-to-peer networking
* Proof-of-Work mining
* Block validation and propagation
* Chain conflict resolution
* Concurrent execution and synchronization

The system simulates a **distributed network of nodes**, each independently mining, validating, broadcasting, and synchronizing blocks without any central coordinator.


## Key Design Goals

* Models real blockchain mechanics, not just data structures
* Mining and networking run asynchronously
* Longest valid chain rule
* All data is revalidated locally
* Easy to reason about and extend


## Architecture

### High-Level Components

| Component      | Description                                 |
| -------------- | ------------------------------------------- |
| **Block**      | Immutable data unit secured with SHA-256    |
| **Blockchain** | Ordered block history with validation rules |
| **Node**       | Independent participant with local state    |
| **Network**    | Fully connected P2P mesh via async channels |
| **Consensus**  | Longest valid chain wins                    |

Each node maintains its **own blockchain** and communicates with peers using **Tokio MPSC channels**, emulating a decentralized gossip network.

## Block Structure

Each block cryptographically commits to:

* Block index
* Timestamp
* Transaction data
* Previous block hash
* Nonce (Proof-of-Work)
* Resulting SHA-256 hash

```text
Block {
  index
  timestamp
  data
  previous_hash
  nonce
  hash = SHA256(index || timestamp || data || previous_hash || nonce)
}
```


## Proof of Work (Mining)

Mining is implemented via a **hash-based difficulty target**:

* A block is valid if its hash starts with `N` leading zeros
* `N` is the network difficulty
* Mining runs on a **dedicated blocking thread** to avoid starving async tasks

```rust
spawn_blocking(mine_block)
```

This closely mirrors real blockchain systems where mining is CPU-bound.

## Network Messaging Protocol

Nodes communicate using a small but expressive protocol:

| Message                 | Purpose                        |
| ----------------------- | ------------------------------ |
| `Mine(data)`            | Trigger block mining           |
| `NewBlock(block)`       | Broadcast newly mined block    |
| `RequestChain(node_id)` | Request full chain on conflict |
| `Chain(Vec<Block>)`     | Respond with full blockchain   |

All messages are **broadcast-based**, enabling decentralized propagation.

## Consensus Mechanism

The system uses a **Longest Valid Chain Rule**:

1. Every received block is independently validated
2. Invalid blocks trigger a full chain request
3. Nodes replace their chain only if:

   * The new chain is longer
   * The entire chain is valid
   * Proof-of-Work is satisfied

This ensures **eventual consistency** across the network.

## Concurrency Model

| Task             | Execution                   |
| ---------------- | --------------------------- |
| Network I/O      | Async Tokio tasks           |
| Mining           | `spawn_blocking` threads    |
| Blockchain State | `Arc<Mutex<>>`              |
| Validation       | Synchronous & deterministic |

This design prevents:

* Async executor starvation
* Race conditions on shared state
* Invalid state propagation

## Simulation Flow

1. Spawn `N` nodes
2. Fully connect all nodes (P2P mesh)
3. Random nodes mine blocks
4. Blocks propagate through the network
5. Forks resolve automatically
6. Final chains converge



