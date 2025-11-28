use sha2::{Digest, Sha256};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
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
}

impl Blockchain {
    
    fn new() -> Self {
        Blockchain {
            chain: vec![Block::genesis_block()],
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
}

fn main() {
    
    let mut chain = Blockchain::new();

    let difficulty = 3;

    let b1 = Block::mine_block(
        1, 
        1, 
        String::from("I mined a block"), 
        chain.last_block().hash.clone(), 
        difficulty);

    chain.add_block(b1);

    let b2 = Block::mine_block(2, 
        2,
        String::from("second block mined"), 
        chain.last_block().hash.clone(),
         difficulty);
    
    chain.add_block(b2);

    println!("Blockchain: {:#?}", chain.chain);

}