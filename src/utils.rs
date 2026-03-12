use near_async::messaging::CanSendAsync;
use near_client::ViewClientActor;
use near_indexer_primitives::types;

/// Fetches the status to retrieve `latest_block_height` to determine if we need to fetch
/// entire block or we already fetched this block.
pub(crate) async fn fetch_latest_block(
    client: &near_async::multithread::MultithreadRuntimeHandle<ViewClientActor>,
) -> anyhow::Result<u64> {
    let block = client
        .send_async(near_client::GetBlock(types::BlockReference::Finality(
            types::Finality::Final,
        )))
        .await??;
    Ok(block.header.height)
}
