use alloy_network::Ethereum;
use alloy_primitives::BlockHash;
use alloy_provider::Provider;
use alloy_rpc_types::{Block, BlockTransactionsKind};
use alloy_transport::Transport;
use log::error;
use loom_actors::{subscribe, Broadcaster, WorkerResult};

pub async fn new_block_with_tx_worker<P, T>(
    client: P,
    block_hash_receiver: Broadcaster<BlockHash>,
    sender: Broadcaster<Block>,
) -> WorkerResult
where
    T: Transport + Clone,
    P: Provider<T, Ethereum> + Send + Sync + 'static,
{
    subscribe!(block_hash_receiver);

    loop {
        if let Ok(block_hash) = block_hash_receiver.recv().await {
            if let Ok(Some(block_with_txes)) = client.get_block_by_hash(block_hash, BlockTransactionsKind::Full).await {
                if let Err(e) = sender.send(block_with_txes).await {
                    error!("Broadcaster error {}", e);
                }
            }
        }
    }
}
