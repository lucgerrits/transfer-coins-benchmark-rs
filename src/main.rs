use ethers::core::rand::thread_rng;
use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::ops::Add;
use std::str::FromStr;
use std::{fs, time::Duration};
use tokio::time::sleep;
use eyre::Result;

#[derive(Serialize, Deserialize, Clone)]
struct KeyPair {
    private_key: String,
    address: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parameters
    let txs_per_second = 10;
    let sleep_duration = Duration::from_micros(1_000_000 / txs_per_second);
    let filename = "keypair.json";
    let key_pair: KeyPair;

    // Check if keypair.json already exists, if not generate and save
    if !std::path::Path::new(filename).exists() {
        // Generate a random key-pair
        let wallet = LocalWallet::new(&mut thread_rng());
        key_pair = KeyPair {
            private_key: wallet.signer().as_nonzero_scalar().to_string(),
            address: hex::encode(wallet.address().as_bytes()),
        };

        let json = serde_json::to_string_pretty(&key_pair.clone())?;
        fs::write(filename, json)?;
        println!("Key-pair saved to {}", filename);
    } else {
        // Read the key-pair from the file
        let data = fs::read_to_string(filename)?;
        key_pair = serde_json::from_str(&data)?;
    }

    // Connect to the network (localhost in this example)
    let provider = Provider::<Http>::try_from("https://rpc.dev.hydrasquare-holding.com")?;

    // Load wallet
    // Replace with your private key
    let private_key_bytes =
        hex::decode("62cbb1e7f78278e34d533e8a76e7fed24f694342ca188a1fb8943d24d6b25d2b").unwrap();
    let wallet = Wallet::from_bytes(&private_key_bytes)
        .expect("Failed to create wallet from private key bytes")
        .with_chain_id(42 as u64);
    let mut nonce = provider
        .get_transaction_count(wallet.address(), None)
        .await
        .unwrap();
    let provider_w_signer = provider.with_signer(wallet);

    let chain_id: U64 = match provider_w_signer.get_chainid().await {
        Ok(id) => U64::from(id.as_u64()),
        Err(_) => U64::from(1),
    };
    println!("Connected to chain {}", chain_id);
    println!("Nonce: {}", nonce);
    println!("Address: {}", key_pair.address);
    println!("Sending {} transactions per second", txs_per_second);

    loop {
        // Send a transaction (in this case, a simple ETH transfer)
        let recipient = Address::from_str(&key_pair.address)?;
        let tx = Eip1559TransactionRequest::new()
            .nonce(nonce)
            .to(recipient)
            .max_priority_fee_per_gas(0u64)
            .max_fee_per_gas(0u64)
            .gas(21000u64)
            .value(U256::exp10(18)); // Sending 100 wei
        let pending_tx = provider_w_signer.send_transaction(tx, None).await?;
        println!("Tx sent: 0x{:x}", pending_tx.tx_hash());

        // get the mined tx
        let _receipt = pending_tx
            .await?
            .ok_or_else(|| eyre::format_err!("tx dropped from mempool"))?;

        // let tx = provider_w_signer.get_transaction(_receipt.transaction_hash).await?;
        // println!("Sent tx: {}\n", serde_json::to_string(&tx)?);
        // println!("Tx receipt: {}", serde_json::to_string(&_receipt)?);

        nonce = nonce.add(1);
        sleep(sleep_duration).await;
        // break;
    }
    #[allow(unreachable_code)]
    Ok(())
}
