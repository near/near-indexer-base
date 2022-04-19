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


### Processed blocks

We've collected all the blocks till January 17, 2022. Next blocks will come soon.

Processed:
- _genesis - 57559700 (8)
- 58000000 - 61560800 (5)
- 62000000 - 63480200 (7)
