use std::collections::HashMap;
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use alloy::eips::BlockId;
use alloy::primitives::{Address, StorageValue};
use alloy::{
    network::Ethereum,
    primitives::{BlockNumber, Bytes, U256, U64},
    providers::{EthCall, Provider, RootProvider, RpcWithBlock},
    rpc::{
        client::RpcCall,
        json_rpc::{Id, Request, RpcReturn},
        types::{Block, BlockNumberOrTag, FilterChanges, TransactionRequest},
    },
    transports::{Transport, TransportResult},
};
use log::debug;
use tokio::sync::RwLock;

use crate::httpcached::HttpCachedTransport;

#[derive(Clone)]
pub struct ArchiveHistoryProvider<P, T> {
    provider: P,
    current_block: Arc<AtomicU64>,
    new_block_filter: Arc<RwLock<HashMap<U256, u64>>>,
    _t: PhantomData<T>,
}

impl<P, T> ArchiveHistoryProvider<P, T>
where
    T: Transport + Clone,
    P: Provider<T, Ethereum> + Send + Sync + Clone + 'static,
{
}

impl<P, T> ArchiveHistoryProvider<P, T>
where
    T: Transport + Clone,
    P: Provider<T, Ethereum> + Send + Sync + Clone + 'static,
{
    pub fn block_number(&self) -> u64 {
        self.current_block.load(Ordering::Relaxed)
    }

    pub fn block_id(&self) -> BlockId {
        BlockId::Number(BlockNumberOrTag::Number(self.block_number()))
    }
}

#[allow(dead_code)]
impl<P> ArchiveHistoryProvider<P, HttpCachedTransport>
where
    P: Provider<HttpCachedTransport, Ethereum> + Send + Sync + Clone + 'static,
{
    pub fn new(provider: P, start_block: u64) -> Self {
        provider.client().transport().set_block_number(start_block);
        Self {
            provider,
            current_block: Arc::new(AtomicU64::new(start_block)),
            new_block_filter: Arc::new(RwLock::new(HashMap::new())),
            _t: PhantomData,
        }
    }

    pub fn next_block(&self) -> u64 {
        let block = self.current_block.fetch_add(1, Ordering::Relaxed);
        let previous_block = self.provider.client().transport().set_block_number(block);
        println!("Change block {previous_block} -> {block} ");
        block
    }
}

#[async_trait::async_trait]
impl<P, T> Provider<T, Ethereum> for ArchiveHistoryProvider<P, T>
where
    T: Transport + Clone,
    P: Provider<T, Ethereum> + Send + Sync + Clone + 'static,
{
    fn root(&self) -> &RootProvider<T, Ethereum> {
        self.provider.root()
    }

    #[allow(clippy::type_complexity)]
    fn get_block_number(&self) -> RpcCall<T, [(); 0], U64, BlockNumber> {
        let rpc_call: RpcCall<T, [(); 0], U64, BlockNumber, fn(U64) -> BlockNumber> =
            RpcCall::new(Request::new("get_block_number", Id::None, [(); 0]), self.provider.client().transport().clone())
                .map_resp(|x: U64| x.to());
        rpc_call
    }

    fn call<'req, 'state>(&self, tx: &'req TransactionRequest) -> EthCall<'req, 'state, T, Ethereum, Bytes> {
        let call = EthCall::new(self.weak_client(), tx).block(self.block_id());
        debug!("call {:?}", self.block_id());
        call
    }

    fn get_storage_at(&self, address: Address, key: U256) -> RpcWithBlock<T, (Address, U256), StorageValue> {
        debug!("get_storage_at {:?}", self.block_id());
        RpcWithBlock::new(self.weak_client(), "eth_getStorageAt", (address, key)).block_id(self.block_id())
    }

    fn get_block_by_number<'life0, 'async_trait>(
        &'life0 self,
        number: BlockNumberOrTag,
        hydrate: bool,
    ) -> Pin<Box<dyn Future<Output = TransportResult<Option<Block>>> + Send + 'async_trait>>
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        self.provider.get_block_by_number(number, hydrate)
    }

    async fn get_filter_changes<R: RpcReturn>(&self, id: U256) -> TransportResult<Vec<R>> {
        println!("get_filter_changes");
        //let pin_v = Box::pin(vec![U256::ZERO]);

        //Ok(R::try_from(pin_v)?)
        // let new_block_filter_guard = self.new_block_filter.write().await;
        //
        // if let Some(block_id) = new_block_filter_guard.get(&id) {
        //     if self.block_number() > *block_id {
        //         Ok(Vec::<U256>::new())
        //     } else {
        //         Ok(Vec::<U256>::new())
        //     }
        // } else {
        //     self.provider.get_filter_changes(id).await
        // }
        self.provider.get_filter_changes(id).await
    }

    async fn get_filter_changes_dyn(&self, id: U256) -> TransportResult<FilterChanges> {
        println!("get_filter_changes_dyn");

        //let pin_v = Box::pin(vec![U256::ZERO]);

        //Ok(R::try_from(pin_v)?)
        // let new_block_filter_guard = self.new_block_filter.write().await;
        //
        // if let Some(block_id) = new_block_filter_guard.get(&id) {
        //     if self.block_number() > *block_id {
        //         Ok(Vec::<U256>::new())
        //     } else {
        //         Ok(Vec::<U256>::new())
        //     }
        // } else {
        //     self.provider.get_filter_changes(id).await
        // }
        self.provider.get_filter_changes_dyn(id).await
    }

    async fn new_block_filter(&self) -> TransportResult<U256> {
        let result = self.provider.new_block_filter().await;
        let cur_block = self.block_number();
        if let Ok(filter_id) = &result {
            self.new_block_filter.write().await.insert(*filter_id, cur_block);
        }
        result
    }
}

#[cfg(test)]
mod test {}
