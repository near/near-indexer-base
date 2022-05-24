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
sudo apt install git build-essential pkg-config libssl-dev tmux postgresql-client
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

### How to rewrite solution from MySql to Postgres

```bash
cd src
grep -rli 'sqlx::Pool<sqlx::MySql>' * | xargs -I@ sed -i '' 's/sqlx::Pool<sqlx::MySql>/sqlx::Pool<sqlx::Postgres>/g' @
grep -rli 'sqlx::mysql::MySqlArguments' * | xargs -I@ sed -i '' 's/sqlx::mysql::MySqlArguments/sqlx::postgres::PgArguments/g' @
```

### New data proposals:

- State changes https://github.com/near/nearcore/blob/master/core/primitives/src/views.rs#L1681-L1719
  - access_key changes: we already have them in access_keys, will be covered in the separate table
  - DataUpdate, DataDeletion: have no opinion here
  - ContractCodeUpdate, ContractCodeDeletion: we may want to create a separate table `contracts`
- Blocks https://github.com/near/nearcore/blob/master/core/primitives/src/views.rs#L448-L489
  - gas_price should contain the other data https://github.com/near/near-analytics/issues/19
- Chunks https://github.com/near/nearcore/blob/master/core/primitives/src/views.rs#L719-L743
  - gas_used, gas_limit (?) should contain the other data https://github.com/near/near-analytics/issues/19
- Execution outcomes https://github.com/near/nearcore/blob/master/core/primitives/src/views.rs#L1142-L1162
  - we need to add execution_outcomes__logs table
  - I want to store result https://github.com/near/nearcore/blob/master/core/primitives/src/views.rs#L1032-L1042 
- Transactions https://github.com/near/nearcore/blob/master/core/primitives/src/views.rs#L959-L967
  - do we want to re-think transaction_actions table? We want at least to write a comment why do we ignore this data
- Receipts https://github.com/near/nearcore/blob/master/core/primitives/src/views.rs#L1374-L1389
  - we may want to create 8 separate tables for each ActionView


sort key as 1 column
tx_hash as the shard key

https://stackoverflow.com/questions/70032527/connecting-to-sql-server-with-sqlx
https://github.com/launchbadge/sqlx/issues/1552
(this is the main) https://github.com/launchbadge/sqlx/issues/414
