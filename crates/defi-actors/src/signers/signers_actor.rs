use std::sync::{Arc, Mutex, RwLock};

use alloy_consensus::{SignableTransaction, Signed, TxEnvelope};
use alloy_network::{Ethereum, Network, TransactionBuilder};
use alloy_primitives::Bytes;
use alloy_rlp::Encodable;
use alloy_rpc_types::{Transaction, TransactionRequest};
use async_trait::async_trait;
use eyre::{eyre, Result};
use log::{error, info};
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::broadcast::Receiver;

use defi_entities::TxSigners;
use defi_events::{MessageTxCompose, RlpState, TxCompose, TxComposeData, TxState};
use loom_actors::{Accessor, Actor, ActorResult, Broadcaster, Consumer, MultiProducer, Producer, SharedState, WorkerResult};
use loom_actors_macros::{Accessor, Consumer, Producer};

async fn sign_task(
    sign_request: TxComposeData,
    compose_channel_tx: Broadcaster<MessageTxCompose>,
) -> Result<()> {
    let signer = sign_request.signer.clone().unwrap();

    let rlp_bundle: Vec<RlpState> = sign_request.tx_bundle.clone().unwrap().iter().map(|tx_request| {
        match &tx_request {
            TxState::Stuffing(t) => {
                let typed_tx: Result<TxEnvelope, _> = t.clone().try_into();

                match typed_tx {
                    Ok(typed_tx) => {
                        let mut v: Vec<u8> = Vec::new();
                        typed_tx.encode(&mut v);
                        RlpState::Stuffing(v.into())
                    }
                    _ => {
                        RlpState::None
                    }
                }
            }
            TxState::SignatureRequired(t) => {
                let (tx_hash, signed_tx_bytes) = signer.sign_sync(t.clone()).unwrap();
                info!("Tx signed {tx_hash:?}");
                RlpState::Backrun(signed_tx_bytes)
            }
            TxState::ReadyForBroadcast(t) => {
                RlpState::Backrun(t.clone())
            }
            TxState::ReadyForBroadcastStuffing(t) => {
                RlpState::Stuffing(t.clone())
            }
            _ => {
                error!("Cannot process bundle tx : {:?}", tx_request);
                RlpState::None
            }
        }
    }).collect();

    if rlp_bundle.iter().any(|item| item.is_none()) {
        error!("Bundle is not ready. Cannot sign");
        return Err(eyre!("CANNOT_SIGN_BUNDLE"));
    }
    //let rlp_bundle= rlp_bundle.into_iter().map(|item| item.unwrap()).collect();

    let broadcast_request = TxComposeData {
        rlp_bundle: Some(rlp_bundle),
        ..sign_request
    };


    match compose_channel_tx.send(
        MessageTxCompose::broadcast(broadcast_request),
    ).await {
        Err(e) => {
            error!("{e}");
            Err(eyre!("BROADCAST_ERROR"))
        }
        _ => { Ok(()) }
    }
}

async fn request_listener_worker(
    mut compose_channel_rx: Receiver<MessageTxCompose>,
    compose_channel_tx: Broadcaster<MessageTxCompose>)
    -> WorkerResult
{
    loop {
        tokio::select! {
            msg = compose_channel_rx.recv() => {
                let compose_request_msg : Result<MessageTxCompose, RecvError> = msg;
                match compose_request_msg {
                    Ok(compose_request) =>{

                        match compose_request.inner {
                            TxCompose::Sign( sign_request)=>{
                                //let rlp_bundle : Vec<Option<Bytes>> = Vec::new();
                                tokio::task::spawn(
                                    sign_task(
                                        sign_request,
                                        compose_channel_tx.clone(),
                                    )
                                );
                            },
                            _=>{}
                        }

                    }
                    Err(e)=>{error!("{}",e)}
                }
            }
        }
    }
}


#[derive(Accessor, Consumer, Producer)]
pub struct SignersActor
{
    #[accessor]
    signers: Option<SharedState<TxSigners>>,
    #[consumer]
    compose_channel_rx: Option<Broadcaster<MessageTxCompose>>,
    #[producer]
    compose_channel_tx: Option<Broadcaster<MessageTxCompose>>,

}

impl SignersActor
{
    pub fn new() -> SignersActor {
        SignersActor {
            signers: None,
            compose_channel_rx: None,
            compose_channel_tx: None,
        }
    }
}


#[async_trait]
impl Actor for SignersActor
{
    async fn start(&mut self) -> ActorResult {
        let task = tokio::task::spawn(
            request_listener_worker(
                self.compose_channel_rx.clone().unwrap().subscribe().await,
                self.compose_channel_tx.clone().unwrap(),
            )
        );


        Ok(vec![task])
    }
}