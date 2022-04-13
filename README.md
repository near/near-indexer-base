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
cargo build
cargo run
```


### Processed blocks

(even a little more)
- _genesis - 15395100
- 20000000 - 23003400
- 30000000 - 32568600
- 40000000 - 41340900
- 50000000 - 50495600
- 60000000 - 60005500 (stopped)
