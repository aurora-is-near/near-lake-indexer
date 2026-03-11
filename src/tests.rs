use crate::{Stats, handle_message, init_tracing};
use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::config::Credentials;
use futures::StreamExt;
use near_indexer_primitives::StreamerMessage;
use std::ffi::OsStr;
use std::sync::Arc;
use tar::Archive;
use testcontainers_modules::minio::MinIO;
use testcontainers_modules::testcontainers::core::{ContainerPort, Mount};
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, ImageExt};
use tokio::sync::{Mutex, mpsc};
use zstd::Decoder;

const BLOCKS_NUMBER: usize = 100;
const MINIO_ROOT_USER: &str = "minioadmin";
const MINIO_ROOT_PASSWORD: &str = "minioadmin";

#[tokio::test]
async fn test_sending_blocks_in_parallel() {
    init_tracing();
    let s3 = S3Container::start().await;
    let (sender, receiver) = mpsc::channel(BLOCKS_NUMBER);
    blocks_reader(sender).await.unwrap();
    let stats = Arc::new(Mutex::new(Stats::new()));
    let client = create_client().await;

    let mut blocks = 0;
    let start = std::time::Instant::now();
    let mut handle_messages = tokio_stream::wrappers::ReceiverStream::new(receiver)
        .map(|streamer_message| {
            handle_message(
                &client,
                streamer_message,
                "blocks".to_string(),
                Arc::clone(&stats),
            )
        })
        .buffer_unordered(2);

    while let Some(result) = handle_messages.next().await {
        assert!(result.is_ok());
        blocks += 1;
    }

    tracing::info!(
        "handle {blocks} blocks took: {:.5} seconds",
        start.elapsed().as_secs_f64()
    );

    assert_eq!(
        stats.lock().await.blocks_processed_count,
        u64::try_from(BLOCKS_NUMBER).expect("Failed to convert blocks number to u64")
    );

    s3.stop().await.unwrap();
}

async fn blocks_reader(sender: mpsc::Sender<StreamerMessage>) -> anyhow::Result<()> {
    decompress_blocks()?;

    let mut dir = tokio::fs::read_dir("blocks").await?;
    let mut files = tokio::task::JoinSet::new();

    while let Some(entry) = dir.next_entry().await? {
        if !is_block(&entry) {
            continue;
        }

        let s = sender.clone();

        files.spawn(async move {
            let bytes = tokio::fs::read(entry.path()).await.unwrap();
            let block: StreamerMessage = serde_json::from_slice(&bytes).unwrap();
            s.send(block).await.unwrap();
        });
    }

    let result = files.join_all().await;
    assert_eq!(result.len(), 100);

    Ok(())
}

async fn create_client() -> aws_sdk_s3::Client {
    let region_provider =
        RegionProviderChain::first_try(Some(aws_sdk_s3::config::Region::new("localhost")))
            .or_default_provider();
    let credentials = Credentials::new(
        MINIO_ROOT_USER,
        MINIO_ROOT_PASSWORD,
        None,
        None,
        "aws-sdk-rust",
    );
    let shared_config = aws_config::defaults(aws_config::BehaviorVersion::latest())
        .region(region_provider)
        .endpoint_url("http://127.0.0.1:9000")
        .credentials_provider(credentials)
        .load()
        .await;
    let s3_conf = aws_sdk_s3::config::Builder::from(&shared_config);
    let client = aws_sdk_s3::Client::from_conf(s3_conf.build());

    client
        .create_bucket()
        .bucket("blocks")
        .send()
        .await
        .unwrap();

    client
}

fn decompress_blocks() -> anyhow::Result<()> {
    let file = std::fs::File::open("blocks.tar.zst")?;
    let reader = std::io::BufReader::new(file);
    let zst_decoder = Decoder::new(reader)?;
    let mut archive = Archive::new(zst_decoder);

    archive.unpack(".")?;
    Ok(())
}

fn is_block(entry: &tokio::fs::DirEntry) -> bool {
    entry
        .path()
        .file_name()
        .and_then(OsStr::to_str)
        .map_or(false, |f| !f.starts_with('.'))
}

struct S3Container {
    c: ContainerAsync<MinIO>,
}

impl S3Container {
    async fn start() -> Self {
        let c = MinIO::default()
            .with_mount(Mount::tmpfs_mount("/data"))
            .with_mapped_port(9000, ContainerPort::Tcp(9000))
            .with_env_var("MINIO_ROOT_USER", MINIO_ROOT_USER)
            .with_env_var("MINIO_ROOT_PASSWORD", MINIO_ROOT_PASSWORD);
        let c = c.start().await.unwrap();

        Self { c }
    }

    async fn stop(self) -> anyhow::Result<()> {
        self.c.stop().await?;
        self.c.rm().await.map_err(Into::into)
    }
}
