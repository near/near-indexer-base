use clap::Parser;

/// NEAR Indexer for Explorer
/// Watches for stream of blocks from the chain
#[derive(Parser, Debug)]
#[clap(
    version,
    author,
    about,
    setting(clap::AppSettings::DisableHelpSubcommand),
    setting(clap::AppSettings::PropagateVersion),
    setting(clap::AppSettings::NextLineHelp)
)]
pub(crate) struct Opts {
    /// Enabled Indexer for Explorer debug level of logs
    #[clap(long)]
    pub debug: bool,
    // todo fix wording
    /// Switches indexer to non-strict mode (skips Receipts without parent Transaction hash, puts such block_height into special table)
    #[clap(long)]
    pub non_strict_mode: bool,
    // todo
    // /// Store initial data from genesis like Accounts, AccessKeys
    // #[clap(long)]
    // pub store_genesis: bool,
    /// AWS S3 bucket name to get the stream from
    #[clap(long)]
    pub s3_bucket_name: String,
    /// AWS S3 bucket region
    #[clap(long)]
    pub s3_region_name: String,
    /// Block height to start the stream from
    #[clap(long, short)]
    pub start_block_height: u64,
}
