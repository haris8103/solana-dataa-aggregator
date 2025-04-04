Solana Transaction Processor and PostgreSQL Logger

This project connects to the Solana Devnet, fetches the latest blocks, processes each transaction, and logs the relevant data (transaction hash, sender, receiver, amount, and timestamp) into a PostgreSQL database using r2d2 for connection pooling.
Features:

    Fetches Solana blocks from the Devnet.

    Processes transactions in the block to extract relevant information.

    Logs transaction details into a PostgreSQL database.

    Multi-threading is used to process each transaction concurrently for efficiency.

Requirements:
1. Solana RPC Client:

    This code uses the Solana RPC client to connect to the Solana Devnet. It fetches the latest epoch and then retrieves blocks based on the slot.

2. PostgreSQL Database:

    Ensure you have a PostgreSQL database running locally or update the connection string to connect to a remote database.

    A table called data_aggregator with the following schema is expected:

    CREATE TABLE public.data_aggregator (
        trans_hash TEXT,
        sender TEXT,
        reciever TEXT,
        amount BIGINT,
        time BIGINT
    );

3. Dependencies:

Make sure the following dependencies are included in your Cargo.toml:

[dependencies]
solana-client = "2.2.6"
solana-transaction-status = "2.2.6"
r2d2 = "0.8"
r2d2-postgres = "0.15"
postgres = "0.19.10"
solana-sdk = "1.9.10"
tokio = { version = "1", features = ["full"] }

How to Run
1. Set up PostgreSQL:

Make sure you have PostgreSQL running locally. If not, install PostgreSQL by following the instructions on the official website.

Create a database (mydb) and a table (data_aggregator):

CREATE DATABASE mydb;

Then run this to create the table:

CREATE TABLE public.data_aggregator (
    trans_hash TEXT,
    sender TEXT,
    reciever TEXT,
    amount BIGINT,
    time BIGINT
);

2. Solana RPC:

This code uses the Solana Devnet RPC. If you want to use a different network (Testnet/Mainnet), change the URL in the code accordingly:

let rpc = RpcClient::new("https://api.devnet.solana.com".to_string());

3. Running the Application:

Once the database is set up, and your Solana Devnet client is correctly configured, run the application:

cargo run

4. Application Flow:

    The program creates an RPC client that connects to the Solana Devnet.

    It initializes a PostgreSQL connection pool using r2d2.

    It enters a loop, fetching the current epoch info and fetching the latest block from Solana based on the absolute slot number.

    For each block, it spawns a thread to process each transaction and store the relevant data (transaction hash, sender, receiver, lamports, and time) in the PostgreSQL database.

5. Multi-threading:

    For each transaction, the program spawns a new thread to insert the transaction data into the database concurrently.

    After all threads are completed, the program waits for a 30-second interval before fetching the next block and repeating the process.

Code Explanation

    RPC Client Setup:

let rpc = RpcClient::new("https://api.devnet.solana.com".to_string());

The Solana RPC client connects to the Devnet to retrieve block data.

PostgreSQL Connection Pool:

let mut cfg = Config::new();
cfg.host("localhost")
   .user("postgres")
   .dbname("mydb")
   .password("12345678");
let manager = PostgresConnectionManager::new(cfg, NoTls);
let pool = Pool::new(manager).unwrap();

This part creates a PostgreSQL connection pool using r2d2 and connects to a locally running database (mydb). The password is set to 12345678.

Fetching Epoch Information:

let epoch_info: solana_sdk::epoch_info::EpochInfo = rpc.get_epoch_info().unwrap();

The program fetches the current epoch information from the Solana Devnet, which includes the absolute slot number used to fetch blocks.

Processing Transactions:

for tx in block.transactions.unwrap() {
    let pool = pool.clone();
    let handle = thread::spawn(move || {
        // Process each transaction and insert into PostgreSQL
    });
    handles.push(handle);
}

For each block, the program spawns a new thread to handle the transaction processing. Each transaction is decoded, and its relevant data is extracted and inserted into the database.

Database Insertion:

conn.execute(
    "INSERT INTO public.data_aggregator (trans_hash, sender, reciever, amount, time)
     VALUES ($1, $2, $3, $4, $5)",
    &[&tx_hash, &sender, &receiver, &lamports, &time],
).unwrap();

Each thread inserts transaction data (hash, sender, receiver, amount, and time) into the data_aggregator table in the PostgreSQL database.
