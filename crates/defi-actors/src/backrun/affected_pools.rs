use std::collections::BTreeMap;
use std::sync::Arc;

use alloy_eips::{BlockId, BlockNumberOrTag};
use alloy_primitives::{Address, hex};
use alloy_provider::Provider;
use eyre::{eyre, Result};
use log::{debug, error, info};
use revm::db::{CacheDB, DatabaseCommit, EmptyDB};
use revm::precompile::Bytes;
use revm::primitives::{Bytecode, Env, U256};

use defi_entities::{Market, MarketState, PoolProtocol, PoolWrapper};
use defi_pools::{UniswapV2Pool, UniswapV3Pool};
use defi_pools::protocols::{UniswapV2Protocol, UniswapV3Protocol};
use defi_pools::state_readers::{UniswapV2StateReader, UniswapV3StateReader};
use defi_types::GethStateUpdateVec;
use loom_actors::SharedState;

use crate::market::get_protocol_by_factory;

pub async fn get_affected_pools(
    market: SharedState<Market>,
    state_update: &GethStateUpdateVec,
) -> Result<BTreeMap<PoolWrapper, Vec<(Address, Address)>>>
{
    let market_guard = market.read().await;

    let mut affected_pools: BTreeMap<PoolWrapper, Vec<(Address, Address)>> = BTreeMap::new();

    for state_update_record in state_update.iter() {
        for (address, state_update_entry) in state_update_record.iter() {
            if let Some(pool) = market_guard.get_pool(address) {
                if affected_pools.contains_key(pool) || !market_guard.is_pool(address) {
                    continue;
                }
                let swap_directions = pool.get_swap_directions();
                affected_pools.insert(pool.clone(), swap_directions.clone());
            }
        }
    }

    Ok((affected_pools))
}


pub async fn get_affected_pools_from_code<P>(
    client: P,
    market: SharedState<Market>,
    state_update: &GethStateUpdateVec,
) -> Result<BTreeMap<PoolWrapper, Vec<(Address, Address)>>>
    where P: Provider + Send + Sync + Clone + 'static
{
    let db = CacheDB::new(EmptyDB::new());
    let mut market_state = MarketState::new(db);
    market_state.apply_state_update(&state_update, false, false);


    let mut ret: BTreeMap<PoolWrapper, Vec<(Address, Address)>> = BTreeMap::new();

    for state_update_record in state_update.iter() {
        for (address, state_update_entry) in state_update_record.iter() {
            match &state_update_entry.code {
                Some(code) => {
                    if UniswapV2Protocol::is_code(code) {
                        match market.read().await.get_pool(address) {
                            None => {
                                info!("Loading UniswapV2 class pool {address:?}");
                                let env = Env::default();
                                match UniswapV2StateReader::factory(&market_state.state_db, env.clone(), *address) {
                                    Ok(factory_address) => {
                                        if factory_address.is_zero() {
                                            for i in 5u64..8 {
                                                match client.get_storage_at(*address, U256::from(i), BlockId::Number(BlockNumberOrTag::Latest)).await {
                                                    Ok(data) => {
                                                        //info!("---- {} {} {:?}", address, i, data);
                                                        market_state.state_db.insert_account_storage(*address, U256::try_from(i).unwrap(), data);
                                                    }
                                                    _ => {}
                                                }
                                            }
                                        }

                                        match UniswapV2Pool::fetch_pool_data_evm(&market_state.state_db, env.clone(), *address) {
                                            Ok(pool) => {
                                                let pool = PoolWrapper::new(Arc::new(pool));
                                                info!("UniswapV2 Pool loaded {address:?} {}", pool.get_protocol());
                                                let swap_directions = pool.get_swap_directions();
                                                ret.insert(pool, swap_directions);
                                            }
                                            Err(e) => {
                                                error!("Error loading UniswapV2 pool @{address:?}: {e}");
                                            }
                                        }
                                    }
                                    Err(e) => { error!("Error loading UniswapV2 factory {e}") }
                                }
                            }
                            Some(pool) => { debug!("Pool already exists {address} {}", pool.get_protocol()); }
                        }
                    }


                    if UniswapV3Protocol::is_code(code) {
                        match market.read().await.get_pool(address) {
                            None => {
                                info!("Loading UniswapV3 class pool {address:?}");
                                let env = Env::default();
                                match UniswapV3StateReader::factory(&market_state.state_db, env.clone(), *address) {
                                    Ok(factory_address) => {
                                        let fetch_result = match get_protocol_by_factory(factory_address) {
                                            PoolProtocol::PancakeV3 => {
                                                //TODO : Add pancake v3
                                                /*let mut pool = PancakeV3Pool::new(*address);
                                                match pool.fetch_pool_data_evm(&market_state.state_db, env.clone()) {
                                                    Ok(_) => {
                                                        info!("PancakeV3 Pool loaded {address:?} {}", pool.get_protocol());
                                                        let swap_directions = pool.get_swap_directions();
                                                        ret.insert(Arc::new(PoolWrapper::new(Arc::new(pool))), swap_directions);
                                                    }
                                                    Err(e) => {
                                                        error!("Error loading PancakeV3 pool @{address:?}: {e}");
                                                    }
                                                }

                                                 */
                                            }
                                            PoolProtocol::Maverick => {
                                                //TODO : Add maverick
                                                /*
                                                let mut pool = MaverickPool::new(*address);
                                                match pool.fetch_pool_data_evm(&market_state.state_db, env.clone()) {
                                                    Ok(_) => {
                                                        info!("Maverick Pool loaded {address:?} {}", pool.get_protocol() );
                                                        let swap_directions = pool.get_swap_directions();
                                                        ret.insert(Arc::new(PoolWrapper::new(Arc::new(pool))), swap_directions);
                                                    }
                                                    Err(e) => {
                                                        error!("Error loading Maverick pool @{address:?}: {e}");
                                                    }
                                                }

                                                 */
                                            }
                                            _ => {
                                                match UniswapV3Pool::fetch_pool_data_evm(&market_state.state_db, env.clone(), *address) {
                                                    Ok(pool) => {
                                                        let pool = PoolWrapper::new(Arc::new(pool));
                                                        info!("UniswapV3 Pool loaded {address:?} {}", pool.get_protocol());
                                                        let swap_directions = pool.get_swap_directions();
                                                        ret.insert(pool, swap_directions);
                                                    }
                                                    Err(e) => {
                                                        error!("Error loading UniswapV3 pool @{address:?}: {e}");
                                                    }
                                                }
                                            }
                                        };
                                    }
                                    Err(e) => { error!("Error loading UniswapV3 factory {e}") }
                                }
                            }
                            Some(pool) => { debug!("Pool already exists {address} {}", pool.get_protocol()) }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    if ret.len() > 0 {
        Ok(ret)
    } else {
        Err(eyre!("NO_POOLS_LOADED"))
    }
}

pub fn is_pool_code(
    state_update: &GethStateUpdateVec
) -> bool
{
    for state_update_record in state_update.iter() {
        for (address, state_update_entry) in state_update_record.iter() {
            match &state_update_entry.code {
                Some(code) => {
                    if UniswapV3Protocol::is_code(code) {
                        return true;
                    }
                    if UniswapV2Protocol::is_code(code) {
                        return true;
                    }
                }
                _ => {}
            }
        }
    }
    false
}