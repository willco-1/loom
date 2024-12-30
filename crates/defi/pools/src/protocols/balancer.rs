use std::fmt::{Display, Formatter};
use std::marker::PhantomData;

use alloy::primitives::{address, Address, Bytes, U256};
use alloy::providers::{Network, Provider};
use alloy::rpc::types::{BlockId, BlockNumberOrTag};
use alloy::sol_types::SolInterface;
use alloy::transports::Transport;
use eyre::{eyre, Report, Result};
use tracing::{debug, error, trace};
use loom_defi_abi::balancer::IVault::*;
#[derive(Clone, Debug)]
pub struct Vault {
    address: Address, // Address of the Vault contract
    authorizer: Option<Address>, // Current authorizer address
    weth: Option<Address>, // WETH token address
    protocol_fees_collector: Option<Address>, // Protocol fees collector address
    paused: bool, // Whether the Vault is paused
    pools: Vec<(Bytes, Address, PoolSpecialization)>, // Registered pools (pool ID, pool address, specialization)
    relayer_approvals: Vec<(Address, Address, bool)>, // Relayer approvals (user, relayer, approved)
    internal_balances: Vec<(Address, Vec<(Address, U256)>)>, // Internal balances (user, Vec<(token, balance)>)
    pool_tokens: Vec<(Bytes, Vec<Address>, Vec<U256>)>, // Pool tokens (pool ID, tokens, balances)
}
impl Vault {
    pub fn new(address: Address) -> Self {
        Vault {
            address,
            authorizer: None,
            weth: None,
            protocol_fees_collector: None,
            paused: false,
            pools: Vec::new(),
            relayer_approvals: Vec::new(),
            internal_balances: Vec::new(),
            pool_tokens: Vec::new(),
        }
    }

    pub fn new_with_data(
        address: Address,
        authorizer: Option<Address>,
        weth: Option<Address>,
        protocol_fees_collector: Option<Address>,
        paused: bool,
        pools: Vec<(Bytes, Address, PoolSpecialization)>,
        relayer_approvals: Vec<(Address, Address, bool)>,
        internal_balances: Vec<(Address, Vec<(Address, U256)>)>,
        pool_tokens: Vec<(Bytes, Vec<Address>, Vec<U256>)>,
    ) -> Self {
        Vault {
            address,
            authorizer,
            weth,
            protocol_fees_collector,
            paused,
            pools,
            relayer_approvals,
            internal_balances,
            pool_tokens,
        }
    }
}
impl Vault {
    pub fn address(&self) -> Address {
        self.address
    }

    pub fn authorizer(&self) -> Option<Address> {
        self.authorizer
    }

    pub fn weth(&self) -> Option<Address> {
        self.weth
    }

    pub fn protocol_fees_collector(&self) -> Option<Address> {
        self.protocol_fees_collector
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

        pub fn relayer_approvals(&self) -> &Vec<(Address, Address, bool)> {
        &self.relayer_approvals
    }

    pub fn internal_balances(&self) -> &Vec<(Address, Vec<(Address, U256)>)> {
        &self.internal_balances
    }

    pub fn pool_tokens(&self) -> &Vec<(Bytes, Vec<Address>, Vec<U256>)> {
        &self.pool_tokens
    }

}

