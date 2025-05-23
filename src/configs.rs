use clap::{Parser, ValueEnum};

/// NEAR Lake
/// Watches for stream of blocks from the chain and puts it in S3 bucket
#[derive(Parser, Clone, Debug)]
#[clap(
    version,
    author,
    about,
    disable_help_subcommand(true),
    disable_help_flag(true),
    propagate_version(true),
    next_line_help(true)
)]
pub(crate) struct Opts {
    /// Sets a custom config dir. Defaults to ~/.near/
    #[clap(short, long)]
    pub home: Option<std::path::PathBuf>,
    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Parser, Clone, Debug)]
pub(crate) enum SubCommand {
    /// Run NEAR Indexer Example. Start observe the network
    Run(RunArgs),
    /// Initialize necessary configs
    Init(Box<InitConfigArgs>), // clippy suggestion: consider boxing the large fields to reduce the total size of the enum
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct RunArgs {
    /// AWS S3 compatible API Endpoint
    #[clap(long, env)]
    pub endpoint: Option<http::Uri>,
    /// Name of S3 bucket
    #[clap(long, env)]
    pub bucket: String,
    /// Region of S3 bucket
    #[clap(long, env)]
    pub region: String,
    /// Fallback region of S3
    #[clap(long, default_value = "eu-central-1")]
    pub fallback_region: String,
    /// Force streaming while node is syncing
    #[clap(long)]
    pub stream_while_syncing: bool,
    /// Tells whether to validate the genesis file before starting
    #[clap(long)]
    pub validate_genesis: bool,
    /// Sets the concurrency for indexing. Note: concurrency (set to 2+) may lead to warnings
    /// due to tight constraints between transactions and receipts (those will get resolved
    /// eventually, but unless it is the second pass of indexing, concurrency won't help at the moment).
    #[clap(long, default_value = "1")]
    pub concurrency: std::num::NonZeroU16,
    /// Sets the starting point for indexing
    #[clap(subcommand)]
    pub sync_mode: SyncModeSubCommand,
    /// Sets the different types of finality
    #[clap(long, value_enum, default_value_t = FinalityArg::Final)]
    pub finality: FinalityArg,
}

impl RunArgs {
    pub(crate) fn to_indexer_config(
        &self,
        home_dir: std::path::PathBuf,
    ) -> near_indexer::IndexerConfig {
        near_indexer::IndexerConfig {
            home_dir,
            sync_mode: self.sync_mode.clone().into(),
            await_for_node_synced: if self.stream_while_syncing {
                near_indexer::AwaitForNodeSyncedEnum::StreamWhileSyncing
            } else {
                near_indexer::AwaitForNodeSyncedEnum::WaitForFullSync
            },
            finality: self.finality.clone().into(),
            validate_genesis: self.validate_genesis,
        }
    }
}

#[allow(clippy::enum_variant_names)] // we want commands to be more explicit
#[derive(Parser, Debug, Clone)]
pub(crate) enum SyncModeSubCommand {
    /// continue from the block Indexer was interrupted
    SyncFromInterruption,
    /// start from the newest block after node finishes syncing
    SyncFromLatest,
    /// start from specified block height
    SyncFromBlock(BlockArgs),
}

/// Different types of finality.
#[derive(Debug, ValueEnum, Clone)]
#[clap(rename_all = "snake_case")]
pub enum FinalityArg {
    Optimistic,
    NearFinal,
    Final,
}

impl From<FinalityArg> for near_indexer::near_primitives::types::Finality {
    fn from(value: FinalityArg) -> Self {
        match value {
            FinalityArg::Optimistic => near_indexer::near_primitives::types::Finality::None,
            FinalityArg::NearFinal => near_indexer::near_primitives::types::Finality::DoomSlug,
            FinalityArg::Final => near_indexer::near_primitives::types::Finality::Final,
        }
    }
}

#[derive(Parser, Debug, Clone)]
pub(crate) struct BlockArgs {
    /// block height for block sync mode
    #[clap(long)]
    pub height: u64,
}

impl From<SyncModeSubCommand> for near_indexer::SyncModeEnum {
    fn from(sync_mode: SyncModeSubCommand) -> Self {
        match sync_mode {
            SyncModeSubCommand::SyncFromInterruption => Self::FromInterruption,
            SyncModeSubCommand::SyncFromLatest => Self::LatestSynced,
            SyncModeSubCommand::SyncFromBlock(args) => Self::BlockHeight(args.height),
        }
    }
}

#[derive(Parser, Clone, Debug)]
pub(crate) struct InitConfigArgs {
    /// chain/network id (localnet, testnet, devnet, betanet)
    #[clap(short, long)]
    pub chain_id: Option<String>,
    /// Account ID for the validator key
    #[clap(long)]
    pub account_id: Option<String>,
    /// Specify private key generated from seed (TESTING ONLY)
    #[clap(long)]
    pub test_seed: Option<String>,
    /// Number of shards to initialize the chain with
    #[clap(short, long, default_value = "1")]
    pub num_shards: u64,
    /// Makes block production fast (TESTING ONLY)
    #[clap(short, long)]
    pub fast: bool,
    /// Genesis file to use when initialize testnet (including downloading)
    #[clap(short, long)]
    pub genesis: Option<String>,
    #[clap(short, long)]
    /// Download the verified NEAR config file automatically.
    #[clap(long)]
    pub download_config: bool,
    #[clap(long)]
    pub download_config_url: Option<String>,
    /// Download the verified NEAR genesis file automatically.
    #[clap(long)]
    pub download_genesis: bool,
    /// Specify a custom download URL for the genesis-file.
    #[clap(long)]
    pub download_genesis_url: Option<String>,
    #[clap(long)]
    pub donwload_genesis_records_url: Option<String>,
    /// Customize max_gas_burnt_view runtime limit.  If not specified, value
    /// from genesis configuration will be taken.
    #[clap(long)]
    pub max_gas_burnt_view: Option<u64>,
    /// Initialize boots nodes in <node_key>@<ip_addr> format seperated by commas
    /// to bootstrap the network and store them in config.json
    #[clap(long)]
    pub boot_nodes: Option<String>,
}
