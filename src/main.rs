use ethers::core::rand::thread_rng;
use ethers::prelude::*;
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::{fs, time::Duration};
use tokio::time::sleep;

#[derive(Serialize, Deserialize, Clone)]
struct KeyPair {
    private_key: String,
    address: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // take the first argument as the number of txs per second
    let args: Vec<String> = std::env::args().collect();
    // Parameters
    let txs_per_second: u64 = args
        .get(1)
        .expect("Please provide the number of txs per second")
        .parse::<u64>()
        .expect("Please provide a valid number of txs per second");
    //based on the TPS and benchmark duration, we can calculate the total number of txs
    let benchmark_duration: u64 = args
        .get(2)
        .expect("Please provide the benchmark duration in seconds")
        .parse::<u64>()
        .expect("Please provide a valid benchmark duration in seconds");
    let total_txs: u64 = txs_per_second * benchmark_duration;
    let sleep_duration = Duration::from_micros(1_000_000 / txs_per_second);
    let sender_filename = "sender_keypair.json";
    let recipient_filename = "recipient_keypair.json";
    let recipient_key_pair: KeyPair;

    // Check if recipient_keypair.json already exists, if not generate and save
    if !std::path::Path::new(recipient_filename).exists() {
        // Generate a random key-pair
        let wallet = LocalWallet::new(&mut thread_rng());
        recipient_key_pair = KeyPair {
            private_key: wallet.signer().as_nonzero_scalar().to_string(),
            address: hex::encode(wallet.address().as_bytes()),
        };

        let json = serde_json::to_string_pretty(&recipient_key_pair.clone())?;
        fs::write(recipient_filename, json)?;
        println!("Key-pair saved to {}", recipient_filename);
    } else {
        // Read the key-pair from the file
        let data = fs::read_to_string(recipient_filename)?;
        recipient_key_pair = serde_json::from_str(&data)?;
    }

    // Check if sender_keypair.json already exists, if not exit
    if !std::path::Path::new(sender_filename).exists() {
        println!("Please create a sender_keypair.json file");
        std::process::exit(1);
    }

    // Read the key-pair from the file
    let data = fs::read_to_string(sender_filename)?;
    let sender_key_pair: KeyPair = serde_json::from_str(&data)?;


    // Connect to the network (localhost in this example)
    let provider = Provider::<Http>::try_from("https://rpc.dev.hydrasquare-holding.com")?;

    // Load wallet
    // Replace with your private key
    // let private_key_bytes =
    //     hex::decode("62cbb1e7f78278e34d533e8a76e7fed24f694342ca188a1fb8943d24d6b25d2b").unwrap();
    let private_key_bytes = hex::decode(sender_key_pair.private_key).unwrap();
    let wallet = Wallet::from_bytes(&private_key_bytes)
        .expect("Failed to create wallet from private key bytes")
        .with_chain_id(42 as u64);
    let nonce = provider
        .get_transaction_count(wallet.address(), None)
        .await
        .unwrap();
    let provider_w_signer = provider.with_signer(wallet);

    let chain_id: U64 = match provider_w_signer.get_chainid().await {
        Ok(id) => U64::from(id.as_u64()),
        Err(_) => U64::from(1),
    };
    let start_block = provider_w_signer.get_block_number().await?;
    let start_block_timestamp = provider_w_signer
        .get_block(start_block)
        .await?
        .unwrap()
        .timestamp;
    // now timestamp
    let start_time = chrono::Utc::now();
    println!("Connected to chain {}", chain_id);
    println!("Nonce: {}", nonce);
    println!("Address: {}", recipient_key_pair.address);
    println!("Sending {} transactions per second", txs_per_second);
    println!("Benchmark duration: {} seconds", benchmark_duration);
    println!("Total txs: {}", total_txs);
    println!("Starting in 3 seconds...");
    // wait a bit
    sleep(Duration::from_secs(3)).await;

    let atomic_nonce = Arc::new(AtomicU64::new(nonce.as_u64()));
    let processed_txs = Arc::new(AtomicU64::new(0));

    while processed_txs.load(Ordering::SeqCst) < total_txs {
        let mut handles = Vec::with_capacity(txs_per_second as usize);

        for _ in 0..txs_per_second {
            let current_nonce = atomic_nonce.fetch_add(1, Ordering::SeqCst);

            let provider_clone = provider_w_signer.clone();
            let recipient_key_pair_clone = recipient_key_pair.clone();
            let processed_txs_clone = processed_txs.clone();

            let handle = tokio::spawn(async move {
                let recipient = Address::from_str(&recipient_key_pair_clone.address).unwrap();

                let tx = Eip1559TransactionRequest::new()
                    .nonce(current_nonce)
                    .to(recipient)
                    .max_priority_fee_per_gas(0u64) // Adjust as needed
                    .max_fee_per_gas(0u64) // Adjust as needed
                    .gas(21000u64)
                    .value(U256::exp10(10));

                let pending_tx = provider_clone.send_transaction(tx, None).await;
                match pending_tx {
                    Ok(tx) => {
                        println!("Tx sent: 0x{:x}", tx.tx_hash());
                        processed_txs_clone.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(e) => println!("Error sending tx: {}", e),
                }
            });

            handles.push(handle);
        }

        // Awaiting all the spawned tasks
        for handle in handles {
            handle.await?;
        }

        sleep(sleep_duration).await;
    }

    let end_block = provider_w_signer.get_block_number().await?;
    let end_block_timestamp = provider_w_signer
        .get_block(end_block)
        .await?
        .unwrap()
        .timestamp;
    let end_time = chrono::Utc::now();
    println!("");
    println!("");
    println!("Benchmark finished");
    println!("Start block: {}", start_block);
    println!("End block: {}", end_block);
    println!("Total txs: {}", total_txs);
    println!("Expected TPS: {}", txs_per_second);
    println!("Expected duration: {} seconds", benchmark_duration);
    println!(
        "Actual TPS: {:.2}",
        (total_txs as f64) / (end_block_timestamp.as_u64() - start_block_timestamp.as_u64()) as f64
    );
    println!(
        "Actual duration: {:.2} seconds",
        (end_time - start_time).num_seconds()
    );
    Ok(())
}
