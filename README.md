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

- _genesis - 14397200
- 20000000 - 22399200
- 30000000 - 32082800
- 40000000 - 41105800
- 50000000 - 50470300
- 60000000 - 60005500 (stopped)
