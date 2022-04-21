This is a tool to move the data from S3 to SingleStore in a structured way.

## Migration

```bash
# Add the new migration
sqlx migrate add migration_name

# Apply migrations
sqlx migrate run
```


## Linux

```bash
sudo apt install git build-essential pkg-config libssl-dev tmux
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
ulimit -n 30000
cargo build --release
cargo run --release -- --s3-bucket-name near-lake-data-mainnet --s3-region-name eu-central-1 --start-block-height 9820210
```

## How to write `SELECT` in a more efficient way?

#### Try not to use `SELECT *`. 

Column databases store columns separately, it's hard to collect the values from all the columns. 

#### If you need to join the tables, and you know that the needed values are stored in the same block, add this condition to the join clause.

This is what you want to query:
```sql
SELECT action_receipts.originated_from_transaction_hash, action_receipts__outputs.output_data_id
FROM action_receipts JOIN action_receipts__outputs ON
    action_receipts.receipt_id = action_receipts__outputs.receipt_id;
```
That's how to speed it up:
```sql
SELECT action_receipts.originated_from_transaction_hash, action_receipts__outputs.output_data_id
FROM action_receipts JOIN action_receipts__outputs ON 
    action_receipts.block_hash = action_receipts__outputs.block_hash AND
    action_receipts.receipt_id = action_receipts__outputs.receipt_id;
```
The reason is the sharding mechanism.
Second query knows that there's no need in searching for the data in the other shards.

#### Straight ordering will always work faster than the opposite.

Unfortunately, this is the limitation from SingleStore, they can't use the sort index in the opposite direction, and they do not allow to have more than one sorting index per table.
I'm considering the idea of storing everything in the opposite direction.
I have a guess that the query "give me top N of the latest rows" is the most frequent, and it's better to optimize for it.

### Processed blocks

We've collected all the blocks till January 19, 2022. Next blocks will come soon.

Processed:
- _genesis - 57559700 (8)
- 58000000 - 61560800 (5)
- 62000000 - 63480200 (7)
