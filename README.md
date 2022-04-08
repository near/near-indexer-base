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
cargo build
cargo run
```