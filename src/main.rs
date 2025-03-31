use postgres::{Config, NoTls};
use solana_client::rpc_client::RpcClient;
use solana_sdk::lamports;
use solana_transaction_status_client_types::UiTransactionEncoding;

use r2d2::Pool;
use r2d2_postgres::PostgresConnectionManager;
use std::thread;

fn main() {
    // Create RPC client
    let rpc = RpcClient::new("https://api.devnet.solana.com".to_string());
    let mut cfg = Config::new();
    // Set up connection pool
    cfg.host("localhost")
        .user("postgres")
        .dbname("mydb")
        .password("12345678");

    let manager = PostgresConnectionManager::new(cfg, NoTls);
    let pool = Pool::new(manager).unwrap();
    loop {
        let epoch_info: solana_sdk::epoch_info::EpochInfo = rpc.get_epoch_info().unwrap();
        // Fetch a block

        let block = rpc
            .get_block_with_config(
                epoch_info.absolute_slot,
                solana_client::rpc_config::RpcBlockConfig {
                    encoding: Some(UiTransactionEncoding::Base64),
                    max_supported_transaction_version: Some(0),
                    ..Default::default()
                },
            )
            .unwrap();

        // Spawn threads for each transaction
        let mut handles = vec![];

        for tx in block.transactions.unwrap() {
            let pool = pool.clone();

            let handle = thread::spawn(move || {
                if let (Some(meta), Some(decoded_tx)) = (tx.meta, tx.transaction.decode()) {
                    let tx_hash = decoded_tx
                        .signatures
                        .get(0)
                        .map(|s| s.to_string())
                        .unwrap_or_default();
                    let sender = decoded_tx
                        .message
                        .static_account_keys()
                        .get(0)
                        .map(|k| k.to_string())
                        .unwrap_or_default();
                    let receiver = decoded_tx
                        .message
                        .static_account_keys()
                        .get(1)
                        .map(|k| k.to_string())
                        .unwrap_or_default();
                    let postbalance = meta.post_balances.get(1).unwrap_or(&0);
                    let prebalance = meta.pre_balances.get(1).unwrap_or(&0);
                    let mut lamports = 0;
                    if postbalance > prebalance {
                        lamports = postbalance - prebalance;
                    } else if prebalance > postbalance {
                        lamports = prebalance - postbalance;
                    }
                    let time: Option<i64> = block.block_time;
                    let lamports = lamports as i64;
                    let mut conn = pool.get().expect("Failed to get DB connection from pool");

                    conn.execute(
                    "INSERT INTO public.data_aggregator (trans_hash, sender, reciever, amount, time)
                     VALUES ($1, $2, $3, $4, $5)",
                    &[&tx_hash, &sender, &receiver, &lamports, &time],
                ).unwrap();
                }
            });

            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
    }
    println!("✅ All transactions inserted.");
}
