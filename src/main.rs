use sha2::{Digest, Sha256};
use serde::{Serialize, Deserialize};
use tokio::sync::mpsc;
use std::{sync::Arc};
use tokio::sync::Mutex;


#[derive(Serialize, Deserialize, Debug, Clone)]
struct Block {
    index: u64,
    timestamp: u128,
    data: String,
    previous_hash: String,
    hash: String,
    nonce: u64,
}

impl Block {

    fn compute_hash(index: u64, timestamp: u128, data: &str, previous_hash: &str, nonce: u64) -> String {
        let mut hasher = Sha256::new();
        hasher.update(index.to_string());
        hasher.update(timestamp.to_string());
        hasher.update(data);
        hasher.update(previous_hash);
        hasher.update(nonce.to_string());

        let result = hasher.finalize();
        hex::encode(result)
}

    fn new_block(index: u64, timestamp: u128, data: String, previous_hash: String, nonce: u64) -> Self {
        let hash = Block::compute_hash(index, timestamp, &data, &previous_hash, nonce);

        Block {
            index,
            timestamp,
            data,
            previous_hash,
            nonce,
            hash,
        }
    }

    fn genesis_block() -> Self {
        let index = 0;
        let timestamp = 0;
        let data = String::from("Hi There");
        let previous_hash = String::from("0");
        let nonce = 0;

        Block::new_block(index, timestamp, data, previous_hash, nonce)
    }


    fn mine_block(index: u64, timestamp: u128, data: String, previous_hash: String, difficulty: usize) -> Self {

        let prefix_target = "0".repeat(difficulty);
        let mut nonce = 0;

        loop {
            let hash = Block::compute_hash(index, timestamp, &data, &previous_hash, nonce);

            if hash.starts_with(&prefix_target) {
                println!("Block minted with nonce: {} -> hash:  {}", nonce, hash);

                return Block { index, timestamp, data, previous_hash, hash, nonce };
            }
            nonce = nonce + 1;
        }
    }

}

struct Blockchain {
    chain: Vec<Block>,
    difficulty: usize
}

impl Blockchain {
    
    fn new(difficulty: usize) -> Self {
        Blockchain {
            chain: vec![Block::genesis_block()],
            difficulty,
        }
    }

    fn last_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    fn add_block(&mut self, block: Block) -> bool {
        let last = self.last_block();

        if block.index != last.index + 1 {
            println!("Index mismatch");
            return false;
        }

        if block.previous_hash != last.hash {
            println!("Previous hash mismatch");
            return false;
        }

        let hash_expected = Block::compute_hash(
            block.index,
            block.timestamp,
            &block.data,
            &block.previous_hash,
            block.nonce,
        );

        if hash_expected != block.hash {
            println!("Hash mismatch");
            return false;
        }

        self.chain.push(block);
        println!("Block added successfully");
        true
    }

    fn is_valid_chain(chain: &[Block], difficulty: usize) -> bool {
        if chain.is_empty() { return false; }
        for i in 1..chain.len() {
            let prev = &chain[i - 1];
            let cur = &chain[i];
            if cur.previous_hash != prev.hash { return false; }
            let recomputed = Block::compute_hash(cur.index, cur.timestamp, &cur.previous_hash, &cur.data, cur.nonce);
            if recomputed != cur.hash { return false; }
            if !cur.hash.starts_with(&"0".repeat(difficulty)) { return false; }
        }
        true
    }

    fn replace_chain(&mut self, new_chain: Vec<Block>) -> bool {
        if new_chain.len() > self.chain.len() && Blockchain::is_valid_chain(&new_chain, self.difficulty) {
            self.chain = new_chain;
            true
        } else {
            false
        }
    }
}

struct Node {
    id: usize,
    blockchain: Arc<Mutex<crate::Blockchain>>,
    senders: Vec<mpsc::Sender<Message>>,
    receiver: mpsc::Receiver<Message>,
}

impl Node {
    fn new(id: usize, difficulty: usize) -> (Self, mpsc::Sender<Message>) {

        let (tx, rx) = mpsc::channel(100);
        let blockchain = crate::Blockchain::new(difficulty);

        (
            Node {
                id,
                blockchain: Arc::new(Mutex::new(blockchain)),
                senders: Vec::new(),
                receiver: rx,
            },
            tx,
        )
    }

    fn connect (&mut self, sender: mpsc::Sender<Message>) {
        self.senders.push(sender);
    }
    
    async fn broadcast (&self, msg: Message) {
        for p in &self.senders {
            let _ = p.send(msg.clone()).await;

        }
    }

    async fn run (mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                Message::Mine(data) => {
                    let blockchain_clone = self.blockchain.clone();
                    let senders_clone = self.senders.clone();
                    let my_id = self.id;

                    //mining
                    tokio::spawn(async move {

                        let (index, previous_hash, difficulty) = {
                            let bchain = blockchain_clone.lock().await;

                            (bchain.last_block().index + 1, bchain.last_block().hash.clone(), bchain.difficulty)
                        };

                        let timestamp = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis();
                    
                        let block_mined = tokio::task::spawn_blocking(move || {
                            crate::Block::mine_block(index, timestamp, data, previous_hash, difficulty)
                        }).await.expect("mining task panicked");

                        let mut bchain = blockchain_clone.lock().await;

                        if bchain.add_block(block_mined.clone()) {
                            println!("node {} mined block {}", my_id, block_mined.index);

                            drop(bchain);

                            for p in senders_clone {
                                let _ = p.send(Message::NewBlock(block_mined.clone())).await;
                            }
                        } else {
                            println!("node is {}. but couldn't add it locally", my_id)
                        }

                    });

                }
                
                Message::NewBlock(block) => {

                    let mut bchain = self.blockchain.lock().await;

                    if bchain.add_block(block.clone()) {
                        println!("node {}, {} block is accepted and broadcasting", self.id, block.index);
                        drop(bchain);
                        self.broadcast(Message::NewBlock(block)).await;
                        
                    } else {
                        println!("node is {}, block is rejected {} -- requesting chain", self.id, block.index);
                        drop(bchain);

                        //requesting the chain with our id
                        self.broadcast(Message::RequestChain(self.id)).await;
                    }
                }

                Message::RequestChain(from_id) => {
                    let bchain = self.blockchain.lock().await;
                    let blockchain_copy = bchain.chain.clone();

                    drop(bchain);

                    println!("node {}, chain is requesting from id: {}", self.id, from_id);
                    self.broadcast(Message::Chain(blockchain_copy)).await;

                }

                Message::Chain(in_chain) => {
                    let mut bchain = self.blockchain.lock().await;

                    if bchain.replace_chain(in_chain.clone()) {
                        println!("node: {}, new chain replaced the old chain (len {})", self.id, in_chain.len());
                    }
                }
            }
        }
    }
}


#[tokio::main]
async fn main() {

    let node_total = 4usize;
    let difficulty = 3usize;
    let run_time = 10u64;

    //nodes and senders creations

    let mut nodes = Vec::new();
    let mut transactions = Vec::new();

    for i in 0..node_total {
        let (node, tx) = Node::new(i, difficulty);

        nodes.push(node);
        transactions.push(tx);
    }

    //connect receivers
    for i in 0..node_total {
        for j in 0..node_total {
            if i == j {
                continue;
            }
            nodes[i].connect(transactions[j].clone()    );
        }
    }

    for node in nodes {
        tokio::spawn(node.run());
    }

    use rand::Rng;

    let mut rng = rand::thread_rng();
    let start = std::time::SystemTime::now();


    //mine random node
    while std::time::SystemTime::now()
        .duration_since(start).unwrap().as_secs() < run_time {
            let somene = rng.gen_range(0..node_total);
            let data = format!("transaction: {}", rng.gen_range(0u64..u64::MAX));

            let _ = transactions[somene].send(Message::Mine(data)).await;

            tokio::time::sleep(std::time::Duration::from_millis(800)).await;

        }

        //broadcast node
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        for tx in &transactions {
            let _ = tx.send(Message::RequestChain(999)).await;
        }

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        println!("Finished the simulation")

    
}

#[derive(Clone)]
enum Message {
    Mine(String),
    NewBlock(crate::Block),
    RequestChain(usize),
    Chain(Vec<crate::Block>),
}